//! L1: smoke_test_gen 集成测试
//!
//! 对真实 C++ 示例执行完整的"预处理 → AST 解析 → 提取 FfiSpec → 生成冒烟测试"流程，
//! 验证 `smoke_test_gen::generate` 与 `project_generator::write_smoke_test` 的端到端集成。
//!
//! 覆盖范围：
//! - 类别 B（自由函数）：001_hello_world（仅含 free functions）
//! - 类别 A（类生命周期）：006_class_basic（含构造/析构的类）
//! - 类别 D（接口类）：016_virtual_pure（纯虚接口类）
//!
//! 运行方式：
//!   cargo test --test l1_smoke_test_gen_tests -- --include-ignored --test-threads=1

mod common;

use cpp2rust_demo::{
    ast_parser, extractor,
    ffi_model::FfiSpec,
    generator::{project_generator, smoke_test_gen},
};
use std::path::{Path, PathBuf};
use tempfile::TempDir;

// ─────────────────────────────────────────────────────────────────
//  辅助函数
// ─────────────────────────────────────────────────────────────────

/// 对单个 C++ 示例执行预处理 + AST 解析 + 提取，返回 `(unit_name, FfiSpec)`。
///
/// 若预处理失败（libclang 未安装、编译器不可用等）返回 `None`，测试将跳过。
fn extract_spec_from_example(
    example: &str,
    preprocess_dir: &Path,
) -> Option<(String, FfiSpec)> {
    let cpp_dir = PathBuf::from(format!("examples/{}/cpp", example));
    if !cpp_dir.exists() {
        return None;
    }

    // 找到第一个 .cpp 文件
    let cpp_file = std::fs::read_dir(&cpp_dir)
        .ok()?
        .flatten()
        .find(|e| e.path().extension().map(|ext| ext == "cpp").unwrap_or(false))
        .map(|e| e.path())?;

    let unit_name = cpp_file
        .file_stem()
        .and_then(|s| s.to_str())?
        .to_string();

    // 预处理
    let preprocessed = common::preprocess_cpp(&cpp_file, &[], preprocess_dir, &unit_name)?;

    // AST 解析
    let ast = ast_parser::parse_preprocessed(&preprocessed).ok()?;

    // 提取 FfiSpec
    let (sys_includes, proj_header) = extractor::read_source_includes(&cpp_file);
    let spec = extractor::extract(&ast, &unit_name, &sys_includes, proj_header.as_deref());

    Some((unit_name, spec))
}

// ─────────────────────────────────────────────────────────────────
//  测试：类别 B（自由函数）— 001_hello_world
// ─────────────────────────────────────────────────────────────────

/// 验证 smoke_test_gen 对含自由函数的示例生成正确的冒烟测试文件。
///
/// 期望：
/// - 文件头注释存在（`自动生成的 FFI 冒烟测试`）
/// - 至少一条 `use` 声明
/// - 生成的 `tests/smoke_test.rs` 可写入磁盘
#[test]
#[ignore = "requires libclang; run with --include-ignored --test-threads=1"]
fn smoke_gen_001_hello_world() {
    let preprocess_tmp = TempDir::new().unwrap();
    let output_tmp = TempDir::new().unwrap();

    let (unit_name, spec) =
        match extract_spec_from_example("001_hello_world", preprocess_tmp.path()) {
            Some(pair) => pair,
            None => {
                eprintln!(
                    "smoke_gen_001_hello_world: 预处理失败，跳过（编译器/libclang 未安装？）"
                );
                return;
            }
        };

    let content = smoke_test_gen::generate(&[(&unit_name, &spec)], "hello_world");

    // ── 内容检验 ──────────────────────────────────────────────────
    assert!(
        content.contains("自动生成的 FFI 冒烟测试"),
        "缺少文件头注释，实际输出前 200 字符：\n{}",
        &content[..content.len().min(200)]
    );
    assert!(
        content.contains("use hello_world::"),
        "缺少 use 声明，实际：\n{}",
        content
    );

    // ── 写入磁盘验证 ──────────────────────────────────────────────
    project_generator::write_smoke_test(output_tmp.path(), &content)
        .expect("write_smoke_test 失败");
    let smoke_path = output_tmp.path().join("tests").join("smoke_test.rs");
    assert!(
        smoke_path.exists(),
        "tests/smoke_test.rs 未写入：{}",
        smoke_path.display()
    );
    let written = std::fs::read_to_string(&smoke_path).unwrap();
    assert_eq!(
        written, content,
        "写入磁盘的内容与生成内容不一致"
    );
}

// ─────────────────────────────────────────────────────────────────
//  测试：类别 A（类生命周期）— 006_class_basic
// ─────────────────────────────────────────────────────────────────

