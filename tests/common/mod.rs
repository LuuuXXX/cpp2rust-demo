#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

pub fn examples_dir() -> PathBuf {
    workspace_root().join("examples")
}

pub fn rust_hicc_dir(example: &str) -> PathBuf {
    examples_dir().join(example).join("rust_hicc")
}

pub fn golden_file(example: &str) -> PathBuf {
    rust_hicc_dir(example).join("src").join("main.rs")
}

pub fn cargo_build(dir: &Path) -> Result<(), String> {
    let output = Command::new("cargo")
        .args(["build"])
        .current_dir(dir)
        .output()
        .map_err(|e| format!("Failed to spawn cargo: {e}"))?;

    if output.status.success() {
        Ok(())
    } else {
        Err(format!(
            "cargo build failed in {}:\nstdout: {}\nstderr: {}",
            dir.display(),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}

pub fn cargo_run(dir: &Path) -> Result<String, String> {
    let output = Command::new("cargo")
        .args(["run"])
        .current_dir(dir)
        .output()
        .map_err(|e| format!("Failed to spawn cargo: {e}"))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    } else {
        Err(format!(
            "cargo run failed in {}:\nstdout: {}\nstderr: {}",
            dir.display(),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}

pub fn read_normalized(path: &Path) -> String {
    let content = fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("Failed to read {}: {}", path.display(), e));
    normalize_content(&content)
}

pub fn normalize_content(s: &str) -> String {
    s.lines()
        .map(str::trim_end)
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn parse_readme_run_result(readme_path: &Path) -> Option<String> {
    let content = fs::read_to_string(readme_path).ok()?;
    let mut in_result = false;
    let mut in_code_block = false;
    let mut result_lines = Vec::new();

    for line in content.lines() {
        if line.trim() == "## 运行结果" {
            in_result = true;
            continue;
        }
        if in_result {
            if line.starts_with("## ") && line.trim() != "## 运行结果" {
                break;
            }
            if line.trim() == "```" {
                if in_code_block {
                    break;
                }
                in_code_block = true;
                continue;
            }
            if in_code_block {
                result_lines.push(line);
            }
        }
    }

    if result_lines.is_empty() {
        None
    } else {
        Some(result_lines.join("\n"))
    }
}

pub fn run_tool_on_example(example: &str) -> Result<String, String> {
    let example_dir = examples_dir().join(example);
    let tool = workspace_root()
        .join("target")
        .join("debug")
        .join("cpp2rust-ffi");

    if !tool.exists() {
        return Err(format!("Tool binary not found: {}", tool.display()));
    }

    let output_dir = workspace_root()
        .join("target")
        .join("test-tool-output")
        .join(example);
    if output_dir.exists() {
        fs::remove_dir_all(&output_dir)
            .map_err(|e| format!("Failed to clean output dir {}: {e}", output_dir.display()))?;
    }

    let output = Command::new(&tool)
        .args([
            "init",
            "-i",
            &example_dir.join(".c2rust/v5").to_string_lossy(),
            "-o",
            &output_dir.to_string_lossy(),
        ])
        .output()
        .map_err(|e| format!("Failed to spawn tool: {e}"))?;

    if !output.status.success() {
        return Err(format!(
            "Tool failed:\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let generated = fs::read_to_string(output_dir.join("src").join("main.rs"))
        .map_err(|e| format!("Failed to read generated output: {e}"))?;
    Ok(generated)
}
