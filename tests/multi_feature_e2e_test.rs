//! E2E: 真实项目多 feature 合并 & output-dir 输出验证
//!
//! 使用 rapidjson 真实 C++ 项目（examples + shim 文件）作为源码，验证：
//! 1. `merge --feature feat_a --feature feat_b` 多 feature 合并流程
//! 2. `merge --feature feat_a --output-dir <dir>` output-dir 导出功能
//!
//! 每个测试函数使用独立的 TempDir 保证完全隔离，
//! 在 Ubuntu / macOS / Windows 多平台 CI 中运行。

mod common;

use cpp2rust_demo::generator::project_generator;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

const RAPIDJSON_ROOT: &str = "references/rapidjson-refactoring/rapidjson_legacy";
const RAPIDJSON_INCLUDE: &str = "references/rapidjson-refactoring/rapidjson_legacy/include";
const RAPIDJSON_SHIM_DIR: &str = "references/rapidjson-refactoring/rapidjson_sys/shim";

/// Feature A：rapidjson 公开 API examples（少量，快速预处理）
const FEATURE_A_SOURCES: &[&str] = &[
    "example/tutorial/tutorial.cpp",
    "example/simpledom/simpledom.cpp",
    "example/simplewriter/simplewriter.cpp",
];

/// Feature B：rapidjson extern-C shim 文件（少量）
const FEATURE_B_SHIM_SOURCES: &[&str] = &[
    "document_ffi.cpp",
    "value_ffi.cpp",
    "reader_ffi.cpp",
];

// ─────────────────────────────────────────────────────────────────
//  辅助函数
// ─────────────────────────────────────────────────────────────────

/// 初始化 feature 目录结构（`.cpp2rust/<feature>/`），向其中写入由真实
/// C++ 源文件生成的 hicc unit `.rs` 文件，并写出 Cargo.toml / lib.rs / build.rs。
///
/// 返回成功写入的 unit 名称列表；若所有文件均预处理失败则返回空列表。
fn setup_feature(
    project_root: &Path,
    feature: &str,
    sources: &[&str],
    base_dir: &Path,
    include_dirs: &[&str],
) -> Vec<String> {
    let feature_root = project_root.join(".cpp2rust").join(feature);
    let rust_dir = feature_root.join("rust");
    let meta_dir = feature_root.join("meta");
    let preprocess_dir = feature_root.join("c");

    std::fs::create_dir_all(rust_dir.join("src")).unwrap();
    std::fs::create_dir_all(&meta_dir).unwrap();
    std::fs::create_dir_all(&preprocess_dir).unwrap();

    let mut unit_names: Vec<String> = Vec::new();

    for src_rel in sources {
        let src_path = base_dir.join(src_rel);
        if let Some((unit_name, code)) =
            common::process_cpp_source(&src_path, include_dirs, &preprocess_dir)
        {
            project_generator::write_unit_rs(&rust_dir, &unit_name, &code)
                .unwrap_or_else(|e| panic!("write_unit_rs {}: {}", unit_name, e));
            unit_names.push(unit_name);
        } else {
            eprintln!(
                "  setup_feature '{}': 跳过预处理失败的文件 {}",
                feature, src_rel
            );
        }
    }

    if unit_names.is_empty() {
        return unit_names;
    }

    project_generator::write_cargo_toml(&rust_dir, feature)
        .unwrap_or_else(|e| panic!("write_cargo_toml: {}", e));
    project_generator::write_lib_rs(&rust_dir, &unit_names)
        .unwrap_or_else(|e| panic!("write_lib_rs: {}", e));
    project_generator::write_build_rs(&rust_dir, feature, &unit_names, &[], &[])
        .unwrap_or_else(|e| panic!("write_build_rs: {}", e));

    // meta/build_cmd.txt：merge 阶段 FeatureLayout::save_merge_report 写入此目录
    std::fs::write(meta_dir.join("build_cmd.txt"), "make")
        .expect("写入 build_cmd.txt 失败");

    println!(
        "  setup_feature '{}': {} 个 unit 文件就绪",
        feature,
        unit_names.len()
    );
    unit_names
}

