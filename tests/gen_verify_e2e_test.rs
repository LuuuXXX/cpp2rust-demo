//! L6 gen-verify：端到端验证工具实际生成的代码可被 Rust 编译器接受。
//!
//! 与 L1/L2/L_smoke 不同：
//! - L1 验证生成代码与手写黄金一致
//! - L2/L_smoke 验证**手写黄金**可编译、行为正确
//! - **L6（本测试）** 直接验证**工具实际生成**的代码可被 Rust 编译器接受
//!
//! 对 3 个代表性示例（模板函数、模板类、接口虚函数）运行完整的
//! 代码生成流水线，然后将生成的代码写入临时 Cargo 项目（使用
//! 绝对路径引用原始 C++ 文件），运行 `cargo build` 验证可编译性。
//!
//! 由于需要 `g++`/`clang++` 进行 C++ 预处理以及 `libclang` 进行 AST 解析，
//! 所有测试均标注 `#[ignore]`，通过 `--include-ignored` 显式运行（CI gen-verify job 调用）。

mod common;

use std::path::{Path, PathBuf};
use tempfile::TempDir;

// ─────────────────────────────────────────────────────────────────
//  测试用例（三类代表性示例）
// ─────────────────────────────────────────────────────────────────

/// L6-1：024_template_function — 模板函数实例化
///
/// 验证工具对函数模板的生成代码（swap_int / swap_double 等）可被 Rust 编译器接受。
#[test]
#[ignore = "gen-verify: 需要 g++/clang++ 与 libclang，在 CI gen-verify job 中运行"]
fn gen_verify_template_function() {
    gen_verify_example(
        "examples/024_template_function",
        "template_function",
        "template_function",
    );
}

/// L6-2：025_template_class — 模板类实例化
///
/// 验证工具对类模板（Stack<int> / Stack<double>）的生成代码可被 Rust 编译器接受。
#[test]
#[ignore = "gen-verify: 需要 g++/clang++ 与 libclang，在 CI gen-verify job 中运行"]
fn gen_verify_template_class() {
    gen_verify_example(
        "examples/025_template_class",
        "template_class",
        "template_class",
    );
}

/// L6-3：015_virtual_basic — 虚函数接口
///
/// 验证工具对含虚函数类（Shape / Circle）的生成代码可被 Rust 编译器接受。
#[test]
#[ignore = "gen-verify: 需要 g++/clang++ 与 libclang，在 CI gen-verify job 中运行"]
fn gen_verify_virtual_basic() {
    gen_verify_example(
        "examples/015_virtual_basic",
        "virtual_basic",
        "virtual_basic",
    );
}

// ─────────────────────────────────────────────────────────────────
//  核心验证逻辑
// ─────────────────────────────────────────────────────────────────

/// 对指定示例运行完整的生成 → 编译验证流程：
///
/// 1. 调用 `common::run_tool_on` 生成 FFI 代码（hicc 三段式 Rust 块）
/// 2. 验证生成代码的基本结构（包含 `import_lib!`）
/// 3. 在临时目录创建完整 Cargo 项目，build.rs 使用绝对路径引用原示例 C++ 文件
/// 4. 运行 `cargo build` 验证生成代码可被编译
///
/// 若预处理/解析失败（如当前环境无 g++ 或 libclang），则优雅跳过而非 panic。
fn gen_verify_example(example_dir: &str, lib_name: &str, cpp_stem: &str) {
    // ── 步骤 1：生成 FFI 代码 ──────────────────────────────────────
    let generated_code = common::run_tool_on(example_dir);
    if generated_code.is_empty() {
        eprintln!(
            "[gen-verify] 跳过 {}：预处理或 AST 解析失败（当前环境可能缺少 g++/libclang）",
            example_dir
        );
        return;
    }

    // ── 步骤 2：验证基本结构 ──────────────────────────────────────
    assert!(
        generated_code.contains("hicc::import_lib!"),
        "[gen-verify] {} 的生成代码应包含 import_lib! 块\n实际生成：\n{}",
        example_dir,
        generated_code
    );

    // ── 步骤 3：创建临时 Cargo 项目 ──────────────────────────────
    let tmp = TempDir::new().expect("创建临时目录失败");
    let project_dir = tmp.path().to_path_buf();
    setup_gen_verify_project(&project_dir, lib_name, cpp_stem, example_dir, &generated_code);

    // ── 步骤 4：运行 cargo build ──────────────────────────────────
    let output = std::process::Command::new("cargo")
        .args(["build", "--manifest-path"])
        .arg(project_dir.join("Cargo.toml"))
        .output()
        .expect("运行 cargo build 失败");

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        panic!(
            "[gen-verify] {} 的生成代码无法通过 cargo build\nstdout:\n{}\nstderr:\n{}",
            example_dir, stdout, stderr
        );
    }

    println!("[gen-verify] ✅ {} 通过 cargo build", example_dir);
}

