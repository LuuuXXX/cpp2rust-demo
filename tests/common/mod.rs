use std::path::Path;
use std::process::Command;

/// Run the cpp2rust-demo tool on an example directory.
/// Returns the generated FFI scaffold content (lib.rs).
/// Currently returns empty string until Phase 1+ is implemented.
pub fn run_tool_on(example_dir: &str) -> String {
    let _ = Path::new(example_dir);
    // TODO: Phase 1+ - actually run the tool
    String::new()
}

/// Extract all hicc::cpp!, hicc::import_class!, hicc::import_lib! blocks from source.
pub fn extract_hicc_blocks(src: &str) -> String {
    let mut result = String::new();
    let mut depth: i32 = 0;
    let mut in_block = false;
    let mut block_buf = String::new();

    for line in src.lines() {
        let trimmed = line.trim();
        if !in_block {
            if trimmed.starts_with("hicc::cpp!")
                || trimmed.starts_with("hicc::import_class!")
                || trimmed.starts_with("hicc::import_lib!")
            {
                in_block = true;
                depth = 0;
                block_buf.clear();
            }
        }
        if in_block {
            block_buf.push_str(line);
            block_buf.push('\n');
            for ch in line.chars() {
                match ch {
                    '{' => depth += 1,
                    '}' => {
                        depth -= 1;
                        if depth == 0 {
                            result.push_str(&block_buf);
                            result.push('\n');
                            in_block = false;
                            break;
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    result
}

/// Normalize source for comparison: collapse whitespace, remove blank lines.
pub fn normalize(src: &str) -> String {
    src.lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

/// Read the golden file content from an example's rust_hicc/src/main.rs
pub fn read_golden(example_dir: &str, relative: &str) -> String {
    let path = format!("{}/{}", example_dir, relative);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("Failed to read golden file {}: {}", path, e))
}

/// Run cargo build in a directory. Returns true on success.
pub fn cargo_build(dir: &str) -> bool {
    Command::new("cargo")
        .args(["build"])
        .current_dir(dir)
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Run cargo run in a directory. Returns stdout output.
pub fn cargo_run(dir: &str) -> String {
    let output = Command::new("cargo")
        .args(["run"])
        .current_dir(dir)
        .output()
        .expect("Failed to run cargo run");
    String::from_utf8_lossy(&output.stdout).to_string()
}

/// Parse README.md and extract the "运行结果" section's code block content.
pub fn parse_readme_run_result(readme_path: &str) -> String {
    let content = std::fs::read_to_string(readme_path)
        .unwrap_or_else(|e| panic!("Failed to read README: {} - {}", readme_path, e));

    let mut in_section = false;
    let mut in_code = false;
    let mut result = String::new();

    for line in content.lines() {
        if line.trim_start_matches('#').trim() == "运行结果" {
            in_section = true;
            continue;
        }
        if in_section {
            if line.starts_with("```") && !in_code {
                in_code = true;
                continue;
            }
            if in_code {
                if line.starts_with("```") {
                    break;
                }
                result.push_str(line);
                result.push('\n');
            }
            if line.starts_with('#') && !line.starts_with("##  ") {
                break;
            }
        }
    }
    result
}
