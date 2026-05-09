use crate::error::Result;
use anyhow::anyhow;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// Build `hook/libhook.so`.
pub fn build_hook() -> Result<PathBuf> {
    let hook_dir = hook_dir()?;
    let so = hook_dir.join("libhook.so");

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
    Ok(so)
}

/// Execute command with LD_PRELOAD hook enabled.
pub fn run_with_hook(
    build_dir: &Path,
    cmd: &[String],
    project_root: &Path,
    feature_root: &Path,
    hook_so: &Path,
) -> Result<()> {
    if cmd.is_empty() {
        return Err(anyhow!("capture command is empty"));
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
            "capture command failed with exit code {}",
            status.code().unwrap_or(-1)
        ));
    }
    Ok(())
}

/// Load headers captured by hook: `.cpp2rust/<feature>/meta/captured_headers.list`.
pub fn load_captured_headers(feature_root: &Path) -> Result<Vec<PathBuf>> {
    let list_path = feature_root.join("meta").join("captured_headers.list");
    if !list_path.exists() {
        return Ok(Vec::new());
    }
    let content = std::fs::read_to_string(&list_path)
        .map_err(|e| anyhow!("read {}: {}", list_path.display(), e))?;
    let mut out = Vec::new();
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        out.push(PathBuf::from(line));
    }
    out.sort();
    out.dedup();
    Ok(out)
}

fn hook_dir() -> Result<PathBuf> {
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            let candidate = parent.join("hook");
            if candidate.join("Makefile").exists() {
                return Ok(candidate);
            }
            if let Some(workspace) = parent.parent().and_then(|p| p.parent()) {
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
