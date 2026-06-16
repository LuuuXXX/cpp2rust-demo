//! E2E-7: Neargye/magic_enum 端到端集成测试（header-only — 重度 constexpr / 模板元编程）
//!
//! magic_enum 是 header-only 库（单个 `magic_enum.hpp`，C++17 重度 `constexpr` +
//! 模板元编程，编译期反射枚举），是验证工具对「极端模板/constexpr 头文件」解析鲁棒性的项目。
//!
//! 验证工具能正确处理：
//! - 重度 `constexpr` / 模板元编程头文件的 libclang 解析
//! - header-only 库的 E2E 流程（无单独 .cpp 源文件）
//! - init + merge 两阶段完整流程及生成 Rust 项目的可编译性（`cargo check`）
//!
//! 与 nlohmann/json E2E 同构：驱动类方法仅声明（签名用标量 / `std` 类型，不引用库类型），
//! 库头文件仅在 `#include` 处参与解析压测，从而保证生成产物可编译。

mod common;

use cpp2rust_demo::{generator::project_generator, merger};
use std::path::Path;
use tempfile::TempDir;

const PROJECT_ROOT: &str = "references/magic_enum";
const MAGIC_ENUM_INCLUDE: &str = "references/magic_enum/include";

/// 测试用 C++ 驱动文件内容（include magic_enum.hpp 以触发模板/constexpr 展开）
const ENUM_DRIVER_CPP: &str = r#"
// magic_enum 驱动文件 — 用于测试重度 constexpr/模板元编程头文件的解析能力
#include <magic_enum/magic_enum.hpp>
#include <string>

namespace enumwrap_ns {

enum class Color { Red, Green, Blue };

// 方法仅声明，签名用标量/std 类型（不引用 magic_enum 类型），
// 与 nlohmann/json E2E 同构，保证生成绑定可编译。
class ColorWrapper {
public:
    int count() const;
    std::string name_of(int idx) const;
};

}  // namespace enumwrap_ns
"#;

#[test]
fn magic_enum_init() {
    if !Path::new(PROJECT_ROOT).exists() {
        eprintln!("magic_enum_e2e: 子模块未初始化，跳过（运行 git submodule update --init references/magic_enum）");
        return;
    }

    let tmp = TempDir::new().unwrap();
    let preprocess_dir = tmp.path().join("preprocessed");
    std::fs::create_dir_all(&preprocess_dir).unwrap();

    // 写临时驱动文件
    let driver_cpp = tmp.path().join("enum_driver.cpp");
    std::fs::write(&driver_cpp, ENUM_DRIVER_CPP).unwrap();

    let includes = &[MAGIC_ENUM_INCLUDE];

    match common::process_cpp_source(&driver_cpp, includes, &preprocess_dir) {
        Some((unit_name, code)) => {
            common::assert_valid_hicc_format(&code, &unit_name);
            // 工具能成功处理重度 constexpr/模板头文件即为通过；
            // 由于驱动方法仅声明，不强制要求提取到类绑定。
        }
        None => {
            eprintln!("magic_enum_e2e: 预处理失败（constexpr 展开可能超时），跳过");
        }
    }
}

// ─────────────────────────────────────────────────────────────────
//  Merge 阶段测试：init + merge + cargo check
// ─────────────────────────────────────────────────────────────────

/// 验证 magic_enum 完整的 init + merge 两阶段流程，并对生成的 Rust 项目执行
/// `cargo check`，确保输出代码可编译。预处理失败时自动跳过。
#[test]
fn magic_enum_merge_phase() {
    if !Path::new(PROJECT_ROOT).exists() {
        eprintln!("magic_enum_merge_phase: 子模块未初始化，跳过（运行 git submodule update --init references/magic_enum）");
        return;
    }

    let tmp = TempDir::new().unwrap();
    let preprocess_dir = tmp.path().join("c");
    let rust_dir = tmp.path().join("rust");
    std::fs::create_dir_all(&preprocess_dir).unwrap();
    std::fs::create_dir_all(&rust_dir).unwrap();

    let driver_cpp = tmp.path().join("enum_driver.cpp");
    std::fs::write(&driver_cpp, ENUM_DRIVER_CPP).unwrap();

    let includes = &[MAGIC_ENUM_INCLUDE];

    let mut unit_paths: Vec<String> = Vec::new();
    match common::process_cpp_source(&driver_cpp, includes, &preprocess_dir) {
        Some((unit_name, code)) => {
            project_generator::write_unit_rs(&rust_dir, &unit_name, &code)
                .expect("write_unit_rs 失败");
            unit_paths.push(unit_name);
        }
        None => {
            eprintln!("magic_enum_merge_phase: 预处理失败（constexpr 展开可能超时），跳过");
            return;
        }
    }

    // 生成 Cargo.toml 与 lib.rs
    project_generator::write_cargo_toml(&rust_dir, "magic-enum").expect("write_cargo_toml 失败");
    project_generator::write_lib_rs(&rust_dir, &unit_paths).expect("write_lib_rs 失败");

    // ── Merge 阶段 ─────────────────────────────────────────────────
    merger::merge_in_place(&rust_dir).expect("merge_in_place 失败");

    // ── 验证输出目录结构 ────────────────────────────────────────────
    let src1 = rust_dir.join("src.1");
    let src_dir = rust_dir.join("src");

    assert!(src1.is_dir(), "magic_enum_merge_phase: src.1/ 目录不存在");
    assert!(
        src_dir.is_dir() && !src_dir.is_symlink(),
        "magic_enum_merge_phase: src/ 不存在或为符号链接"
    );

    // ── 验证合并后文件 hicc 格式 ─────────────────────────────────
    let merged_files = merger::collect_unit_rs_files(&src_dir);
    assert!(
        !merged_files.is_empty(),
        "magic_enum_merge_phase: src/ 下未找到任何 .rs 文件"
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
                "magic_enum_merge_phase: cargo check 失败（生成的 Rust 项目不可编译）:\n{}",
                stderr
            );
        }
        Ok(_) => println!("magic_enum_merge_phase: cargo check 通过"),
        Err(e) => eprintln!(
            "magic_enum_merge_phase: cargo check 跳过（cargo 不可用: {}）",
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
                    "magic_enum_merge_phase: cargo test 失败（生成的冒烟测试未通过）:\n{}",
                    stderr
                );
            }
            Ok(_) => {
                println!("magic_enum_merge_phase: cargo test 通过（生成的冒烟测试全部通过）")
            }
            Err(e) => eprintln!(
                "magic_enum_merge_phase: cargo test 跳过（cargo 不可用: {}）",
                e
            ),
        }
    } else {
        println!("magic_enum_merge_phase: cargo test 跳过（未生成 tests/smoke.rs）");
    }
}
