//! E2E-2: pugixml 端到端集成测试（简单项目）
//!
//! pugixml 是单头 + 单源的 XML 解析库，具有清晰的 `xml_document`/`xml_node`/
//! `xml_attribute` OOP API，并包含迭代器类，复杂度略高于 tinyxml2。
//!
//! 验证工具能正确处理：
//! - `xml_document`/`xml_node`/`xml_attribute` 等类的提取
//! - 迭代器类型的识别
//! - 同目录头文件 include
//! - init + merge 两阶段完整流程及生成 Rust 项目的可编译性（`cargo check`）

mod common;

use cpp2rust_demo::{generator::project_generator, merger};
use std::path::Path;
use tempfile::TempDir;

const PROJECT_ROOT: &str = "references/pugixml";
const PUGIXML_SRC: &str = "references/pugixml/src";

const SOURCES: &[&str] = &["src/pugixml.cpp"];

#[test]
fn pugixml_init_sources() {
    if !Path::new(PROJECT_ROOT).exists() {
        eprintln!("pugixml_e2e: 子模块未初始化，跳过（运行 git submodule update --init references/pugixml）");
        return;
    }

    let tmp = TempDir::new().unwrap();
    let preprocess_dir = tmp.path().join("preprocessed");
    std::fs::create_dir_all(&preprocess_dir).unwrap();

    let includes = &[PUGIXML_SRC];

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
        "pugixml E2E: {} 个文件处理失败:\n{}",
        skipped.len(),
        skipped.join("\n")
    );
    assert_eq!(
        processed,
        SOURCES.len(),
        "pugixml E2E: 期望处理 {} 个文件，实际 {}",
        SOURCES.len(),
        processed
    );
}

// ─────────────────────────────────────────────────────────────────
//  Merge 阶段测试：init + merge + cargo check
// ─────────────────────────────────────────────────────────────────

/// 验证 pugixml 完整的 init + merge 两阶段流程，并对生成的 Rust 项目执行 `cargo check`，
/// 确保输出代码可编译。与 `rapidjson_merge_phase` 模式一致。
#[test]
fn pugixml_merge_phase() {
    if !Path::new(PROJECT_ROOT).exists() {
        eprintln!("pugixml_merge_phase: 子模块未初始化，跳过（运行 git submodule update --init references/pugixml）");
        return;
    }

    let tmp = TempDir::new().unwrap();
    let preprocess_dir = tmp.path().join("c");
    let rust_dir = tmp.path().join("rust");
    std::fs::create_dir_all(&preprocess_dir).unwrap();

    let includes = &[PUGIXML_SRC];
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
        "pugixml_merge_phase: init 阶段未生成任何 unit 文件（g++ / clang++ 是否已安装？）"
    );

    // 生成 Cargo.toml 与 lib.rs
    project_generator::write_cargo_toml(&rust_dir, "pugixml").expect("write_cargo_toml 失败");
    project_generator::write_lib_rs(&rust_dir, &unit_paths).expect("write_lib_rs 失败");

    // ── Merge 阶段 ─────────────────────────────────────────────────
    merger::merge_in_place(&rust_dir).expect("merge_in_place 失败");

    // ── 验证输出目录结构 ────────────────────────────────────────────
    let src1 = rust_dir.join("src.1");
    let src_dir = rust_dir.join("src");

    assert!(src1.is_dir(), "pugixml_merge_phase: src.1/ 目录不存在");
    assert!(
        src_dir.is_dir() && !src_dir.is_symlink(),
        "pugixml_merge_phase: src/ 不存在或为符号链接"
    );
    assert!(
        !rust_dir.join("src.2").exists(),
        "pugixml_merge_phase: src.2 应已被 rename 为 src"
    );

    // ── 验证合并后文件 hicc 格式 ─────────────────────────────────
    let merged_files = merger::collect_unit_rs_files(&src_dir);
    assert!(
        !merged_files.is_empty(),
        "pugixml_merge_phase: src/ 下未找到任何 .rs 文件"
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
                "pugixml_merge_phase: cargo check 失败（生成的 Rust 项目不可编译）:\n{}",
                stderr
            );
        }
        Ok(_) => println!("pugixml_merge_phase: cargo check 通过 ({} 个 unit)", unit_paths.len()),
        Err(e) => eprintln!("pugixml_merge_phase: cargo check 跳过（cargo 不可用: {}）", e),
    }
}
