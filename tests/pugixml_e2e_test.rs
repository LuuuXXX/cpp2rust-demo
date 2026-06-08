//! E2E-2: pugixml 端到端集成测试（简单项目）
//!
//! pugixml 是单头 + 单源的 XML 解析库，具有清晰的 `xml_document`/`xml_node`/
//! `xml_attribute` OOP API，并包含迭代器类，复杂度略高于 tinyxml2。
//!
//! 验证工具能正确处理：
//! - `xml_document`/`xml_node`/`xml_attribute` 等类的提取
//! - 迭代器类型的识别
//! - 同目录头文件 include

mod common;

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
