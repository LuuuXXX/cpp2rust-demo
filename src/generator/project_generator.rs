//! Rust 项目生成器（Phase 2）
//!
//! 在 `.cpp2rust/<feature>/rust/` 下生成完整的 Cargo 项目：
//! `Cargo.toml`、`src/lib.rs`（汇总模块），每个编译单元对应一个 `src/<unit_path>.rs`。
//!
//! ## 目录结构
//!
//! 生成的 Rust 源文件目录结构与 C++ 项目保持一致，避免因文件名相同（位于不同目录）
//! 导致冲突。例如，C++ 项目中的 `src/utils/foo.cpp` 对应 Rust 侧的
//! `rust/src/src/utils/foo.rs`，`lib.rs` 及各层 `mod.rs` 会自动生成。
//!
//! `unit_path` 使用 `/` 分隔符，每个组成部分均经过 [`sanitize_mod_ident`] 处理，
//! 确保是合法的 Rust 标识符。

use crate::error::Result;
use anyhow::anyhow;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

// ─────────────────────────────────────────────
//  标识符清理
// ─────────────────────────────────────────────

/// 将路径分量转换为合法的 Rust 模块标识符：
/// 用 `_` 替换非字母数字字符，开头数字前插入 `_`。
pub fn sanitize_mod_ident(s: &str) -> String {
    if s.is_empty() {
        return "unit".to_string();
    }
    let mut result = String::with_capacity(s.len() + 1);
    let mut chars = s.chars().peekable();
    // 首字符若为数字，先插一个下划线
    if chars.peek().map_or(false, |c| c.is_ascii_digit()) {
        result.push('_');
    }
    for c in chars {
        if c.is_alphanumeric() || c == '_' {
            result.push(c);
        } else {
            result.push('_');
        }
    }
    result
}

// ─────────────────────────────────────────────
//  模块树
// ─────────────────────────────────────────────

/// 模块树节点：叶节点（unit 文件）或目录节点（含子模块）。
enum ModuleNode {
    Leaf,
    Dir(BTreeMap<String, ModuleNode>),
}

/// 将一组 `unit_path`（`/` 分隔，如 `src/utils/foo`）构建为模块树。
fn build_module_tree(unit_paths: &[String]) -> BTreeMap<String, ModuleNode> {
    let mut tree: BTreeMap<String, ModuleNode> = BTreeMap::new();
    for path in unit_paths {
        let parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
        insert_path(&mut tree, &parts);
    }
    tree
}

fn insert_path(tree: &mut BTreeMap<String, ModuleNode>, parts: &[&str]) {
    if parts.is_empty() {
        return;
    }
    if parts.len() == 1 {
        tree.entry(parts[0].to_string()).or_insert(ModuleNode::Leaf);
    } else {
        let node = tree
            .entry(parts[0].to_string())
            .or_insert_with(|| ModuleNode::Dir(BTreeMap::new()));
        if let ModuleNode::Dir(children) = node {
            insert_path(children, &parts[1..]);
        } else {
            // 节点已是 Leaf 但又需作为目录（极端冲突），发出警告
            eprintln!(
                "Warning: module path conflict at '{}': used as both a file and a directory",
                parts[0]
            );
        }
    }
}

/// 生成 `pub mod xxx;\n` 声明列表。
fn generate_mod_declarations(tree: &BTreeMap<String, ModuleNode>) -> String {
    let mut content = String::new();
    for name in tree.keys() {
        content.push_str(&format!("pub mod {};\n", name));
    }
    if content.is_empty() {
        content.push_str("// No units selected.\n");
    }
    content
}

/// 递归写出各层 `mod.rs`（仅目录节点需要）。
fn write_mod_files(src_dir: &Path, tree: &BTreeMap<String, ModuleNode>) -> Result<()> {
    for (name, node) in tree {
        if let ModuleNode::Dir(children) = node {
            let dir_path = src_dir.join(name);
            std::fs::create_dir_all(&dir_path)
                .map_err(|e| anyhow!("create dir {}: {}", dir_path.display(), e))?;
            let mod_rs_path = dir_path.join("mod.rs");
            let mod_content = generate_mod_declarations(children);
            std::fs::write(&mod_rs_path, &mod_content)
                .map_err(|e| anyhow!("write {}: {}", mod_rs_path.display(), e))?;
            write_mod_files(&dir_path, children)?;
        }
    }
    Ok(())
}

// ─────────────────────────────────────────────
//  公开 API
// ─────────────────────────────────────────────

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

/// 写出 `src/lib.rs` 及所有中间 `mod.rs`，声明模块层级与 C++ 项目目录一致。
///
/// `unit_paths` 中每个元素是相对于 `src/` 的路径（`/` 分隔，不含扩展名），
/// 例如 `"src/utils/foo"` 或扁平情形下的 `"foo"`。
pub fn write_lib_rs(rust_dir: &Path, unit_paths: &[String]) -> Result<()> {
    let src_dir = rust_dir.join("src");
    std::fs::create_dir_all(&src_dir)
        .map_err(|e| anyhow!("create src dir {}: {}", src_dir.display(), e))?;

    let tree = build_module_tree(unit_paths);
    let lib_content = generate_mod_declarations(&tree);

    let lib_rs_path = src_dir.join("lib.rs");
    std::fs::write(&lib_rs_path, &lib_content)
        .map_err(|e| anyhow!("write {}: {}", lib_rs_path.display(), e))?;

    write_mod_files(&src_dir, &tree)?;

    Ok(())
}

