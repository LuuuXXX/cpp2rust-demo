#![allow(dead_code)]

pub mod nm_utils;

use cpp2rust_demo::ast_parser;
use cpp2rust_demo::extractor;
use cpp2rust_demo::generator::hicc_codegen;
use std::process::Command;

/// Run the cpp2rust-demo tool on an example directory.
/// Returns the generated FFI scaffold content (hicc blocks only).
pub fn run_tool_on(example_dir: &str) -> String {
    // 1. 找 .cpp 文件（examples/NNN_name/cpp/*.cpp）
    let cpp_dir = format!("{}/cpp", example_dir);
    let cpp_file = find_cpp_file(&cpp_dir);
    let cpp_file = match cpp_file {
        Some(p) => p,
        None => {
            eprintln!("run_tool_on: no .cpp file found in {}", cpp_dir);
            return String::new();
        }
    };

    // 2. 推导 unit_name（文件名去掉 .cpp）
    let unit_name = cpp_file
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unit")
        .to_string();

    // 3. g++ -E -C file.cpp → 临时 .cpp2rust 文件（Windows 优先尝试 clang++）
    let tmp_dir = std::env::temp_dir().join(format!("cpp2rust_{}", unit_name));
    std::fs::create_dir_all(&tmp_dir).ok();
    let preprocessed = tmp_dir.join(format!("{}.cpp2rust", unit_name));

    let preprocess_ok = run_preprocess(&cpp_file, &preprocessed);
    match preprocess_ok {
        true => {}
        false => {
            eprintln!(
                "run_tool_on: preprocessing failed for {}",
                cpp_file.display()
            );
            return String::new();
        }
    }

    // 4. 解析 AST
    let ast = match ast_parser::parse_preprocessed(&preprocessed) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("run_tool_on: AST parse failed: {}", e);
            return String::new();
        }
    };

    // 5. 读取原始 .cpp 文件的 include 信息
    let (system_includes, project_header) = extractor::read_source_includes(&cpp_file);

    // 6. 提取 FfiSpec
    let spec = extractor::extract(
        &ast,
        &unit_name,
        &system_includes,
        project_header.as_deref(),
    );

    // 7. 生成 hicc 代码
    hicc_codegen::generate(&spec)
}

fn find_cpp_file(dir: &str) -> Option<std::path::PathBuf> {
    let entries = std::fs::read_dir(dir).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("cpp") {
            return Some(path);
        }
    }
    None
}

/// 对 C++ 源文件运行预处理器（-E -C），输出到 `out`。
///
/// - Linux / macOS：优先尝试 `clang++`（macOS Xcode CLI 默认可用；Linux 也常见），
///   回退到 `g++`（Linux 默认编译器；macOS Homebrew GCC）。
/// - Windows：优先尝试 `clang++`（LLVM for Windows），
///   回退到 `g++`（MinGW/MSYS2），再回退到 `cl.exe /P /C`（MSVC）。
fn run_preprocess(src: &std::path::Path, out: &std::path::Path) -> bool {
    // 通用辅助：尝试用指定编译器执行 -E -C 预处理
    let try_cxx = |compiler: &str| -> bool {
        Command::new(compiler)
            .args(["-E", "-C", src.to_str().unwrap(), "-o", out.to_str().unwrap()])
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    };

    // clang++ 和 g++ 在所有平台上逻辑相同
    if try_cxx("clang++") {
        return true;
    }
    if try_cxx("g++") {
        return true;
    }

    // Windows 独有的 cl.exe 回退
    #[cfg(windows)]
    {
        // cl.exe 预处理输出文件固定写到当前工作目录下的 <stem>.i，需手动 rename
        let stem = src.file_stem().and_then(|s| s.to_str()).unwrap_or("out");
        let cl_out = std::env::current_dir()
            .unwrap_or_else(|_| std::path::PathBuf::from("."))
            .join(format!("{}.i", stem));
        let ok = Command::new("cl.exe")
            .args(["/P", "/C", "/nologo"])
            .arg(src)
            .status()
            .map(|s| s.success())
            .unwrap_or(false);
        if ok && cl_out.exists() {
            let _ = std::fs::rename(&cl_out, out);
            return true;
        }
    }

    false
}

