use crate::error::{DemoError, Result};
use crate::layout::FeatureLayout;
use std::path::PathBuf;
use std::process::Command;

fn hook_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("hook")
}

fn hook_library() -> PathBuf {
    hook_dir().join("libcpp2rust_hook.so")
}

pub fn build_hook(_layout: &FeatureLayout) -> Result<()> {
    let status = Command::new("make")
        .arg("-C")
        .arg(hook_dir())
        .status()?;
    if !status.success() {
        return Err(DemoError::CommandFailed {
            program: "make -C hook".into(),
            code: status.code(),
        });
    }
    Ok(())
}

pub fn run_capture(layout: &FeatureLayout, build_command: &[String]) -> Result<()> {
    if build_command.is_empty() {
        return Err(DemoError::MissingArgument("build command"));
    }

    let mut command = Command::new(&build_command[0]);
    command.args(&build_command[1..]);
    command.current_dir(&layout.project_root);
    command.env("C2RUST_PROJECT_ROOT", &layout.project_root);
    command.env("C2RUST_FEATURE_ROOT", &layout.feature_root);
    command.env("LD_PRELOAD", hook_library());
    command.env_remove("C2RUST_AST_HELPER");

    let status = command.status()?;
    if !status.success() {
        return Err(DemoError::CommandFailed {
            program: build_command.join(" "),
            code: status.code(),
        });
    }
    Ok(())
}

pub fn read_compiler_options(path: &std::path::Path) -> Result<Vec<String>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let content = std::fs::read_to_string(path)?;
    Ok(content
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(ToOwned::to_owned)
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_dir(name: &str) -> PathBuf {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("target/test-work/capture")
            .join(format!("{name}-{}", std::process::id()));
        if root.exists() {
            std::fs::remove_dir_all(&root).unwrap();
        }
        std::fs::create_dir_all(&root).unwrap();
        root
    }

    #[test]
    fn rejects_empty_build_command() {
        let dir = test_dir("empty");
        let layout = FeatureLayout::new(&dir, "demo");
        let err = run_capture(&layout, &[]).unwrap_err();
        assert!(matches!(err, DemoError::MissingArgument(_)));
    }

    #[test]
    fn reads_compiler_options() {
        let dir = test_dir("opts");
        let file = dir.join("main.opts");
        std::fs::write(&file, "-std=c++17\n-Iinclude\n\n").unwrap();
        let opts = read_compiler_options(&file).unwrap();
        assert_eq!(opts, vec!["-std=c++17", "-Iinclude"]);
    }

    #[test]
    fn missing_options_file_is_empty() {
        let dir = test_dir("missing");
        let file = dir.join("missing.opts");
        let opts = read_compiler_options(&file).unwrap();
        assert!(opts.is_empty());
    }
}
