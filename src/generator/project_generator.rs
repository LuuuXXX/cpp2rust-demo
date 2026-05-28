//! Rust 项目生成器（Phase 2）
//!
//! 在 `.cpp2rust/<feature>/rust/` 下生成完整的 Cargo 项目：
//! `Cargo.toml`、`src/lib.rs`（汇总模块），每个编译单元对应一个 `src/<unit>.rs`。

use crate::error::Result;
use anyhow::anyhow;
use std::path::Path;

/// 在 `rust_dir` 下写出 Cargo.toml，`package.name = feature_name`。
pub fn write_cargo_toml(rust_dir: &Path, feature_name: &str) -> Result<()> {
    let content = format!(
        r#"[package]
name = "{feature_name}"
version = "0.1.0"
edition = "2021"

[lib]
name = "{lib_name}"
path = "src/lib.rs"

[dependencies]
hicc = {{ version = "0.2" }}

[build-dependencies]
hicc-build = {{ version = "0.2" }}
cc = "1.0"
"#,
        feature_name = feature_name,
        lib_name = feature_name.replace('-', "_"),
    );
    let path = rust_dir.join("Cargo.toml");
    std::fs::write(&path, content).map_err(|e| anyhow!("write {}: {}", path.display(), e))
}

/// 写出 `src/lib.rs`，声明所有单元为 `pub mod`。
pub fn write_lib_rs(rust_dir: &Path, unit_names: &[String]) -> Result<()> {
    let src_dir = rust_dir.join("src");
    std::fs::create_dir_all(&src_dir)
        .map_err(|e| anyhow!("create src dir {}: {}", src_dir.display(), e))?;

    let mut content = String::new();
    for unit in unit_names {
        content.push_str(&format!("pub mod {};\n", unit));
    }
    if content.is_empty() {
        content.push_str("// No units selected.\n");
    }

    let path = src_dir.join("lib.rs");
    std::fs::write(&path, content).map_err(|e| anyhow!("write {}: {}", path.display(), e))
}

/// 写出 `src/<unit_name>.rs`，内容为 hicc 三段式 FFI 代码。
pub fn write_unit_rs(rust_dir: &Path, unit_name: &str, code: &str) -> Result<()> {
    let src_dir = rust_dir.join("src");
    std::fs::create_dir_all(&src_dir)
        .map_err(|e| anyhow!("create src dir {}: {}", src_dir.display(), e))?;
    let path = src_dir.join(format!("{}.rs", unit_name));
    std::fs::write(&path, code).map_err(|e| anyhow!("write {}: {}", path.display(), e))
}