/// Extract all hicc::cpp!, hicc::import_class!, hicc::import_lib! blocks from source.
pub fn extract_hicc_blocks(src: &str) -> String {
    let mut result = String::new();
    let mut depth: i32 = 0;
    let mut in_block = false;
    let mut block_buf = String::new();

    for line in src.lines() {
        let trimmed = line.trim();
        if !in_block
            && (trimmed.starts_with("hicc::cpp!")
                || trimmed.starts_with("hicc::import_class!")
                || trimmed.starts_with("hicc::import_lib!"))
        {
            in_block = true;
            depth = 0;
            block_buf.clear();
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

/// Normalize source for comparison: collapse whitespace, remove blank lines, strip // comments.
pub fn normalize(src: &str) -> String {
    src.lines()
        .map(|l| {
            // Strip // line comments (but not inside strings — good enough for golden comparison)
            let l = if let Some(pos) = l.find("//") {
                l[..pos].trim_end()
            } else {
                l
            };
            collapse_spaces(l.trim())
        })
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

/// Collapse multiple consecutive spaces into one (preserves other whitespace).
fn collapse_spaces(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut prev_space = false;
    for c in s.chars() {
        if c == ' ' {
            if !prev_space {
                result.push(c);
            }
            prev_space = true;
        } else {
            result.push(c);
            prev_space = false;
        }
    }
    result
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
///
/// 自动将同级 `../cpp/` 目录加入动态库搜索路径：
/// - Linux：`LD_LIBRARY_PATH`
/// - macOS：`DYLD_LIBRARY_PATH`
///
/// 这样开发者只需提前在 `cpp/` 目录中编译好 `.so`/`.dylib`，
/// 就能直接运行 L3 测试，无需手动设置环境变量。
pub fn cargo_run(dir: &str) -> String {
    // 从 rust_hicc 目录推导 cpp 目录：examples/NNN_name/rust_hicc -> examples/NNN_name/cpp
    let cpp_dir = std::path::Path::new(dir)
        .parent()
        .map(|p| p.join("cpp"))
        .filter(|p| p.exists());

    let mut cmd = Command::new("cargo");
    cmd.args(["run"]).current_dir(dir);

    if let Some(cpp_path) = cpp_dir {
        let cpp_abs = std::fs::canonicalize(&cpp_path).unwrap_or(cpp_path);
        let lib_path = cpp_abs.to_string_lossy().to_string();

        #[cfg(target_os = "macos")]
        {
            let existing = std::env::var("DYLD_LIBRARY_PATH").unwrap_or_default();
            let new_path = if existing.is_empty() {
                lib_path
            } else {
                format!("{}:{}", lib_path, existing)
            };
            cmd.env("DYLD_LIBRARY_PATH", new_path);
        }
        #[cfg(not(target_os = "macos"))]
        {
            let existing = std::env::var("LD_LIBRARY_PATH").unwrap_or_default();
            let new_path = if existing.is_empty() {
                lib_path
            } else {
                format!("{}:{}", lib_path, existing)
            };
            cmd.env("LD_LIBRARY_PATH", new_path);
        }
    }

    let output = cmd.output().expect("Failed to run cargo run");
    String::from_utf8_lossy(&output.stdout).to_string()
}

/// Compare actual cargo run output against expected README output.
///
/// Rules:
/// - Trailing whitespace on each line is ignored.
/// - A `0x...` token in an expected line matches any `0x[0-9a-fA-F]+` in the actual line.
pub fn compare_run_output(actual: &str, expected: &str) -> bool {
    let actual_lines: Vec<&str> = actual.lines().collect();
    let expected_lines: Vec<&str> = expected.lines().collect();
    if actual_lines.len() != expected_lines.len() {
        return false;
    }
    for (a, e) in actual_lines.iter().zip(expected_lines.iter()) {
        let a = a.trim_end_matches(|c: char| c.is_whitespace() || c == '\0');
        let e = e.trim_end_matches(|c: char| c.is_whitespace() || c == '\0');
        if e.contains("0x...") {
            let normalized = normalize_hex_addresses(a);
            if normalized != e {
                return false;
            }
        } else if a != e {
            return false;
        }
    }
    true
}

/// Replace all `0x[0-9a-fA-F]+` occurrences in `s` with `0x...`.
fn normalize_hex_addresses(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if i + 1 < bytes.len() && bytes[i] == b'0' && bytes[i + 1] == b'x' {
            let start = i;
            i += 2;
            while i < bytes.len() && (bytes[i].is_ascii_hexdigit()) {
                i += 1;
            }
            if i > start + 2 {
                result.push_str("0x...");
            } else {
                result.push_str(&s[start..i]);
            }
        } else {
            result.push(bytes[i] as char);
            i += 1;
        }
    }
    result
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
