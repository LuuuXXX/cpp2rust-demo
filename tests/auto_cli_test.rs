//! L5: `auto` 命令 CLI 端到端集成测试
//!
//! 使用 rapidjson 的 simpledom.cpp 作为输入，执行完整的
//! `cpp2rust-demo auto -- g++ -c ...` 流程，验证：
//! 1. 命令执行成功（exit code 0）
//! 2. `.cpp2rust/default/` 目录结构正确创建
//! 3. 生成的 Rust 文件符合 hicc 三段式格式

use assert_cmd::prelude::*;
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

const RAPIDJSON_INCLUDE: &str = "references/rapidjson-refactoring/rapidjson_legacy/include";
const RAPIDJSON_SIMPLEDOM: &str =
    "references/rapidjson-refactoring/rapidjson_legacy/example/simpledom/simpledom.cpp";

/// 返回 workspace 根目录（tests/ 目录的上级）。
fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// 验证生成的 hicc 代码符合三段式格式最低要求。
fn assert_valid_hicc(code: &str, label: &str) {
    assert!(
        code.contains("hicc::cpp! {"),
        "{}: 缺少 hicc::cpp! 块",
        label
    );
    assert!(
        code.trim_end().ends_with('}'),
        "{}: 输出未以 }} 结束",
        label
    );
}

/// L5-Auto: `auto` 命令对 rapidjson simpledom 的完整 CLI 端到端测试。
///
/// 将 simpledom.cpp 复制到临时目录（作为项目根），使 LD_PRELOAD hook 能正确
/// 识别源文件在项目根下，从而触发捕获。
#[test]
fn auto_cli_rapidjson_simpledom() {
    let root = workspace_root();
    let include_abs = root.join(RAPIDJSON_INCLUDE);
    let src_abs = root.join(RAPIDJSON_SIMPLEDOM);

    // 跳过：rapidjson 源文件或 include 不存在
    if !src_abs.exists() || !include_abs.exists() {
        eprintln!(
            "auto_cli_rapidjson_simpledom: SKIP — rapidjson source not found at {}",
            src_abs.display()
        );
        return;
    }

    // 创建临时工作目录（作为 C++ 项目根目录）
    let project_dir = TempDir::new().expect("create temp dir");
    let project_path = project_dir.path();

    // 将 simpledom.cpp 复制到 <project>/src/，确保源文件在项目根目录下
    // （hook 的 strip_prefix 检查要求 realpath(src) 必须在 CPP2RUST_PROJECT_ROOT 下）
    let src_local_dir = project_path.join("src");
    std::fs::create_dir_all(&src_local_dir).expect("create src dir");
    let src_local = src_local_dir.join("simpledom.cpp");
    std::fs::copy(&src_abs, &src_local).expect("copy simpledom.cpp");

    // 构建 build command：使用相对路径，这样 hook 能将其解析为项目根下的绝对路径
    let include_flag = format!("-I{}", include_abs.display());
    let build_cmd_args: &[&str] = &[
        "g++", "-c", &include_flag, "src/simpledom.cpp", "-o", "/dev/null",
    ];

    // 执行 cpp2rust-demo auto
    let mut cmd = Command::cargo_bin("cpp2rust-demo").expect("cpp2rust-demo binary");
    cmd.current_dir(project_path);
    cmd.arg("auto");
    cmd.arg("--");
    for a in build_cmd_args {
        cmd.arg(a);
    }

    let output = cmd.output().expect("failed to run cpp2rust-demo auto");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "cpp2rust-demo auto failed\n--- stdout ---\n{}\n--- stderr ---\n{}",
        stdout,
        stderr
    );

    // ── 验证输出目录结构 ───────────────────────────────────────────────
    let feature_dir = project_path.join(".cpp2rust").join("default");
    let rust_dir = feature_dir.join("rust");
    let c_dir = feature_dir.join("c");

    assert!(
        c_dir.exists(),
        "auto: .cpp2rust/default/c/ 目录不存在\nstdout: {}",
        stdout
    );
    assert!(
        rust_dir.exists(),
        "auto: .cpp2rust/default/rust/ 目录不存在\nstdout: {}",
        stdout
    );
    assert!(
        rust_dir.join("Cargo.toml").exists(),
        "auto: Cargo.toml 未生成"
    );

    // ── 验证至少生成了一个 .rs 文件 ───────────────────────────────────
    let rs_files: Vec<_> = walkdir::WalkDir::new(&rust_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path().extension().is_some_and(|x| x == "rs")
                && e.file_name() != "lib.rs"
                && e.file_name() != "mod.rs"
        })
        .collect();

    assert!(
        !rs_files.is_empty(),
        "auto: rust/src/ 下未找到任何 .rs 文件\nstdout: {}",
        stdout
    );

    // ── 验证 hicc 格式 ────────────────────────────────────────────────
    for entry in &rs_files {
        let code = std::fs::read_to_string(entry.path()).expect("read .rs file");
        assert_valid_hicc(&code, entry.path().to_str().unwrap_or("?"));
    }

    // ── 验证 merge 输出（src.1 / src.2 / src symlink）────────────────
    let src1 = rust_dir.join("src.1");
    let src2 = rust_dir.join("src.2");
    let src_link = rust_dir.join("src");

    assert!(src1.is_dir(), "auto: src.1/ 不存在（merge 未运行？）");
    assert!(src2.is_dir(), "auto: src.2/ 不存在（merge 未运行？）");
    assert!(src_link.is_symlink(), "auto: src 不是符号链接");

    println!(
        "auto_cli_rapidjson_simpledom: ✓ {} unit .rs file(s) generated and validated",
        rs_files.len()
    );
}