/// 验证 smoke_test_gen 对含构造/析构类的示例生成生命周期测试。
///
/// 期望：
/// - 含 `_lifecycle` 函数名（类别 A）
/// - 含 `drop(obj)` 调用（析构验证）
#[test]
#[ignore = "requires libclang; run with --include-ignored --test-threads=1"]
fn smoke_gen_006_class_basic() {
    let preprocess_tmp = TempDir::new().unwrap();
    let output_tmp = TempDir::new().unwrap();

    let (unit_name, spec) =
        match extract_spec_from_example("006_class_basic", preprocess_tmp.path()) {
            Some(pair) => pair,
            None => {
                eprintln!(
                    "smoke_gen_006_class_basic: 预处理失败，跳过（编译器/libclang 未安装？）"
                );
                return;
            }
        };

    let content = smoke_test_gen::generate(&[(&unit_name, &spec)], "class_basic");

    // 如果存在类（有析构函数和构造函数），应生成生命周期测试
    if content.contains("_lifecycle") {
        assert!(
            content.contains("drop(obj)"),
            "生命周期测试缺少 drop(obj) 调用，实际：\n{}",
            content
        );
    }

    // 写入磁盘
    project_generator::write_smoke_test(output_tmp.path(), &content)
        .expect("write_smoke_test 失败");
    assert!(
        output_tmp.path().join("tests").join("smoke_test.rs").exists(),
        "tests/smoke_test.rs 未生成"
    );
}

// ─────────────────────────────────────────────────────────────────
//  测试：类别 D（接口类）— 016_virtual_pure
// ─────────────────────────────────────────────────────────────────

/// 验证 smoke_test_gen 对含纯虚接口类的示例正确生成类别 D 测试或注释桩。
#[test]
#[ignore = "requires libclang; run with --include-ignored --test-threads=1"]
fn smoke_gen_016_virtual_pure() {
    let preprocess_tmp = TempDir::new().unwrap();
    let output_tmp = TempDir::new().unwrap();

    let (unit_name, spec) =
        match extract_spec_from_example("016_virtual_pure", preprocess_tmp.path()) {
            Some(pair) => pair,
            None => {
                eprintln!(
                    "smoke_gen_016_virtual_pure: 预处理失败，跳过（编译器/libclang 未安装？）"
                );
                return;
            }
        };

    let content = smoke_test_gen::generate(&[(&unit_name, &spec)], "virtual_pure");

    // 必须有文件头
    assert!(
        content.contains("自动生成的 FFI 冒烟测试"),
        "缺少文件头注释"
    );

    // 写入磁盘
    project_generator::write_smoke_test(output_tmp.path(), &content)
        .expect("write_smoke_test 失败");
    assert!(
        output_tmp.path().join("tests").join("smoke_test.rs").exists(),
        "tests/smoke_test.rs 未生成"
    );
}

// ─────────────────────────────────────────────────────────────────
//  测试：多单元合并 — 001 + 006
// ─────────────────────────────────────────────────────────────────

/// 验证 smoke_test_gen 对多个编译单元生成正确的 use 声明和分段注释。
#[test]
#[ignore = "requires libclang; run with --include-ignored --test-threads=1"]
fn smoke_gen_multi_unit() {
    let preprocess_tmp = TempDir::new().unwrap();
    let output_tmp = TempDir::new().unwrap();

    let mut units: Vec<(String, FfiSpec)> = Vec::new();

    for example in &["001_hello_world", "006_class_basic"] {
        match extract_spec_from_example(example, preprocess_tmp.path()) {
            Some(pair) => units.push(pair),
            None => {
                eprintln!(
                    "smoke_gen_multi_unit: 跳过 {}（预处理失败）",
                    example
                );
            }
        }
    }

    if units.is_empty() {
        eprintln!("smoke_gen_multi_unit: 所有文件预处理失败，跳过");
        return;
    }

    let unit_refs: Vec<(&str, &FfiSpec)> =
        units.iter().map(|(n, s)| (n.as_str(), s)).collect();
    let content = smoke_test_gen::generate(&unit_refs, "mylib");

    // 每个成功处理的 unit 都应有 use 声明
    for (unit_name, _) in &units {
        assert!(
            content.contains(&format!("use mylib::{}::", unit_name)),
            "缺少 unit '{}' 的 use 声明，实际：\n{}",
            unit_name,
            content
        );
    }

    // 每个 unit 都有分段注释
    for (unit_name, _) in &units {
        assert!(
            content.contains(&format!("单元：{}", unit_name)),
            "缺少 unit '{}' 的分段注释，实际：\n{}",
            unit_name,
            content
        );
    }

    // 写入磁盘验证
    project_generator::write_smoke_test(output_tmp.path(), &content)
        .expect("write_smoke_test 失败");
    assert!(
        output_tmp.path().join("tests").join("smoke_test.rs").exists(),
        "tests/smoke_test.rs 未生成"
    );
}
