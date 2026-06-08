//! L4-Extended: {fmt} 库端到端集成测试
//!
//! 针对系统安装的 {fmt} 库（libfmt-dev）执行 init 阶段转换，
//! 验证工具能从 `extern "C"` shim 文件生成正确的 hicc 三段式 FFI 绑定。
//!
//! 运行前提：
//! - 已安装 libfmt-dev：`sudo apt-get install libfmt-dev`
//!
//! 运行方式：
//! ```bash
//! cargo test --test fmt_e2e_test -- --ignored
//! ```
//!
//! 验证要点：
//! 1. shim 文件能成功预处理（g++ -E -C）
//! 2. 工具从 extern "C" 函数生成 hicc::import_lib! 块
//! 3. import_lib! 包含正确的函数绑定（#[cpp(func = "...")]）
//! 4. link_name 不含路径分隔符

use cpp2rust_demo::{ast_parser, extractor, generator::hicc_codegen};
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;

const FMT_SHIM_DIR: &str = "references/fmt_sys/shim";

/// shim 源文件列表
const FMT_SHIM_SOURCES: &[&str] = &["format_ffi.cpp"];

// ─────────────────────────────────────────────────────────────────
//  辅助函数
// ─────────────────────────────────────────────────────────────────

/// 检测 fmt 系统头文件是否可用。
fn fmt_headers_available() -> bool {
    // 尝试用 pkg-config 或直接检测头文件
    if Path::new("/usr/include/fmt/core.h").exists() {
        return true;
    }
    // pkg-config 方式
    Command::new("pkg-config")
        .args(["--exists", "fmt"])
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// 获取 fmt 系统头文件 include 路径。
fn fmt_include_dirs() -> Vec<String> {
    // 尝试 pkg-config
    if let Ok(output) = Command::new("pkg-config").args(["--cflags-only-I", "fmt"]).output() {
        if output.status.success() {
            let flags = String::from_utf8_lossy(&output.stdout);
            return flags
                .split_whitespace()
                .filter_map(|f| f.strip_prefix("-I"))
                .map(|s| s.to_string())
                .collect();
        }
    }
    // 回退到标准路径
    vec!["/usr/include".to_string()]
}

/// 使用 g++ -E -C 预处理 C++ 文件。
fn preprocess(
    src: &Path,
    include_dirs: &[String],
    out_dir: &Path,
    unit_name: &str,
) -> Option<PathBuf> {
    let out = out_dir.join(format!("{}.cpp2rust", unit_name));
    let mut cmd = Command::new("g++");
    cmd.args(["-E", "-C", "-w", "-std=c++17"]);
    for inc in include_dirs {
        cmd.arg(format!("-I{}", inc));
    }
    cmd.arg(src).arg("-o").arg(&out);
    let ok = cmd.status().map(|s| s.success()).unwrap_or(false);
    if ok { Some(out) } else { None }
}

/// 处理单个 shim 文件：预处理 → AST → 提取 → 生成。
fn process_shim(
    src_name: &str,
    shim_dir: &Path,
    include_dirs: &[String],
    preprocess_dir: &Path,
) -> Option<(String, String)> {
    let src_path = shim_dir.join(src_name);
    if !src_path.exists() {
        return None;
    }
    let unit_name = src_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unit")
        .to_string();

    let preprocessed = preprocess(&src_path, include_dirs, preprocess_dir, &unit_name)?;
    let ast = ast_parser::parse_preprocessed(&preprocessed).ok()?;
    let (sys_includes, proj_header) = extractor::read_source_includes(&src_path);
    let spec = extractor::extract(&ast, &unit_name, &sys_includes, proj_header.as_deref());
    let code = hicc_codegen::generate(&spec);
    Some((unit_name, code))
}

// ─────────────────────────────────────────────────────────────────
//  L4-Extended-Fmt 测试
// ─────────────────────────────────────────────────────────────────

/// L4-Extended-Fmt：对 fmt_sys/shim/ 中的 extern-C 包装文件执行 init 阶段转换。
///
/// 验证工具能从 {fmt} 的 extern-C shim 生成正确的 hicc::import_lib! FFI 绑定。
/// 测试需要系统安装 libfmt-dev；使用 `cargo test -- --ignored` 显式运行。
#[test]
#[ignore = "需要 libfmt-dev；运行方式：cargo test --test fmt_e2e_test -- --ignored"]
fn fmt_shim_ffi_generates_importlib() {
    if !fmt_headers_available() {
        eprintln!("⚠ 跳过 fmt E2E 测试：未找到 fmt 系统头文件（请安装 libfmt-dev）");
        return;
    }

    let tmp = TempDir::new().unwrap();
    let preprocess_dir = tmp.path().join("c");
    std::fs::create_dir_all(&preprocess_dir).unwrap();

    let shim_dir = PathBuf::from(FMT_SHIM_DIR);
    let include_dirs = fmt_include_dirs();

    assert!(
        shim_dir.exists(),
        "shim 目录不存在：{}\n  请确认 references/fmt_sys/shim/ 目录已就绪",
        shim_dir.display()
    );

    let mut processed = 0usize;
    let mut failed: Vec<String> = Vec::new();
    let mut skipped: Vec<&str> = Vec::new();

    for src_name in FMT_SHIM_SOURCES {
        match process_shim(src_name, &shim_dir, &include_dirs, &preprocess_dir) {
            Some((unit_name, code)) => {
                // 验证 1：必须包含 hicc::cpp! 块
                if !code.contains("hicc::cpp! {") {
                    failed.push(format!("{}: 生成代码缺少 hicc::cpp! 块", unit_name));
                    continue;
                }

                // 验证 2：必须有 import_lib! 块
                if !code.contains("hicc::import_lib! {") {
                    failed.push(format!(
                        "{}: 生成代码缺少 hicc::import_lib! 块",
                        unit_name
                    ));
                    continue;
                }

                // 验证 3：函数绑定属性检查
                if !code.contains("#[cpp(func = \"") {
                    failed.push(format!(
                        "{}: import_lib! 块缺少 #[cpp(func = \"...\")] 注解",
                        unit_name
                    ));
                    continue;
                }

                // 验证 4：link_name 不含路径分隔符
                for line in code.lines() {
                    let trimmed = line.trim();
                    if let Some(rest) = trimmed.strip_prefix("#![link_name = \"") {
                        let link_name = rest.strip_suffix("\"]").unwrap_or(rest);
                        if link_name.contains('/') || link_name.contains('\\') {
                            failed.push(format!("{}: link_name 含路径分隔符：{}", unit_name, link_name));
                        }
                    }
                }

                println!(
                    "  [OK] {}: 生成 import_lib! 块，含 #[cpp(func)] 绑定",
                    unit_name
                );
                processed += 1;
            }
            None => {
                skipped.push(src_name);
            }
        }
    }

    assert!(
        skipped.is_empty(),
        "fmt-shim-ffi: {} 个文件预处理失败（g++ 是否已安装？）:\n{}",
        skipped.len(),
        skipped.join("\n")
    );

    assert!(
        failed.is_empty(),
        "fmt-shim-ffi: {} 个 shim 文件未能生成预期 FFI 绑定:\n{}",
        failed.len(),
        failed.join("\n")
    );

    assert_eq!(
        processed,
        FMT_SHIM_SOURCES.len(),
        "fmt-shim-ffi: 期望处理 {} 个文件，实际 {}",
        FMT_SHIM_SOURCES.len(),
        processed
    );

    println!("\nfmt-shim-ffi: {} 个 shim 文件全部生成 import_lib! FFI 绑定 ✓", processed);
}
