use crate::error::Result;
use anyhow::anyhow;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

// 将 hook 源文件内容直接嵌入 binary，确保 `cargo install` 后无需额外文件。
const HOOK_CPP: &str = include_str!("../hook/hook.cpp");
const HOOK_MAKEFILE: &str = include_str!("../hook/Makefile");

/// Build the `libhook.so` from `hook/Makefile`.
///
/// If `libhook.so` already exists and is newer than `hook.cpp`, compilation is
/// skipped ("up-to-date" fast path).  Returns the path to `libhook.so`.
pub fn build_hook() -> Result<PathBuf> {
    let hook_dir = hook_dir()?;
    let so = hook_dir.join("libhook.so");
    let cpp = hook_dir.join("hook.cpp");

    // Fast path: skip recompilation when .so is newer than hook.cpp.
    // Note: hook_dir() above already calls ensure_hook_data_dir() as its
    // final fallback, so hook.cpp is guaranteed to exist in the data-dir
    // case before this check runs.
    if so.exists() && cpp.exists() {
        if let (Ok(so_meta), Ok(cpp_meta)) = (so.metadata(), cpp.metadata()) {
            if let (Ok(so_mtime), Ok(cpp_mtime)) =
                (so_meta.modified(), cpp_meta.modified())
            {
                if so_mtime >= cpp_mtime {
                    println!("Hook library up-to-date: {}", so.display());
                    return Ok(so);
                }
            }
        }
    }

    println!("Building hook library from {}...", hook_dir.display());
    let status = Command::new("make")
        .current_dir(&hook_dir)
        .arg("-s")
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|e| anyhow!("failed to run make in {}: {}", hook_dir.display(), e))?;

    if !status.success() {
        return Err(anyhow!("make failed in {}", hook_dir.display()));
    }

    if !so.exists() {
        return Err(anyhow!("libhook.so not found after build at {}", so.display()));
    }

    println!("Hook library built: {}", so.display());
    Ok(so)
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
/// binary and searching upward.  As a final fallback, the hook sources
/// embedded in the binary are extracted to a per-user data directory so that
/// `cargo install` users do not need a separate checkout.
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

    // Final fallback: extract embedded sources to user data directory.
    ensure_hook_data_dir()
}

/// Return the per-user hook data directory, creating it and writing the
/// embedded `hook.cpp` / `Makefile` when they are absent or stale.
///
/// Directory:
/// - Linux / other:  `$XDG_DATA_HOME/cpp2rust-demo/hook/`
///                    (default `~/.local/share/cpp2rust-demo/hook/`)
/// - macOS:          `~/Library/Application Support/cpp2rust-demo/hook/`
fn ensure_hook_data_dir() -> Result<PathBuf> {
    let base = data_dir().ok_or_else(|| anyhow!("cannot determine user data directory"))?;
    let hook_dir = base.join("cpp2rust-demo").join("hook");

    std::fs::create_dir_all(&hook_dir)
        .map_err(|e| anyhow!("create_dir_all {}: {}", hook_dir.display(), e))?;

    // Write hook.cpp if absent or content has changed (auto-upgrade on binary update).
    let cpp_path = hook_dir.join("hook.cpp");
    let needs_write = match std::fs::read_to_string(&cpp_path) {
        Ok(existing) => existing != HOOK_CPP,
        Err(_) => true,
    };
    if needs_write {
        std::fs::write(&cpp_path, HOOK_CPP)
            .map_err(|e| anyhow!("write {}: {}", cpp_path.display(), e))?;
    }

    // Write Makefile if absent or content has changed.
    let mk_path = hook_dir.join("Makefile");
    let mk_needs_write = match std::fs::read_to_string(&mk_path) {
        Ok(existing) => existing != HOOK_MAKEFILE,
        Err(_) => true,
    };
    if mk_needs_write {
        std::fs::write(&mk_path, HOOK_MAKEFILE)
            .map_err(|e| anyhow!("write {}: {}", mk_path.display(), e))?;
    }

    Ok(hook_dir)
}

/// Platform-specific base data directory (without the application sub-path).
fn data_dir() -> Option<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        dirs_home().map(|h| h.join("Library").join("Application Support"))
    }
    #[cfg(not(target_os = "macos"))]
    {
        // Respect XDG_DATA_HOME; fall back to ~/.local/share.
        if let Some(xdg) = std::env::var_os("XDG_DATA_HOME") {
            let p = PathBuf::from(xdg);
            if p.is_absolute() {
                return Some(p);
            }
        }
        dirs_home().map(|h| h.join(".local").join("share"))
    }
}

/// Returns the current user's home directory.
///
/// Uses the `HOME` environment variable, which is the standard POSIX mechanism
/// and covers Linux, macOS, and most Unix-like systems.  Windows is not a
/// supported target for this tool (it relies on LD_PRELOAD and ELF shared
/// libraries), so no Windows-specific fallback is needed.
fn dirs_home() -> Option<PathBuf> {
    // Prefer HOME env var (works in most POSIX environments).
    if let Some(h) = std::env::var_os("HOME") {
        let p = PathBuf::from(h);
        if p.is_absolute() {
            return Some(p);
        }
    }
    None
}
