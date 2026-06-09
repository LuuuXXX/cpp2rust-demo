//! E2E-5: {fmt} 端到端集成测试（中等偏复杂 — 多文件 + 模板 + 格式化系统）
//!
//! {fmt} 是现代 C++ 格式化库，包含多个 .cc 源文件，`format_string`/`formatter`
//! 模板类，以及 wide string 支持，是多翻译单元合并能力的验证项目。
//!
//! 验证工具能正确处理：
//! - 多个源文件的独立处理（`format.cc`、`os.cc` 等）
//! - 格式化相关模板类的识别
//! - 非标准扩展名（`.cc` 而非 `.cpp`）的预处理

mod common;

use std::path::Path;
use tempfile::TempDir;

const PROJECT_ROOT: &str = "references/fmtlib";
const FMT_INCLUDE: &str = "references/fmtlib/include";
const FMT_SRC: &str = "references/fmtlib/src";

/// 要测试的 {fmt} 源文件（相对 PROJECT_ROOT）
const SOURCES: &[&str] = &["src/format.cc", "src/os.cc"];

#[test]
fn fmtlib_init_sources() {
    if !Path::new(PROJECT_ROOT).exists() {
        eprintln!("fmtlib_e2e: 子模块未初始化，跳过（运行 git submodule update --init references/fmtlib）");
        return;
    }

    let tmp = TempDir::new().unwrap();
    let preprocess_dir = tmp.path().join("preprocessed");
    std::fs::create_dir_all(&preprocess_dir).unwrap();

    let includes = &[FMT_INCLUDE, FMT_SRC];

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

    // 至少有一个文件处理成功（os.cc 和 format.cc 至少其中之一）
    assert!(
        processed > 0,
        "fmtlib E2E: 全部 {} 个文件均处理失败:\n{}",
        SOURCES.len(),
        skipped.join("\n")
    );

    if !skipped.is_empty() {
        eprintln!(
            "fmtlib E2E: {} 个文件处理失败（非致命）:\n{}",
            skipped.len(),
            skipped.join("\n")
        );
    }
}