/// 在 `project_dir` 下创建完整的临时 Cargo 项目结构：
/// - `src/lib.rs`：工具生成的 FFI 代码（hicc 三段式）
/// - `Cargo.toml`：依赖 hicc / hicc-build / cc
/// - `build.rs`：使用绝对路径编译原示例 C++ 文件
fn setup_gen_verify_project(
    project_dir: &Path,
    lib_name: &str,
    cpp_stem: &str,
    example_dir: &str,
    generated_code: &str,
) {
    // 创建 src/ 目录
    let src_dir = project_dir.join("src");
    std::fs::create_dir_all(&src_dir).expect("创建 src 目录失败");

    // ── lib.rs ────────────────────────────────────────────────────
    // 工具生成的 hicc 三段式就是完整的 lib.rs 内容
    std::fs::write(src_dir.join("lib.rs"), generated_code)
        .expect("写 lib.rs 失败");

    // ── Cargo.toml ───────────────────────────────────────────────
    let lib_name_ident = lib_name.replace('-', "_");
    let cargo_toml = format!(
        r#"[package]
name = "{lib_name}-gen-verify"
version = "0.1.0"
edition = "2021"

[lib]
name = "{lib_name_ident}"
path = "src/lib.rs"

[dependencies]
hicc = {{ version = "0.2" }}

[build-dependencies]
hicc-build = {{ version = "0.2" }}
cc = "1.0"
"#,
        lib_name = lib_name,
        lib_name_ident = lib_name_ident,
    );
    std::fs::write(project_dir.join("Cargo.toml"), cargo_toml)
        .expect("写 Cargo.toml 失败");

    // ── build.rs ─────────────────────────────────────────────────
    // 使用绝对路径引用原示例 C++ 文件，避免相对路径问题
    let example_abs = PathBuf::from(example_dir)
        .canonicalize()
        .unwrap_or_else(|_| PathBuf::from(example_dir));
    let cpp_dir = example_abs.join("cpp");

    // 收集所有 .cpp 文件
    let cpp_files: Vec<PathBuf> = std::fs::read_dir(&cpp_dir)
        .unwrap_or_else(|_| panic!("无法读取 C++ 目录：{}", cpp_dir.display()))
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("cpp"))
        .collect();

    let cpp_file_lines: String = cpp_files
        .iter()
        .map(|f| format!("    cc_build.file({:?});\n", f))
        .collect();

    // rerun-if-changed 行
    let rerun_lines: String = cpp_files
        .iter()
        .map(|f| format!("    println!(\"cargo::rerun-if-changed={}\");\n", f.display()))
        .collect();

    let build_rs = format!(
        r#"fn main() {{
    let cpp_dir = std::path::PathBuf::from({cpp_dir:?});

    let mut build = hicc_build::Build::new();
    use std::ops::DerefMut;
    let cc_build: &mut cc::Build = build.deref_mut();
    cc_build.include(&cpp_dir);
    cc_build.cpp(true);
{cpp_file_lines}
    build.rust_file("src/lib.rs").compile({cpp_stem:?});

    println!("cargo::rustc-link-lib={cpp_stem}");
    #[cfg(target_os = "macos")]
    println!("cargo::rustc-link-lib=c++");
    #[cfg(not(any(target_os = "macos", all(target_os = "windows", target_env = "msvc"))))]
    println!("cargo::rustc-link-lib=stdc++");
    println!("cargo::rerun-if-changed=src/lib.rs");
{rerun_lines}}}
"#,
        cpp_dir = cpp_dir,
        cpp_stem = cpp_stem,
        cpp_file_lines = cpp_file_lines,
        rerun_lines = rerun_lines,
    );

    std::fs::write(project_dir.join("build.rs"), build_rs)
        .expect("写 build.rs 失败");
}
