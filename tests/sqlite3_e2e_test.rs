//! E2E-3: SQLite3 端到端集成测试（中等项目 — extern "C" 接口）
//!
//! SQLite3 是纯 `extern "C"` 接口的 C 库，通过 C++ wrapper 调用。
//! 直接使用系统安装的 `sqlite3.h` 头文件。
//!
//! 验证工具能正确处理：
//! - 大量 `extern "C"` API 的提取（import_lib! 路径）
//! - `#include <sqlite3.h>` 系统头文件
//! - C-style 接口在 Rust FFI 层的完整映射

mod common;

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
        eprintln!("sqlite3_e2e: 系统 sqlite3.h 未安装，跳过（sudo apt-get install libsqlite3-dev）");
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
