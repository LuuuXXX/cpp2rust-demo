use anyhow::{bail, Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

/// LD_PRELOAD 捕获执行逻辑
/// 使用 hook 库拦截编译器调用，输出预处理文件

/// 运行捕获：在目标 C++ 项目中运行 make，通过 LD_PRELOAD 拦截
pub fn run_capture(
    project_dir: &Path,
    hook_lib: &Path,
    feature_root: &Path,
    cxx: Option<&str>,
    debug: bool,
) -> Result<Vec<PathBuf>> {
    if !hook_lib.exists() {
        bail!("Hook library not found: {}", hook_lib.display());
    }

    let feature_root_str = feature_root.to_string_lossy();
    let project_root_str = project_dir.to_string_lossy();
    let hook_lib_str = hook_lib.to_string_lossy();

    let mut cmd = Command::new("make");
    cmd.current_dir(project_dir)
        .env("LD_PRELOAD", hook_lib_str.as_ref())
        .env("C2RUST_FEATURE_ROOT", feature_root_str.as_ref())
        .env("C2RUST_PROJECT_ROOT", project_root_str.as_ref());

    if let Some(cxx) = cxx {
        cmd.env("C2RUST_CXX", cxx);
    }

    if debug {
        cmd.env("C2RUST_DEBUG", "1");
    }

    let status = cmd.status().context("Failed to run make")?;
    if !status.success() {
        bail!("make failed with status: {}", status);
    }

    // 收集生成的 .c2rust 文件
    collect_c2rust_files(feature_root)
}

/// 收集 feature_root 下所有 .c2rust 文件
pub fn collect_c2rust_files(feature_root: &Path) -> Result<Vec<PathBuf>> {
    let c_dir = feature_root.join("c");
    let mut files = Vec::new();

    if !c_dir.exists() {
        return Ok(files);
    }

    collect_recursive(&c_dir, &mut files)?;
    files.sort();
    Ok(files)
}

fn collect_recursive(dir: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_recursive(&path, files)?;
        } else if path.extension().map_or(false, |e| e == "c2rust") {
            files.push(path);
        }
    }
    Ok(())
}
