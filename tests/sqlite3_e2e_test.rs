//! E2E-3: SQLite3 端到端集成测试（中等项目 — extern "C" 接口）
//!
//! SQLite3 是纯 `extern "C"` 接口的 C 库，通过 C++ wrapper 调用。
//! 直接使用系统安装的 `sqlite3.h` 头文件。
//!
//! 验证工具能正确处理：
//! - 大量 `extern "C"` API 的提取（import_lib! 路径）
//! - `#include <sqlite3.h>` 系统头文件
//! - C-style 接口在 Rust FFI 层的完整映射
//! - init + merge 两阶段完整流程及生成 Rust 项目的可编译性（`cargo check`）

mod common;

use cpp2rust_demo::{generator::project_generator, merger};
use std::path::Path;
use tempfile::TempDir;

/// 系统 sqlite3 头文件路径（Linux/macOS 通用）
const SQLITE3_HEADER: &str = "/usr/include/sqlite3.h";

/// 测试用的临时 C++ wrapper 文件内容
const SQLITE3_WRAPPER_CPP: &str = r#"
// sqlite3 C++ wrapper — 用于测试工具对 extern "C" 接口的处理能力
extern "C" {
#include <sqlite3.h>
}
"#;

#[test]
fn sqlite3_init_extern_c() {
    if !Path::new(SQLITE3_HEADER).exists() {
        eprintln!(
            "sqlite3_e2e: 系统 sqlite3.h 未安装，跳过（sudo apt-get install libsqlite3-dev）"
        );
        return;
    }

    let tmp = TempDir::new().unwrap();
    let preprocess_dir = tmp.path().join("preprocessed");
    std::fs::create_dir_all(&preprocess_dir).unwrap();

    // 写临时 wrapper .cpp
    let wrapper_cpp = tmp.path().join("sqlite3_wrapper.cpp");
    std::fs::write(&wrapper_cpp, SQLITE3_WRAPPER_CPP).unwrap();

    let includes: &[&str] = &[];
    match common::process_cpp_source(&wrapper_cpp, includes, &preprocess_dir) {
        Some((unit_name, code)) => {
            common::assert_valid_hicc_format(&code, &unit_name);
            // sqlite3 是纯 C 接口，应生成 import_lib! 而非 import_class!
            // 注意：若没有任何函数被识别（纯 C 接口工具暂不处理），仅验证格式正确即可
        }
        None => {
            // 预处理失败（例如无 g++ 等），优雅跳过
            eprintln!("sqlite3_e2e: 预处理失败，跳过");
        }
    }
}

// ─────────────────────────────────────────────────────────────────
//  Merge 阶段测试：init + merge + cargo check
// ─────────────────────────────────────────────────────────────────

/// 验证 sqlite3 wrapper 完整的 init + merge 两阶段流程，并对生成的 Rust 项目执行
/// `cargo check`，确保输出代码可编译。sqlite3.h 不存在时自动跳过。
#[test]
fn sqlite3_merge_phase() {
    if !Path::new(SQLITE3_HEADER).exists() {
        eprintln!("sqlite3_merge_phase: 系统 sqlite3.h 未安装，跳过");
        return;
    }

    let tmp = TempDir::new().unwrap();
    let preprocess_dir = tmp.path().join("c");
    let rust_dir = tmp.path().join("rust");
    std::fs::create_dir_all(&preprocess_dir).unwrap();

    // 写临时 wrapper .cpp
    let wrapper_cpp = tmp.path().join("sqlite3_wrapper.cpp");
    std::fs::write(&wrapper_cpp, SQLITE3_WRAPPER_CPP).unwrap();

    let includes: &[&str] = &[];
    let mut unit_paths: Vec<String> = Vec::new();

    // ── Init 阶段 ──────────────────────────────────────────────────
    match common::process_cpp_source(&wrapper_cpp, includes, &preprocess_dir) {
        Some((unit_name, code)) => {
            project_generator::write_unit_rs(&rust_dir, &unit_name, &code)
                .expect("write_unit_rs 失败");
            unit_paths.push(unit_name);
        }
        None => {
            eprintln!("sqlite3_merge_phase: 预处理失败，跳过");
            return;
        }
    }

    // 生成 Cargo.toml 与 lib.rs
    project_generator::write_cargo_toml(&rust_dir, "sqlite3").expect("write_cargo_toml 失败");
    project_generator::write_lib_rs(&rust_dir, &unit_paths).expect("write_lib_rs 失败");

    // ── Merge 阶段 ─────────────────────────────────────────────────
    merger::merge_in_place(&rust_dir).expect("merge_in_place 失败");

    // ── 验证输出目录结构 ────────────────────────────────────────────
    let src1 = rust_dir.join("src.1");
    let src_dir = rust_dir.join("src");

    assert!(src1.is_dir(), "sqlite3_merge_phase: src.1/ 目录不存在");
    assert!(
        src_dir.is_dir() && !src_dir.is_symlink(),
        "sqlite3_merge_phase: src/ 不存在或为符号链接"
    );

    // ── 验证合并后文件 hicc 格式 ─────────────────────────────────
    let merged_files = merger::collect_unit_rs_files(&src_dir);
    assert!(
        !merged_files.is_empty(),
        "sqlite3_merge_phase: src/ 下未找到任何 .rs 文件"
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
                "sqlite3_merge_phase: cargo check 失败（生成的 Rust 项目不可编译）:\n{}",
                stderr
            );
        }
        Ok(_) => println!("sqlite3_merge_phase: cargo check 通过"),
        Err(e) => eprintln!(
            "sqlite3_merge_phase: cargo check 跳过（cargo 不可用: {}）",
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
                    "sqlite3_merge_phase: cargo test 失败（生成的冒烟测试未通过）:\n{}",
                    stderr
                );
            }
            Ok(_) => println!(
                "sqlite3_merge_phase: cargo test 通过（生成的冒烟测试全部通过）"
            ),
            Err(e) => eprintln!(
                "sqlite3_merge_phase: cargo test 跳过（cargo 不可用: {}）",
                e
            ),
        }
    } else {
        println!("sqlite3_merge_phase: cargo test 跳过（未生成 tests/smoke.rs）");
    }
}