/// 写出 `src/<unit_path>.rs`，内容为 hicc 三段式 FFI 代码。
///
/// `unit_path` 使用 `/` 分隔符，可包含子目录，例如 `"src/utils/foo"`。
/// 函数会自动创建所需的父目录。
pub fn write_unit_rs(rust_dir: &Path, unit_path: &str, code: &str) -> Result<()> {
    let src_dir = rust_dir.join("src");
    let file_path: PathBuf = src_dir.join(format!("{}.rs", unit_path));
    if let Some(parent) = file_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| anyhow!("create dir {}: {}", parent.display(), e))?;
    }
    std::fs::write(&file_path, code).map_err(|e| anyhow!("write {}: {}", file_path.display(), e))
}

// ─────────────────────────────────────────────
//  单元测试
// ─────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    // ── sanitize_mod_ident ─────────────────────

    #[test]
    fn sanitize_plain_name() {
        assert_eq!(sanitize_mod_ident("foo"), "foo");
    }

    #[test]
    fn sanitize_hyphen_replaced() {
        assert_eq!(sanitize_mod_ident("my-lib"), "my_lib");
    }

    #[test]
    fn sanitize_leading_digit() {
        assert_eq!(sanitize_mod_ident("3d_render"), "_3d_render");
    }

    #[test]
    fn sanitize_empty_becomes_unit() {
        assert_eq!(sanitize_mod_ident(""), "unit");
    }

    #[test]
    fn sanitize_special_chars() {
        assert_eq!(sanitize_mod_ident("foo.bar"), "foo_bar");
    }

    // ── write_unit_rs ──────────────────────────

    #[test]
    fn write_unit_rs_flat() {
        let tmp = TempDir::new().unwrap();
        write_unit_rs(tmp.path(), "foo", "// content\n").unwrap();
        let p = tmp.path().join("src/foo.rs");
        assert!(p.exists());
        assert_eq!(std::fs::read_to_string(p).unwrap(), "// content\n");
    }

    #[test]
    fn write_unit_rs_nested() {
        let tmp = TempDir::new().unwrap();
        write_unit_rs(tmp.path(), "src/utils/foo", "// nested\n").unwrap();
        let p = tmp.path().join("src/src/utils/foo.rs");
        assert!(p.exists());
        assert_eq!(std::fs::read_to_string(p).unwrap(), "// nested\n");
    }

    #[test]
    fn write_unit_rs_creates_parent_dirs() {
        let tmp = TempDir::new().unwrap();
        write_unit_rs(tmp.path(), "a/b/c/deep", "x").unwrap();
        assert!(tmp.path().join("src/a/b/c/deep.rs").exists());
    }

    // ── write_lib_rs ───────────────────────────

    #[test]
    fn write_lib_rs_flat() {
        let tmp = TempDir::new().unwrap();
        write_lib_rs(tmp.path(), &["unit_a".to_string(), "unit_b".to_string()]).unwrap();
        let lib = std::fs::read_to_string(tmp.path().join("src/lib.rs")).unwrap();
        assert!(lib.contains("pub mod unit_a;"));
        assert!(lib.contains("pub mod unit_b;"));
        // 扁平模式不应生成 mod.rs
        assert!(!tmp.path().join("src/unit_a/mod.rs").exists());
    }

    #[test]
    fn write_lib_rs_empty() {
        let tmp = TempDir::new().unwrap();
        write_lib_rs(tmp.path(), &[]).unwrap();
        let lib = std::fs::read_to_string(tmp.path().join("src/lib.rs")).unwrap();
        assert!(lib.contains("// No units selected."));
    }

    #[test]
    fn write_lib_rs_nested_creates_mod_rs() {
        let tmp = TempDir::new().unwrap();
        write_lib_rs(
            tmp.path(),
            &["src/utils/foo".to_string(), "src/utils/bar".to_string(), "src/main".to_string()],
        )
        .unwrap();

        // lib.rs 只声明顶层 src
        let lib = std::fs::read_to_string(tmp.path().join("src/lib.rs")).unwrap();
        assert!(lib.contains("pub mod src;"));
        assert!(!lib.contains("pub mod utils;"));

        // src/mod.rs 声明 utils 和 main
        let src_mod = std::fs::read_to_string(tmp.path().join("src/src/mod.rs")).unwrap();
        assert!(src_mod.contains("pub mod utils;"));
        assert!(src_mod.contains("pub mod main;"));

        // src/utils/mod.rs 声明 foo 和 bar
        let utils_mod =
            std::fs::read_to_string(tmp.path().join("src/src/utils/mod.rs")).unwrap();
        assert!(utils_mod.contains("pub mod foo;"));
        assert!(utils_mod.contains("pub mod bar;"));
    }

    #[test]
    fn write_lib_rs_mixed_depth() {
        let tmp = TempDir::new().unwrap();
        write_lib_rs(
            tmp.path(),
            &["flat".to_string(), "sub/deep".to_string()],
        )
        .unwrap();

        let lib = std::fs::read_to_string(tmp.path().join("src/lib.rs")).unwrap();
        assert!(lib.contains("pub mod flat;"));
        assert!(lib.contains("pub mod sub;"));

        let sub_mod = std::fs::read_to_string(tmp.path().join("src/sub/mod.rs")).unwrap();
        assert!(sub_mod.contains("pub mod deep;"));
    }

    // ── conflict detection: same stem, different dirs ──

    #[test]
    fn no_conflict_with_different_dirs() {
        let tmp = TempDir::new().unwrap();
        // 同名文件在不同目录下不应冲突
        write_unit_rs(tmp.path(), "a/foo", "// a\n").unwrap();
        write_unit_rs(tmp.path(), "b/foo", "// b\n").unwrap();
        assert_eq!(
            std::fs::read_to_string(tmp.path().join("src/a/foo.rs")).unwrap(),
            "// a\n"
        );
        assert_eq!(
            std::fs::read_to_string(tmp.path().join("src/b/foo.rs")).unwrap(),
            "// b\n"
        );
    }
}
