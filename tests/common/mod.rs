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

    // 7. 生成 hicc 代码，提取纯 hicc 块（去除文件级前缀如 use crate::*;）
    let raw = hicc_codegen::generate(&spec);
    extract_hicc_blocks(&raw)
}

/// 对一段 C++ 源码运行「预处理 → 解析 → 提取 → 生成」流水线，返回完整的生成代码
/// （含文件级前缀）。供模板生成测试使用，可自定义临时文件名以隔离并发测试。
///
/// 返回 `None` 表示当前环境缺少 C++ 预处理器或 libclang，调用方应据此跳过断言。
pub fn generate_from_source(unit_name: &str, cpp_source: &str) -> Option<String> {
    let tmp_dir = std::env::temp_dir().join(format!("cpp2rust_src_{}", unit_name));
    std::fs::create_dir_all(&tmp_dir).ok()?;
    let cpp_file = tmp_dir.join(format!("{}.cpp", unit_name));
    std::fs::write(&cpp_file, cpp_source).ok()?;
    let preprocessed = tmp_dir.join(format!("{}.cpp2rust", unit_name));
    if !run_preprocess(&cpp_file, &preprocessed) {
        return None;
    }
    let ast = ast_parser::parse_preprocessed(&preprocessed).ok()?;
    let (system_includes, project_header) = extractor::read_source_includes(&cpp_file);
    let spec = extractor::extract(&ast, unit_name, &system_includes, project_header.as_deref());
    Some(hicc_codegen::generate(&spec))
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
            .args([
                "-E",
                "-C",
                src.to_str().unwrap(),
                "-o",
                out.to_str().unwrap(),
            ])
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
    let blocks = cpp2rust_demo::merger::block_parser::extract_block_texts(src);
    let mut result = String::new();
    for block in blocks {
        result.push_str(&block);
        result.push_str("\n\n");
    }
    // 去掉末尾多余换行
    result.trim_end().to_string() + "\n"
}

