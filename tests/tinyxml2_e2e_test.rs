//! E2E-1: tinyxml2 端到端集成测试（最简单项目）
//!
//! tinyxml2 是单头文件 + 单 .cpp 的经典 XML 解析库，包含典型 OOP 类层级：
//! `XMLDocument` → `XMLElement` → `XMLNode`，代码约 4K 行，复杂度最低。
//!
//! 验证工具能正确处理：
//! - 单文件项目的完整 init 流程
//! - 带继承关系的 C++ 类 (`XMLNode` 基类 / `XMLElement` 子类等)
//! - `#include` 同目录头文件的情形

mod common;

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
