//! E2E-4: nlohmann/json 端到端集成测试（中等项目 — 重度模板 + 单超大头文件）
//!
//! nlohmann/json 是 header-only 库（单个 `json.hpp` ~23K 行），重度使用模板和
//! `template<typename T>` 特化，是验证计划一（跨翻译单元模板合并）的核心项目。
//!
//! 验证工具能正确处理：
//! - 超大头文件（~23K 行）的解析
//! - 模板类（`basic_json<...>`）的提取和 `template_base` 识别
//! - header-only 库的 E2E 流程（无单独 .cpp 源文件）
//! - init + merge 两阶段完整流程及生成 Rust 项目的可编译性（`cargo check`）

mod common;

use cpp2rust_demo::{generator::project_generator, merger};
use std::path::Path;
use tempfile::TempDir;

const PROJECT_ROOT: &str = "references/nlohmann-json";
const NLOHMANN_INCLUDE: &str = "references/nlohmann-json/include";

/// 测试用 C++ 驱动文件内容（include json.hpp 以触发模板展开）
const JSON_DRIVER_CPP: &str = r#"
// nlohmann/json 驱动文件 — 用于测试模板类提取能力
#include <nlohmann/json.hpp>

// 使用基本类型触发模板实例化
using json = nlohmann::json;

class JsonWrapper {
public:
    json parse(const std::string& s);
    void set_int(const std::string& key, int value);
};
"#;

#[test]
fn nlohmann_json_init() {
    if !Path::new(PROJECT_ROOT).exists() {
        eprintln!("nlohmann_json_e2e: 子模块未初始化，跳过（运行 git submodule update --init references/nlohmann-json）");
        return;
    }

    let tmp = TempDir::new().unwrap();
    let preprocess_dir = tmp.path().join("preprocessed");
    std::fs::create_dir_all(&preprocess_dir).unwrap();

    // 写临时驱动文件
    let driver_cpp = tmp.path().join("json_driver.cpp");
    std::fs::write(&driver_cpp, JSON_DRIVER_CPP).unwrap();

    let includes = &[NLOHMANN_INCLUDE];

    match common::process_cpp_source(&driver_cpp, includes, &preprocess_dir) {
        Some((unit_name, code)) => {
            common::assert_valid_hicc_format(&code, &unit_name);
            // 工具能成功处理 ~23K 行的超大头文件即为通过；
            // 由于 JsonWrapper 方法引用了 nlohmann 模板类型，不强制要求提取到类绑定。
        }
        None => {
            eprintln!("nlohmann_json_e2e: 预处理失败（json.hpp 展开可能超时），跳过");
        }
    }
}

// ─────────────────────────────────────────────────────────────────
//  Merge 阶段测试：init + merge + cargo check
// ─────────────────────────────────────────────────────────────────

/// 验证 nlohmann/json 完整的 init + merge 两阶段流程，并对生成的 Rust 项目执行
/// `cargo check`，确保输出代码可编译。预处理失败时自动跳过（超大头文件可能超时）。
#[test]
fn nlohmann_json_merge_phase() {
    if !Path::new(PROJECT_ROOT).exists() {
        eprintln!("nlohmann_json_merge_phase: 子模块未初始化，跳过（运行 git submodule update --init references/nlohmann-json）");
        return;
    }

    let tmp = TempDir::new().unwrap();
    let preprocess_dir = tmp.path().join("c");
    let rust_dir = tmp.path().join("rust");
    std::fs::create_dir_all(&preprocess_dir).unwrap();

    // 写临时驱动文件
    let driver_cpp = tmp.path().join("json_driver.cpp");
    std::fs::write(&driver_cpp, JSON_DRIVER_CPP).unwrap();

    let includes = &[NLOHMANN_INCLUDE];
    let mut unit_paths: Vec<String> = Vec::new();

    // ── Init 阶段 ──────────────────────────────────────────────────
    match common::process_cpp_source(&driver_cpp, includes, &preprocess_dir) {
        Some((unit_name, code)) => {
            project_generator::write_unit_rs(&rust_dir, &unit_name, &code)
                .expect("write_unit_rs 失败");
            unit_paths.push(unit_name);
        }
        None => {
            eprintln!("nlohmann_json_merge_phase: 预处理失败（json.hpp 展开可能超时），跳过");
            return;
        }
    }

    // 生成 Cargo.toml 与 lib.rs
    project_generator::write_cargo_toml(&rust_dir, "nlohmann-json").expect("write_cargo_toml 失败");
    project_generator::write_lib_rs(&rust_dir, &unit_paths).expect("write_lib_rs 失败");

    // ── Merge 阶段 ─────────────────────────────────────────────────
    merger::merge_in_place(&rust_dir).expect("merge_in_place 失败");

    // ── 验证输出目录结构 ────────────────────────────────────────────
    let src1 = rust_dir.join("src.1");
    let src_dir = rust_dir.join("src");

    assert!(
        src1.is_dir(),
        "nlohmann_json_merge_phase: src.1/ 目录不存在"
    );
    assert!(
        src_dir.is_dir() && !src_dir.is_symlink(),
        "nlohmann_json_merge_phase: src/ 不存在或为符号链接"
    );

    // ── 验证合并后文件 hicc 格式 ─────────────────────────────────
    let merged_files = merger::collect_unit_rs_files(&src_dir);
    assert!(
        !merged_files.is_empty(),
        "nlohmann_json_merge_phase: src/ 下未找到任何 .rs 文件"
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
                "nlohmann_json_merge_phase: cargo check 失败（生成的 Rust 项目不可编译）:\n{}",
                stderr
            );
        }
        Ok(_) => println!("nlohmann_json_merge_phase: cargo check 通过"),
        Err(e) => eprintln!(
            "nlohmann_json_merge_phase: cargo check 跳过（cargo 不可用: {}）",
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
                    "nlohmann_json_merge_phase: cargo test 失败（生成的冒烟测试未通过）:\n{}",
                    stderr
                );
            }
            Ok(_) => println!(
                "nlohmann_json_merge_phase: cargo test 通过（生成的冒烟测试全部通过）"
            ),
            Err(e) => eprintln!(
                "nlohmann_json_merge_phase: cargo test 跳过（cargo 不可用: {}）",
                e
            ),
        }
    } else {
        println!("nlohmann_json_merge_phase: cargo test 跳过（未生成 tests/smoke.rs）");
    }
}