/// Normalize source for comparison: collapse whitespace, remove blank lines, strip // comments.
/// Exception: lines containing `cpp2rust-todo` are preserved as-is (they carry degradation markers).
pub fn normalize(src: &str) -> String {
    src.lines()
        .map(|l| {
            // cpp2rust-todo 降级标记注释行不参与注释剥除，保留完整内容（含标记）
            if l.contains("cpp2rust-todo") {
                return collapse_spaces(l.trim());
            }
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
/// 当环境变量 `CARGO_TARGET_DIR` 已设置且对应的预编译二进制存在时，
/// 直接执行该二进制以跳过 cargo 的调度开销（用于 CI 预构建场景）。
/// 否则回退到 `cargo run`。
pub fn cargo_run(dir: &str) -> String {
    // 尝试直接运行预构建的二进制（跳过 cargo 开销）
    if let Ok(target_dir) = std::env::var("CARGO_TARGET_DIR") {
        let manifest = format!("{}/Cargo.toml", dir);
        if let Some(bin_name) = parse_cargo_package_name(&manifest) {
            let bin_path = if cfg!(windows) {
                format!("{}/debug/{}.exe", target_dir, bin_name)
            } else {
                format!("{}/debug/{}", target_dir, bin_name)
            };
            if std::path::Path::new(&bin_path).exists() {
                let output = Command::new(&bin_path)
                    .current_dir(dir)
                    .output()
                    .unwrap_or_else(|e| panic!("Failed to run binary {}: {}", bin_path, e));
                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    panic!(
                        "Binary {} exited with {:?}\nstderr:\n{}",
                        bin_path,
                        output.status.code(),
                        stderr
                    );
                }
                return String::from_utf8_lossy(&output.stdout).to_string();
            }
        }
    }
    // 回退：通过 cargo run 编译并运行
    let output = Command::new("cargo")
        .args(["run"])
        .current_dir(dir)
        .output()
        .expect("Failed to run cargo run");
    String::from_utf8_lossy(&output.stdout).to_string()
}

/// 从 Cargo.toml 文件中解析 `[package]` section 的 `name` 字段。
///
/// 支持带空格（`name = "foo"`）和不带空格（`name="foo"`）两种格式。
fn parse_cargo_package_name(manifest_path: &str) -> Option<String> {
    let content = std::fs::read_to_string(manifest_path).ok()?;
    let mut in_package = false;
    for line in content.lines() {
        let trimmed = line.trim();
        // 跟踪当前 TOML section
        if trimmed.starts_with('[') {
            in_package = trimmed == "[package]";
            continue;
        }
        if !in_package {
            continue;
        }
        // 匹配 `name` 键（避免匹配 `name_blah` 等键名）
        let after_name = match trimmed.strip_prefix("name") {
            Some(s) => s,
            None => continue,
        };
        if after_name.starts_with(|c: char| c.is_alphanumeric() || c == '_') {
            continue; // 不是精确的 name 键
        }
        let after_eq = match after_name.trim_start().strip_prefix('=') {
            Some(s) => s,
            None => continue,
        };
        let value = after_eq.trim();
        // 提取双引号包裹的值
        if value.starts_with('"') && value.ends_with('"') && value.len() >= 2 {
            return Some(value[1..value.len() - 1].to_string());
        }
    }
    None
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

// ─────────────────────────────────────────────────────────────────
//  E2E 测试共享辅助函数
// ─────────────────────────────────────────────────────────────────

/// 使用 C++ 编译器预处理源文件，写出到 `out_dir/<unit_name>.cpp2rust`。
///
/// 编译器选择顺序：`CXX` 环境变量 → `g++` → `clang++`。
/// `include_dirs` 为额外 `-I` 搜索路径列表（相对路径以仓库根目录为基准）。
/// 成功返回输出文件路径，预处理失败返回 `None`。
pub fn preprocess_cpp(
    src: &std::path::Path,
    include_dirs: &[&str],
    out_dir: &std::path::Path,
    unit_name: &str,
) -> Option<std::path::PathBuf> {
    let out = out_dir.join(format!("{}.cpp2rust", unit_name));

    let try_cxx = |compiler: &str| -> bool {
        let mut cmd = Command::new(compiler);
        cmd.args(["-E", "-C", "-w"]);
        for inc in include_dirs {
            cmd.arg(format!("-I{}", inc));
        }
        cmd.arg(src).arg("-o").arg(&out);
        cmd.status().map(|s| s.success()).unwrap_or(false)
    };

    // 优先使用 CXX 环境变量指定的编译器，否则依次尝试 g++ 和 clang++
    let cxx_env = std::env::var("CXX").unwrap_or_default();
    let candidates: Vec<&str> = if !cxx_env.is_empty() {
        vec![cxx_env.as_str()]
    } else {
        vec!["g++", "clang++"]
    };

    for compiler in &candidates {
        if try_cxx(compiler) {
            return Some(out);
        }
    }
    None
}

/// 验证生成的 hicc 代码符合三段式格式约束：
///
/// 1. 必须包含 `hicc::cpp! {` 块
/// 2. 输出文件以 `}` 结束（最后一个宏块正确关闭）
/// 3. 每个 import_class!/import_lib! 块内部括号平衡
/// 4. 若存在 `hicc::import_class!` 块，每个类必须有 `#[cpp(class` 或 `#[interface]`
/// 5. 若存在 `hicc::import_lib!` 块，必须包含 `#![link_name = "`
/// 6. 类方法绑定在 import_class! 块内必须有 `#[cpp(method = "`
/// 7. 函数绑定在 import_lib! 块内必须有 `#[cpp(func = "`
pub fn assert_valid_hicc_format(code: &str, unit_name: &str) {
    assert!(
        code.contains("hicc::cpp! {"),
        "unit '{}': 缺少 hicc::cpp! 块\n首 400 字符:\n{}",
        unit_name,
        &code[..code.len().min(400)]
    );

    assert!(
        code.trim_end().ends_with('}'),
        "unit '{}': 输出文件未以 }} 结束（宏块可能未正确关闭）",
        unit_name
    );

    // 从源码中提取每个宏块的文本内容，用于块级精确检查（避免跨块误判）
    let extract_blocks = |macro_prefix: &str| -> Vec<String> {
        let mut blocks = Vec::new();
        let mut search = 0usize;
        while let Some(rel) = code[search..].find(macro_prefix) {
            let block_start = search + rel + macro_prefix.len();
            let mut depth = 1i32;
            let mut in_str = false;
            let mut esc = false;
            let mut closed = false;
            // `end` 在 closed == true 时被设为块末尾；若循环结束仍为 false，
            // 下方 assert!(closed, ...) 会捕获该错误，此时 end 值不再被使用。
            let mut end = block_start;
            for (i, c) in code[block_start..].char_indices() {
                if esc {
                    esc = false;
                    continue;
                }
                match c {
                    '\\' if in_str => esc = true,
                    '"' => in_str = !in_str,
                    '{' if !in_str => depth += 1,
                    '}' if !in_str => {
                        depth -= 1;
                        if depth == 0 {
                            end = block_start + i + 1;
                            closed = true;
                            break;
                        }
                    }
                    _ => {}
                }
            }
            assert!(
                closed,
                "unit '{}': {} 块未正确关闭（括号不平衡）",
                unit_name, macro_prefix
            );
            blocks.push(code[block_start..end].to_string());
            search = block_start;
        }
        blocks
    };

    let class_blocks = extract_blocks("hicc::import_class! {");
    let lib_blocks = extract_blocks("hicc::import_lib! {");

    for block in &class_blocks {
        assert!(
            block.contains("#[cpp(class") || block.contains("#[interface]"),
            "unit '{}': import_class! 块缺少类注解 (#[cpp(class...)] 或 #[interface])",
            unit_name
        );
        // 仅在当前 import_class! 块内检查方法绑定
        if block.contains("fn ") {
            assert!(
                block.contains("#[cpp(method = \""),
                "unit '{}': import_class! 块包含方法但缺少 #[cpp(method = \"...\")]",
                unit_name
            );
        }
    }

    for block in &lib_blocks {
        assert!(
            block.contains("#![link_name = \""),
            "unit '{}': import_lib! 块缺少 #![link_name = \"...\"]",
            unit_name
        );
        // 仅在当前 import_lib! 块内检查函数绑定
        if block.contains("fn ") {
            assert!(
                block.contains("#[cpp(func = \""),
                "unit '{}': import_lib! 块包含函数但缺少 #[cpp(func = \"...\")]",
                unit_name
            );
        }
    }
}

/// 对单个 C++ 源文件执行完整 init 流程（预处理 → AST → 提取 → 生成），返回 hicc 代码。
///
/// 失败时返回 `None`（预处理失败、文件不存在等）。
pub fn process_cpp_source(
    src: &std::path::Path,
    include_dirs: &[&str],
    preprocess_dir: &std::path::Path,
) -> Option<(String, String)> {
    if !src.exists() {
        return None;
    }
    let unit_name = src
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unit")
        .to_string();

    let preprocessed = preprocess_cpp(src, include_dirs, preprocess_dir, &unit_name)?;
    let ast = ast_parser::parse_preprocessed(&preprocessed).ok()?;
    let (sys_includes, proj_header) = extractor::read_source_includes(src);
    let spec = extractor::extract(&ast, &unit_name, &sys_includes, proj_header.as_deref());
    let code = hicc_codegen::generate(&spec);
    Some((unit_name, code))
}

// ─────────────────────────────────────────────────────────────────
//  L3 运行测试辅助函数
// ─────────────────────────────────────────────────────────────────

/// 确保指定 example 的 C++ 动态库已经编译好。
///
/// 库文件若已存在则直接返回（增量编译）；不存在时自动调用编译器编译 `.cpp` 文件。
/// 若编译失败，会 `panic!` 并给出错误信息，测试会立即失败（比静默跳过更易发现问题）。
pub fn ensure_cpp_lib(example: &str) {
    let cpp_dir = format!("examples/{}/cpp", example);
    // 去掉形如 "013_" 的数字前缀，得到库的短名称
    let short_name = example.splitn(2, '_').nth(1).unwrap_or(example);

    let lib_name = if cfg!(target_os = "macos") {
        format!("lib{}.dylib", short_name)
    } else if cfg!(windows) {
        format!("{}.dll", short_name)
    } else {
        format!("lib{}.so", short_name)
    };

    let lib_path = format!("{}/{}", cpp_dir, lib_name);
    if std::path::Path::new(&lib_path).exists() {
        return; // 已存在，快速路径
    }

    // 收集目录下所有 .cpp 文件
    let cpp_files: Vec<_> = std::fs::read_dir(&cpp_dir)
        .unwrap_or_else(|e| panic!("无法读取目录 {}: {}", cpp_dir, e))
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|x| x.to_str()) == Some("cpp"))
        .collect();

    if cpp_files.is_empty() {
        panic!("ensure_cpp_lib: {} 中没有找到 .cpp 文件", cpp_dir);
    }

    let (compiler, shared_flag): (&str, &[&str]) = if cfg!(target_os = "macos") {
        // 使用系统 Apple Clang，避免 KyleMayes/install-llvm-action 安装的 LLVM clang++
        // 覆盖 PATH 后使用 LLVM 自带 libc++ 头文件导致与 macOS SDK 不兼容的问题
        ("/usr/bin/clang++", &["-dynamiclib"])
    } else if cfg!(windows) {
        ("g++", &["-shared"])
    } else {
        ("g++", &["-shared", "-fPIC"])
    };

    let mut cmd = Command::new(compiler);
    cmd.args(shared_flag);
    for f in &cpp_files {
        cmd.arg(f);
    }
    cmd.arg("-o").arg(&lib_path);

    let status = cmd
        .status()
        .unwrap_or_else(|e| panic!("ensure_cpp_lib: 启动编译器 {} 失败: {}", compiler, e));

    if !status.success() {
        panic!(
            "ensure_cpp_lib: 编译 {} 失败（退出码: {:?}）",
            example,
            status.code()
        );
    }
}

/// 断言生成代码中包含指定 TAG 的降级标记注释（`cpp2rust-todo[TAG]`）。
///
/// 用于在 L1 golden 测试中直接验证降级标记是否被正确生成，
/// 而不依赖 normalize 的注释剥除行为来隐含地放过这类差异。
pub fn assert_contains_todo_tag(code: &str, tag: &str, unit_name: &str) {
    let marker = format!("cpp2rust-todo[{}]", tag);
    assert!(
        code.contains(&marker),
        "unit '{}': 期望生成降级标记 {} 但未找到",
        unit_name,
        marker
    );
}

// ─────────────────────────────────────────────────────────────────
//  E2E merge_phase 共用辅助函数
// ─────────────────────────────────────────────────────────────────

/// E2E merge 阶段共用逻辑：执行 init → merge → 验证目录结构 → cargo check → cargo test。
///
/// 将 tinyxml2 / pugixml / fmtlib 等 E2E 测试中高度重复的 merge_phase 代码统一到此处。
///
/// ## 参数
/// - `project_name`: Cargo.toml 中的 package name（如 "tinyxml2"）
/// - `project_root`: C++ 项目根目录路径（如 "references/tinyxml2"）
/// - `sources`: 要处理的 C++ 源文件相对路径列表（如 `&["tinyxml2.cpp"]`）
/// - `includes`: 传递给 `process_cpp_source` 的 include 目录列表
pub fn run_merge_phase_e2e(
    project_name: &str,
    project_root: &str,
    sources: &[&str],
    includes: &[&str],
) {
    use cpp2rust_demo::{generator::project_generator, merger};
    use tempfile::TempDir;

    let tmp = TempDir::new().unwrap();
    let preprocess_dir = tmp.path().join("c");
    let rust_dir = tmp.path().join("rust");
    std::fs::create_dir_all(&preprocess_dir).unwrap();

    let mut unit_paths: Vec<String> = Vec::new();

    // ── Init 阶段：生成所有 unit .rs 文件 ──────────────────────────
    for src_rel in sources {
        let src_path = std::path::Path::new(project_root).join(src_rel);
        if let Some((unit_name, code)) =
            process_cpp_source(&src_path, includes, &preprocess_dir)
        {
            project_generator::write_unit_rs(&rust_dir, &unit_name, &code)
                .expect("write_unit_rs 失败");
            unit_paths.push(unit_name);
        }
    }

    assert!(
        !unit_paths.is_empty(),
        "{}_merge_phase: init 阶段未生成任何 unit 文件（g++ / clang++ 是否已安装？）",
        project_name
    );

    // 生成 Cargo.toml 与 lib.rs（merge 前必须存在 src/）
    project_generator::write_cargo_toml(&rust_dir, project_name)
        .expect("write_cargo_toml 失败");
    project_generator::write_lib_rs(&rust_dir, &unit_paths).expect("write_lib_rs 失败");

    // ── Merge 阶段 ─────────────────────────────────────────────────
    merger::merge_in_place(&rust_dir).expect("merge_in_place 失败");

    // ── 验证输出目录结构 ────────────────────────────────────────────
    let src1 = rust_dir.join("src.1");
    let src_dir = rust_dir.join("src");

    assert!(
        src1.is_dir(),
        "{}_merge_phase: src.1/ 目录不存在",
        project_name
    );
    assert!(
        src_dir.is_dir() && !src_dir.is_symlink(),
        "{}_merge_phase: src/ 不存在或为符号链接",
        project_name
    );
    assert!(
        !rust_dir.join("src.2").exists(),
        "{}_merge_phase: src.2 应已被 rename 为 src",
        project_name
    );

    // ── 验证合并后文件 hicc 格式 ─────────────────────────────────
    let merged_files = merger::collect_unit_rs_files(&src_dir);
    assert!(
        !merged_files.is_empty(),
        "{}_merge_phase: src/ 下未找到任何 .rs 文件",
        project_name
    );
    for rs_path in &merged_files {
        let fname = rs_path.file_name().and_then(|f| f.to_str()).unwrap_or("");
        if fname == "lib.rs" || fname == "mod.rs" {
            continue;
        }
        let content = std::fs::read_to_string(rs_path).expect("读取合并后 .rs 文件失败");
        assert_valid_hicc_format(&content, rs_path.to_str().unwrap_or("?"));
    }

    // ── cargo check：验证生成的 Rust 项目可编译 ────────────────────
    match std::process::Command::new("cargo")
        .args(["check", "--quiet"])
        .current_dir(&rust_dir)
        .output()
    {
        Ok(output) if !output.status.success() => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            panic!(
                "{}_merge_phase: cargo check 失败（生成的 Rust 项目不可编译）:\n{}",
                project_name, stderr
            );
        }
        Ok(_) => println!(
            "{}_merge_phase: cargo check 通过 ({} 个 unit)",
            project_name,
            unit_paths.len()
        ),
        Err(e) => eprintln!(
            "{}_merge_phase: cargo check 跳过（cargo 不可用: {}）",
            project_name, e
        ),
    }

    // ── cargo test：验证生成的冒烟测试可通过 ───────────────────────
    let smoke_test_path = rust_dir.join("tests/smoke.rs");
    if smoke_test_path.exists() {
        match std::process::Command::new("cargo")
            .args(["test", "--quiet"])
            .current_dir(&rust_dir)
            .output()
        {
            Ok(output) if !output.status.success() => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                panic!(
                    "{}_merge_phase: cargo test 失败（生成的冒烟测试未通过）:\n{}",
                    project_name, stderr
                );
            }
            Ok(_) => println!(
                "{}_merge_phase: cargo test 通过（生成的冒烟测试全部通过）",
                project_name
            ),
            Err(e) => eprintln!(
                "{}_merge_phase: cargo test 跳过（cargo 不可用: {}）",
                project_name, e
            ),
        }
    } else {
        println!(
            "{}_merge_phase: cargo test 跳过（未生成 tests/smoke.rs）",
            project_name
        );
    }
}