/// 调用 cpp2rust-demo 二进制，`current_dir` 设为 `project_root`。
fn run_binary(project_root: &Path, args: &[&str]) -> std::process::Output {
    std::process::Command::new(env!("CARGO_BIN_EXE_cpp2rust-demo"))
        .args(args)
        .current_dir(project_root)
        .output()
        .unwrap_or_else(|e| panic!("启动 cpp2rust-demo 失败: {}", e))
}

/// 打印二进制输出（调试用）。
fn dump_output(label: &str, out: &std::process::Output) {
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    if !stdout.is_empty() {
        println!("[{}] stdout:\n{}", label, stdout);
    }
    if !stderr.is_empty() {
        eprintln!("[{}] stderr:\n{}", label, stderr);
    }
}

// ─────────────────────────────────────────────────────────────────
//  Test 1: 多 feature 合并
// ─────────────────────────────────────────────────────────────────

/// 验证 `merge --feature feat_examples --feature feat_shim` 的完整流程：
/// - 输出目录 `.cpp2rust/feat_examples_feat_shim/rust/` 存在
/// - Cargo.toml 包含 `[features]` 且列出两个 feature
/// - `src/lib.rs` 含条件编译守卫
/// - `src/feat_examples/` 与 `src/feat_shim/` 模块目录均存在
#[test]
fn rapidjson_multi_feature_merge() {
    if !Path::new(RAPIDJSON_ROOT).exists() {
        eprintln!(
            "multi_feature_e2e: rapidjson 子模块未初始化，跳过\
             （运行 git submodule update --init references/rapidjson-refactoring）"
        );
        return;
    }

    let tmp = TempDir::new().unwrap();
    let project_root = tmp.path().to_path_buf();
    // 创建 .cpp2rust/ 目录，使 find_project_root 能识别此目录为项目根
    std::fs::create_dir_all(project_root.join(".cpp2rust")).unwrap();

    let rapidjson_root = PathBuf::from(RAPIDJSON_ROOT);
    let shim_dir = PathBuf::from(RAPIDJSON_SHIM_DIR);

    // ── 初始化两个 feature ────────────────────────────────────────────────
    let feat_a_units = setup_feature(
        &project_root,
        "feat_examples",
        FEATURE_A_SOURCES,
        &rapidjson_root,
        &[RAPIDJSON_INCLUDE],
    );
    if feat_a_units.is_empty() {
        eprintln!("multi_feature_e2e: feat_examples 预处理全部失败，跳过（g++ / clang++ 是否已安装？）");
        return;
    }

    let feat_b_units = setup_feature(
        &project_root,
        "feat_shim",
        FEATURE_B_SHIM_SOURCES,
        &shim_dir,
        &[RAPIDJSON_INCLUDE, RAPIDJSON_SHIM_DIR],
    );
    if feat_b_units.is_empty() {
        eprintln!("multi_feature_e2e: feat_shim 预处理全部失败，跳过");
        return;
    }

    // ── 执行多 feature 合并 ───────────────────────────────────────────────
    let output = run_binary(
        &project_root,
        &[
            "merge",
            "--feature", "feat_examples",
            "--feature", "feat_shim",
        ],
    );
    dump_output("multi-feature merge", &output);

    assert!(
        output.status.success(),
        "多 feature 合并失败（退出码: {:?}）\nstdout: {}\nstderr: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );

    // ── 验证输出目录结构 ──────────────────────────────────────────────────
    let combined_rust = project_root
        .join(".cpp2rust")
        .join("feat_examples_feat_shim")
        .join("rust");

    assert!(
        combined_rust.exists(),
        "合并输出目录不存在：{}",
        combined_rust.display()
    );
    assert!(
        combined_rust.join("Cargo.toml").exists(),
        "合并项目缺少 Cargo.toml"
    );
    assert!(
        combined_rust.join("build.rs").exists(),
        "合并项目缺少 build.rs"
    );
    assert!(
        combined_rust.join("src").join("lib.rs").exists(),
        "合并项目缺少 src/lib.rs"
    );
    assert!(
        combined_rust.join("src").join("feat_examples").exists(),
        "合并项目缺少 src/feat_examples/ 模块目录"
    );
    assert!(
        combined_rust.join("src").join("feat_shim").exists(),
        "合并项目缺少 src/feat_shim/ 模块目录"
    );

    // Cargo.toml 必须含 [features] 及两个 feature 条目
    let cargo_toml_content =
        std::fs::read_to_string(combined_rust.join("Cargo.toml")).unwrap();
    assert!(
        cargo_toml_content.contains("feat_examples = []"),
        "Cargo.toml 缺少 feat_examples feature 条目\n内容：\n{}",
        cargo_toml_content
    );
    assert!(
        cargo_toml_content.contains("feat_shim = []"),
        "Cargo.toml 缺少 feat_shim feature 条目\n内容：\n{}",
        cargo_toml_content
    );

    // src/lib.rs 必须含 #[cfg(feature = "...")] 条件编译守卫
    let lib_rs_content =
        std::fs::read_to_string(combined_rust.join("src").join("lib.rs")).unwrap();
    assert!(
        lib_rs_content.contains("#[cfg(feature = \"feat_examples\")]"),
        "src/lib.rs 缺少 feat_examples 条件编译守卫\n内容：\n{}",
        lib_rs_content
    );
    assert!(
        lib_rs_content.contains("#[cfg(feature = \"feat_shim\")]"),
        "src/lib.rs 缺少 feat_shim 条件编译守卫\n内容：\n{}",
        lib_rs_content
    );

    println!(
        "\n✓ 多 feature 合并验证通过 (feat_examples[{}个单元] + feat_shim[{}个单元])",
        feat_a_units.len(),
        feat_b_units.len()
    );
}

// ─────────────────────────────────────────────────────────────────
//  Test 2: output-dir 单 feature 导出
// ─────────────────────────────────────────────────────────────────

/// 验证 `merge --feature feat_shim --output-dir <dir>` 的完整流程：
/// - 导出目录存在
/// - `Cargo.toml` 已复制到导出目录
/// - `build.rs` 已复制到导出目录
/// - `src/lib.rs` 已复制到导出目录
/// - `meta/` 目录存在（`.cpp2rust/` 的副本）
#[test]
fn rapidjson_merge_output_dir() {
    if !Path::new(RAPIDJSON_ROOT).exists() {
        eprintln!(
            "output_dir_e2e: rapidjson 子模块未初始化，跳过\
             （运行 git submodule update --init references/rapidjson-refactoring）"
        );
        return;
    }

    let tmp = TempDir::new().unwrap();
    let project_root = tmp.path().to_path_buf();
    std::fs::create_dir_all(project_root.join(".cpp2rust")).unwrap();

    let shim_dir = PathBuf::from(RAPIDJSON_SHIM_DIR);

    // ── 初始化一个 feature ────────────────────────────────────────────────
    let feat_units = setup_feature(
        &project_root,
        "feat_shim",
        FEATURE_B_SHIM_SOURCES,
        &shim_dir,
        &[RAPIDJSON_INCLUDE, RAPIDJSON_SHIM_DIR],
    );
    if feat_units.is_empty() {
        eprintln!("output_dir_e2e: feat_shim 预处理全部失败，跳过（g++ / clang++ 是否已安装？）");
        return;
    }

    // ── 执行 merge --output-dir ───────────────────────────────────────────
    let out_dir = tmp.path().join("exported");
    let output = run_binary(
        &project_root,
        &[
            "merge",
            "--feature", "feat_shim",
            "--output-dir", out_dir.to_str().unwrap(),
        ],
    );
    dump_output("merge --output-dir", &output);

    assert!(
        output.status.success(),
        "merge --output-dir 失败（退出码: {:?}）\nstdout: {}\nstderr: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );

    // ── 验证导出目录结构 ──────────────────────────────────────────────────
    assert!(
        out_dir.exists(),
        "导出目录不存在：{}",
        out_dir.display()
    );
    assert!(
        out_dir.join("Cargo.toml").exists(),
        "导出目录缺少 Cargo.toml"
    );
    assert!(
        out_dir.join("build.rs").exists(),
        "导出目录缺少 build.rs"
    );
    assert!(
        out_dir.join("src").join("lib.rs").exists(),
        "导出目录缺少 src/lib.rs"
    );
    assert!(
        out_dir.join("meta").exists(),
        "导出目录缺少 meta/（.cpp2rust/ 副本）"
    );

    println!(
        "\n✓ merge --output-dir 验证通过 (feat_shim[{}个单元] → {})",
        feat_units.len(),
        out_dir.display()
    );
}

// ─────────────────────────────────────────────────────────────────
//  Test 3: 多 feature 合并 + output-dir 联合验证
// ─────────────────────────────────────────────────────────────────

/// 验证 `merge --feature feat_a --feature feat_b --output-dir <dir>` 联合流程：
/// - 先执行多 feature 合并
/// - 再将合并结果导出到 output-dir
/// - 导出目录含 Cargo.toml / build.rs / src/lib.rs / meta/
/// - 导出目录的 src/lib.rs 含两个 feature 的条件编译守卫
#[test]
fn rapidjson_multi_feature_with_output_dir() {
    if !Path::new(RAPIDJSON_ROOT).exists() {
        eprintln!(
            "multi_feature_output_dir_e2e: rapidjson 子模块未初始化，跳过"
        );
        return;
    }

    let tmp = TempDir::new().unwrap();
    let project_root = tmp.path().to_path_buf();
    std::fs::create_dir_all(project_root.join(".cpp2rust")).unwrap();

    let rapidjson_root = PathBuf::from(RAPIDJSON_ROOT);
    let shim_dir = PathBuf::from(RAPIDJSON_SHIM_DIR);

    // ── 初始化两个 feature ────────────────────────────────────────────────
    let feat_a_units = setup_feature(
        &project_root,
        "feat_examples",
        FEATURE_A_SOURCES,
        &rapidjson_root,
        &[RAPIDJSON_INCLUDE],
    );
    if feat_a_units.is_empty() {
        eprintln!("multi_feature_output_dir_e2e: feat_examples 预处理失败，跳过");
        return;
    }

    let feat_b_units = setup_feature(
        &project_root,
        "feat_shim",
        FEATURE_B_SHIM_SOURCES,
        &shim_dir,
        &[RAPIDJSON_INCLUDE, RAPIDJSON_SHIM_DIR],
    );
    if feat_b_units.is_empty() {
        eprintln!("multi_feature_output_dir_e2e: feat_shim 预处理失败，跳过");
        return;
    }

    // ── 执行多 feature 合并 + output-dir ─────────────────────────────────
    let out_dir = tmp.path().join("multi_exported");
    let output = run_binary(
        &project_root,
        &[
            "merge",
            "--feature", "feat_examples",
            "--feature", "feat_shim",
            "--output-dir", out_dir.to_str().unwrap(),
        ],
    );
    dump_output("multi-feature merge --output-dir", &output);

    assert!(
        output.status.success(),
        "多 feature merge --output-dir 失败（退出码: {:?}）\nstdout: {}\nstderr: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );

    // ── 验证导出目录结构 ──────────────────────────────────────────────────
    assert!(out_dir.exists(), "导出目录不存在：{}", out_dir.display());
    assert!(out_dir.join("Cargo.toml").exists(), "导出目录缺少 Cargo.toml");
    assert!(out_dir.join("build.rs").exists(), "导出目录缺少 build.rs");
    assert!(
        out_dir.join("src").join("lib.rs").exists(),
        "导出目录缺少 src/lib.rs"
    );
    assert!(out_dir.join("meta").exists(), "导出目录缺少 meta/");

    // 导出的 src/lib.rs 应含两个 feature 的条件编译守卫
    let lib_rs_content =
        std::fs::read_to_string(out_dir.join("src").join("lib.rs")).unwrap();
    assert!(
        lib_rs_content.contains("#[cfg(feature = \"feat_examples\")]"),
        "导出 src/lib.rs 缺少 feat_examples 守卫\n内容：\n{}",
        lib_rs_content
    );
    assert!(
        lib_rs_content.contains("#[cfg(feature = \"feat_shim\")]"),
        "导出 src/lib.rs 缺少 feat_shim 守卫\n内容：\n{}",
        lib_rs_content
    );

    println!(
        "\n✓ 多 feature merge --output-dir 验证通过 \
         (feat_examples[{}] + feat_shim[{}] → {})",
        feat_a_units.len(),
        feat_b_units.len(),
        out_dir.display()
    );
}
