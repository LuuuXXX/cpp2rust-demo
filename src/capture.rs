use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{bail, Context, Result};
use walkdir::WalkDir;

#[allow(dead_code)]
pub fn capture_cpp_project(
    project_dir: &Path,
    feature_root: &Path,
    hook_lib: &Path,
) -> Result<Vec<PathBuf>> {
    let status = Command::new("make")
        .arg("-j4")
        .current_dir(project_dir)
        .env("LD_PRELOAD", hook_lib)
        .env("C2RUST_FEATURE_ROOT", feature_root)
        .env("C2RUST_PROJECT_ROOT", project_dir)
        .status()
        .with_context(|| format!("running make in {}", project_dir.display()))?;

    if !status.success() {
        bail!("make failed in {}", project_dir.display());
    }

    let capture_root = feature_root.join("c");
    let mut files = WalkDir::new(&capture_root)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.into_path())
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("c2rust"))
        .collect::<Vec<_>>();
    files.sort();
    Ok(files)
}
