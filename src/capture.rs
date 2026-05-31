use crate::error::Result;
use anyhow::anyhow;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// Source of `hook.cpp` embedded at compile time so that `cargo install` works
/// without requiring a separate `hook/` directory on the user's machine.
const HOOK_CPP_SRC: &str = include_str!("../hook/hook.cpp");

/// Returns the per-user cache directory for cpp2rust-demo artefacts.
///
/// Priority: `$XDG_DATA_HOME/cpp2rust-demo` → `~/.local/share/cpp2rust-demo` → `$TMPDIR/cpp2rust-demo`.
fn user_hook_cache_dir() -> PathBuf {
    let base = std::env::var("XDG_DATA_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            std::env::var("HOME")
                .map(|h| PathBuf::from(h).join(".local").join("share"))
                .unwrap_or_else(|_| std::env::temp_dir())
        });
    base.join("cpp2rust-demo")
}

/// Compile `libhook.so` from the source code embedded in the binary.
///
/// The source is written to the user cache directory and compiled with `g++`.
/// This is the fallback path used after `cargo install` when the `hook/`
/// source directory is not adjacent to the installed binary.
fn build_hook_embedded() -> Result<PathBuf> {
    let cache_dir = user_hook_cache_dir();
    std::fs::create_dir_all(&cache_dir)
        .map_err(|e| anyhow!("create cache dir {}: {}", cache_dir.display(), e))?;

    let src = cache_dir.join("hook.cpp");
    let so = cache_dir.join("libhook.so");

    std::fs::write(&src, HOOK_CPP_SRC)
        .map_err(|e| anyhow!("write hook.cpp to {}: {}", src.display(), e))?;

    println!("Compiling hook library from embedded source to {}...", cache_dir.display());
    let status = Command::new("g++")
        .args(["-Wall", "-fPIC", "-shared", "-o"])
        .arg(&so)
        .arg(&src)
        .arg("-ldl")
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|e| anyhow!("g++ not found (required to compile hook): {}", e))?;

    if !status.success() {
        return Err(anyhow!("failed to compile hook.cpp to {}", so.display()));
    }

    println!("Hook library compiled: {}", so.display());
    Ok(so)
}

/// Build `libhook.so`, preferring a local `hook/Makefile` when available and
/// falling back to compiling the embedded source for `cargo install` setups.
///
/// Returns the path to the built `libhook.so`.
pub fn build_hook() -> Result<PathBuf> {
    match hook_dir() {
        Ok(dir) => {
            let so = dir.join("libhook.so");
            println!("Building hook library from {}...", dir.display());
            let status = Command::new("make")
                .current_dir(&dir)
                .arg("-s")
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .status()
                .map_err(|e| anyhow!("failed to run make in {}: {}", dir.display(), e))?;
            if !status.success() {
                return Err(anyhow!("make failed in {}", dir.display()));
            }
            if !so.exists() {
                return Err(anyhow!("libhook.so not found after build at {}", so.display()));
            }
            println!("Hook library built: {}", so.display());
            Ok(so)
        }
        Err(_) => build_hook_embedded(),
    }
}

/// Execute the user-supplied build command with LD_PRELOAD set to libhook.so.
pub fn run_with_hook(
    build_dir: &Path,
    cmd: &[String],
    project_root: &Path,
    feature_root: &Path,
    hook_so: &Path,
) -> Result<()> {
    if cmd.is_empty() {
        return Err(anyhow!("build command is empty"));
    }

    let abs_project_root = project_root
        .canonicalize()
        .map_err(|e| anyhow!("canonicalize {}: {}", project_root.display(), e))?;
    let abs_feature_root = feature_root
        .canonicalize()
        .map_err(|e| anyhow!("canonicalize {}: {}", feature_root.display(), e))?;
    let abs_hook = hook_so
        .canonicalize()
        .map_err(|e| anyhow!("canonicalize {}: {}", hook_so.display(), e))?;

    println!("Running build command: {}", cmd.join(" "));
    println!("  CPP2RUST_PROJECT_ROOT = {}", abs_project_root.display());
    println!("  CPP2RUST_FEATURE_ROOT = {}", abs_feature_root.display());
    println!("  LD_PRELOAD            = {}", abs_hook.display());
    println!();

    let status = Command::new(&cmd[0])
        .args(&cmd[1..])
        .current_dir(build_dir)
        .env("LD_PRELOAD", &abs_hook)
        .env("CPP2RUST_PROJECT_ROOT", &abs_project_root)
        .env("CPP2RUST_FEATURE_ROOT", &abs_feature_root)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|e| anyhow!("failed to spawn '{}': {}", cmd[0], e))?;

    if !status.success() {
        return Err(anyhow!(
            "build command failed with exit code {}",
            status.code().unwrap_or(-1)
        ));
    }
    Ok(())
}

/// Locate the `hook/` directory, starting from the directory of the running
/// binary and searching upward. Falls back to a path relative to the manifest.
fn hook_dir() -> Result<PathBuf> {
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            let candidate = parent.join("hook");
            if candidate.join("Makefile").exists() {
                return Ok(candidate);
            }
            if let Some(workspace) = parent.parent().and_then(|p| p.parent()).and_then(|p| p.parent()) {
                let candidate = workspace.join("hook");
                if candidate.join("Makefile").exists() {
                    return Ok(candidate);
                }
            }
        }
    }

    let cwd_candidate = std::env::current_dir()
        .map_err(|e| anyhow!("current_dir: {}", e))?
        .join("hook");
    if cwd_candidate.join("Makefile").exists() {
        return Ok(cwd_candidate);
    }

    Err(anyhow!(
        "hook/ directory with Makefile not found (searched near binary and cwd)"
    ))
}
