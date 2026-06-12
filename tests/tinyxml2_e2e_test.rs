//! E2E-1: tinyxml2 端到端集成测试（最简单项目）
//!
//! tinyxml2 是单头文件 + 单 .cpp 的经典 XML 解析库，包含典型 OOP 类层级：
//! `XMLDocument` → `XMLElement` → `XMLNode`，代码约 4K 行，复杂度最低。
//!
//! 验证工具能正确处理：
//! - 单文件项目的完整 init 流程
//! - 带继承关系的 C++ 类 (`XMLNode` 基类 / `XMLElement` 子类等)
//! - `#include` 同目录头文件的情形
//! - init + merge 两阶段完整流程及生成 Rust 项目的可编译性（`cargo check`）

mod common;

use cpp2rust_demo::{generator::project_generator, merger};
use std::path::Path;
use tempfile::TempDir;

const PROJECT_ROOT: &str = "references/tinyxml2";

/// tinyxml2 主源文件（tinyxml2.cpp 包含完整实现 + tinyxml2.h 头文件）
const SOURCES: &[&str] = &["tinyxml2.cpp"];

#[test]
fn tinyxml2_init_sources() {
    if !Path::new(PROJECT_ROOT).exists() {
        eprintln!("tinyxml2_e2e: 子模块未初始化，跳过（运行 git submodule update --init references/tinyxml2）");
        return;
    }

    let tmp = TempDir::new().unwrap();
    let preprocess_dir = tmp.path().join("preprocessed");
    std::fs::create_dir_all(&preprocess_dir).unwrap();

    let includes = &[PROJECT_ROOT];

    let mut processed = 0usize;
    let mut skipped = Vec::new();

    for src_rel in SOURCES {
        let src_path = Path::new(PROJECT_ROOT).join(src_rel);
        match common::process_cpp_source(&src_path, includes, &preprocess_dir) {
            Some((unit_name, code)) => {
                common::assert_valid_hicc_format(&code, &unit_name);
                processed += 1;
            }
            None => {
                skipped.push(*src_rel);
            }
        }
    }

    assert!(
        skipped.is_empty(),
        "tinyxml2 E2E: {} 个文件处理失败:\n{}",
        skipped.len(),
        skipped.join("\n")
    );
    assert_eq!(
        processed,
        SOURCES.len(),
        "tinyxml2 E2E: 期望处理 {} 个文件，实际 {}",
        SOURCES.len(),
        processed
    );
}

// ─────────────────────────────────────────────────────────────────
//  Merge 阶段测试：init + merge + cargo check
// ─────────────────────────────────────────────────────────────────

/// 验证 tinyxml2 完整的 init + merge 两阶段流程，并对生成的 Rust 项目执行 `cargo check`，
/// 确保输出代码可编译。与 `rapidjson_merge_phase` 模式一致。
#[test]
fn tinyxml2_merge_phase() {
    if !Path::new(PROJECT_ROOT).exists() {
        eprintln!("tinyxml2_merge_phase: 子模块未初始化，跳过（运行 git submodule update --init references/tinyxml2）");
        return;
    }

    let tmp = TempDir::new().unwrap();
    let preprocess_dir = tmp.path().join("c");
    let rust_dir = tmp.path().join("rust");
    std::fs::create_dir_all(&preprocess_dir).unwrap();

    let includes = &[PROJECT_ROOT];
    let mut unit_paths: Vec<String> = Vec::new();

    // ── Init 阶段：生成所有 unit .rs 文件 ──────────────────────────
    for src_rel in SOURCES {
        let src_path = Path::new(PROJECT_ROOT).join(src_rel);
        if let Some((unit_name, code)) =
            common::process_cpp_source(&src_path, includes, &preprocess_dir)
        {
            project_generator::write_unit_rs(&rust_dir, &unit_name, &code)
                .expect("write_unit_rs 失败");
            unit_paths.push(unit_name);
        }
    }

    assert!(
        !unit_paths.is_empty(),
        "tinyxml2_merge_phase: init 阶段未生成任何 unit 文件（g++ / clang++ 是否已安装？）"
    );

    // 生成 Cargo.toml 与 lib.rs（merge 前必须存在 src/）
    project_generator::write_cargo_toml(&rust_dir, "tinyxml2").expect("write_cargo_toml 失败");
    project_generator::write_lib_rs(&rust_dir, &unit_paths).expect("write_lib_rs 失败");

    // ── Merge 阶段 ─────────────────────────────────────────────────
    merger::merge_in_place(&rust_dir).expect("merge_in_place 失败");

    // ── 验证输出目录结构 ────────────────────────────────────────────
    let src1 = rust_dir.join("src.1");
    let src_dir = rust_dir.join("src");

    assert!(src1.is_dir(), "tinyxml2_merge_phase: src.1/ 目录不存在");
    assert!(
        src_dir.is_dir() && !src_dir.is_symlink(),
        "tinyxml2_merge_phase: src/ 不存在或为符号链接"
    );
    assert!(
        !rust_dir.join("src.2").exists(),
        "tinyxml2_merge_phase: src.2 应已被 rename 为 src"
    );

    // ── 验证合并后文件 hicc 格式 ─────────────────────────────────
    let merged_files = merger::collect_unit_rs_files(&src_dir);
    assert!(
        !merged_files.is_empty(),
        "tinyxml2_merge_phase: src/ 下未找到任何 .rs 文件"
    );
    for rs_path in &merged_files {
        let fname = rs_path.file_name().and_then(|f| f.to_str()).unwrap_or("");
        if fname == "lib.rs" || fname == "mod.rs" {
            continue;
        }
        let content = std::fs::read_to_string(rs_path).expect("读取合并后 .rs 文件失败");
        common::assert_valid_hicc_format(&content, rs_path.to_str().unwrap_or("?"));
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
                "tinyxml2_merge_phase: cargo check 失败（生成的 Rust 项目不可编译）:\n{}",
                stderr
            );
        }
        Ok(_) => println!(
            "tinyxml2_merge_phase: cargo check 通过 ({} 个 unit)",
            unit_paths.len()
        ),
        Err(e) => eprintln!(
            "tinyxml2_merge_phase: cargo check 跳过（cargo 不可用: {}）",
            e
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
                    "tinyxml2_merge_phase: cargo test 失败（生成的冒烟测试未通过）:\n{}",
                    stderr
                );
            }
            Ok(_) => println!(
                "tinyxml2_merge_phase: cargo test 通过（生成的冒烟测试全部通过）"
            ),
            Err(e) => eprintln!(
                "tinyxml2_merge_phase: cargo test 跳过（cargo 不可用: {}）",
                e
            ),
        }
    } else {
        println!("tinyxml2_merge_phase: cargo test 跳过（未生成 tests/smoke.rs）");
    }
}
