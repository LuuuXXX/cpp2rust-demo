//! Rust 项目生成器（Phase 2）
//!
//! 在 `.cpp2rust/<feature>/rust/` 下生成完整的 Cargo 项目：
//! `Cargo.toml`、`src/lib.rs`（汇总模块），每个编译单元对应一个 `src/<unit_path>.rs`。
//!
//! ## 目录结构
//!
//! 生成的 Rust 源文件目录结构与 C++ 项目保持一致，避免因文件名相同（位于不同目录）
//! 导致冲突。例如，C++ 项目中的 `src/utils/foo.cpp` 对应 Rust 侧的
//! `rust/src/utils/foo.rs`，`lib.rs` 及各层 `mod.rs` 会自动生成。
//!
//! `unit_path` 由 [`derive_unit_path`] 生成，去掉了 C++ 源码根目录（如 `src/`），
//! 使用 `/` 分隔符，每个组成部分均经过 [`sanitize_mod_ident`] 处理，
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
    if chars.peek().is_some_and(|c| c.is_ascii_digit()) {
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

/// C++ 编译单元文件路径转换为 Rust 模块路径（`/` 分隔，不含扩展名）。
///
/// 转换规则：
/// 1. 去掉 `c_dir` 前缀，得到相对路径；
/// 2. 去掉 `.cpp2rust` 后缀；
/// 3. 取文件 stem（去掉 `.cpp` 等扩展名）；
/// 4. **去掉第一级路径分量**（即 C++ 源码根目录，可以是任意名称，如 `src/`、`lib/`、
///    `source/` 等），避免与 Rust crate 自身的 `src/` 目录叠加产生 `rust/src/src/…`
///    的双重路径。若文件直接位于 `c_dir` 下（无父级目录），则不做去除；
/// 5. 对每个路径分量执行 [`sanitize_mod_ident`]。
///
/// # 示例
///
/// | c_dir 内的文件                       | 结果             |
/// |-------------------------------------|------------------|
/// | `src/utils/foo.cpp.cpp2rust`        | `utils/foo`      |
/// | `lib/utils/bar.cpp.cpp2rust`        | `utils/bar`      |
/// | `src/main.cpp.cpp2rust`             | `main`           |
/// | `main.cpp.cpp2rust`（项目根）       | `main`           |
/// | `src/my-mod/foo-bar.cpp.cpp2rust`   | `my_mod/foo_bar` |
pub fn derive_unit_path(c_dir: &Path, cpp2rust_file: &Path) -> String {
    let rel = cpp2rust_file.strip_prefix(c_dir).unwrap_or(cpp2rust_file);
    let rel_str = rel.to_string_lossy();
    let after_cpp2rust = rel_str.strip_suffix(".cpp2rust").unwrap_or(&rel_str);
    let p = Path::new(after_cpp2rust);
    let stem = p.file_stem().and_then(|s| s.to_str()).unwrap_or("unit");
    let sanitized_stem = sanitize_mod_ident(stem);

    match p.parent().filter(|pp| !pp.as_os_str().is_empty()) {
        None => sanitized_stem,
        Some(parent) => {
            let mut parts: Vec<String> = parent
                .components()
                .filter_map(|c| c.as_os_str().to_str())
                .map(sanitize_mod_ident)
                .collect();
            // 去掉第一级分量（C++ 源码根目录，如 "src"），消除双重 src 问题
            if !parts.is_empty() {
                parts.remove(0);
            }
            if parts.is_empty() {
                sanitized_stem
            } else {
                format!("{}/{}", parts.join("/"), sanitized_stem)
            }
        }
    }
}

/// 在 `rust_dir` 下写出 Cargo.toml，`package.name = feature_name`。
///
/// 注：使用 edition 2018 而非 2021，以避免 Rust 2021 对 `L'\0'`（C++ 宽字符字面量）
/// 保留前缀的 lex 错误。在 hicc::cpp! 中 C++ 代码以 token stream 传入，Rust 2021
/// 会在 proc macro 执行前就报 lex error；2018 则将其 tokenize 为标识符 `L` + 字符字面量。
pub fn write_cargo_toml(rust_dir: &Path, feature_name: &str) -> Result<()> {
    let content = format!(
        r#"[package]
name = "{feature_name}"
version = "0.1.0"
edition = "2018"

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

/// 写出 `build.rs`，调用 `hicc_build::Build::new()` 完成 C++ shim 编译。
///
/// `Cargo.toml` 中已声明 `hicc-build` 为 build-dependency，
/// 必须有对应的 `build.rs` 才能触发构建脚本。
pub fn write_build_rs(rust_dir: &Path, lib_name: &str) -> Result<()> {
    let content = format!(
        "\
fn main() {{
    hicc_build::Build::new().rust_file(\"src/lib.rs\").compile(\"{lib_name}\");
}}
"
    );
    let path = rust_dir.join("build.rs");
    std::fs::write(&path, content).map_err(|e| anyhow!("write {}: {}", path.display(), e))
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
        // write_unit_rs 接受 derive_unit_path 输出的路径（不含首级目录）
        let tmp = TempDir::new().unwrap();
        write_unit_rs(tmp.path(), "utils/foo", "// nested\n").unwrap();
        let p = tmp.path().join("src/utils/foo.rs");
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
        // derive_unit_path 已去掉首级目录（"src"），故 unit_path 形如 "utils/foo"
        write_lib_rs(
            tmp.path(),
            &[
                "utils/foo".to_string(),
                "utils/bar".to_string(),
                "main".to_string(),
            ],
        )
        .unwrap();

        // lib.rs 直接声明顶层 utils 和 main（不再出现中间的 src 层）
        let lib = std::fs::read_to_string(tmp.path().join("src/lib.rs")).unwrap();
        assert!(lib.contains("pub mod utils;"));
        assert!(lib.contains("pub mod main;"));
        assert!(!lib.contains("pub mod src;"));

        // utils/mod.rs 声明 foo 和 bar
        let utils_mod = std::fs::read_to_string(tmp.path().join("src/utils/mod.rs")).unwrap();
        assert!(utils_mod.contains("pub mod foo;"));
        assert!(utils_mod.contains("pub mod bar;"));
    }

    #[test]
    fn write_lib_rs_mixed_depth() {
        let tmp = TempDir::new().unwrap();
        write_lib_rs(tmp.path(), &["flat".to_string(), "sub/deep".to_string()]).unwrap();

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

    // ── derive_unit_path ──────────────────────────

    #[test]
    fn derive_unit_path_nested_in_src() {
        let tmp = TempDir::new().unwrap();
        let c_dir = tmp.path().join("c");
        // <c_dir>/src/utils/foo.cpp.cpp2rust → "utils/foo"（去掉首级 "src"）
        let f = c_dir.join("src/utils/foo.cpp.cpp2rust");
        assert_eq!(derive_unit_path(&c_dir, &f), "utils/foo");
    }

    #[test]
    fn derive_unit_path_flat_in_src() {
        let tmp = TempDir::new().unwrap();
        let c_dir = tmp.path().join("c");
        // <c_dir>/src/main.cpp.cpp2rust → "main"（去掉首级 "src" 后无父级）
        let f = c_dir.join("src/main.cpp.cpp2rust");
        assert_eq!(derive_unit_path(&c_dir, &f), "main");
    }

    #[test]
    fn derive_unit_path_at_project_root() {
        let tmp = TempDir::new().unwrap();
        let c_dir = tmp.path().join("c");
        // <c_dir>/foo.cpp.cpp2rust → "foo"（无父级，不需要去掉）
        let f = c_dir.join("foo.cpp.cpp2rust");
        assert_eq!(derive_unit_path(&c_dir, &f), "foo");
    }

    #[test]
    fn derive_unit_path_sanitizes_idents() {
        let tmp = TempDir::new().unwrap();
        let c_dir = tmp.path().join("c");
        // 连字符、特殊字符均被替换为下划线
        let f = c_dir.join("src/my-module/foo-bar.cpp.cpp2rust");
        assert_eq!(derive_unit_path(&c_dir, &f), "my_module/foo_bar");
    }

    #[test]
    fn derive_unit_path_deep_nesting() {
        let tmp = TempDir::new().unwrap();
        let c_dir = tmp.path().join("c");
        // <c_dir>/src/a/b/c.cpp.cpp2rust → "a/b/c"（仅去掉首级 "src"）
        let f = c_dir.join("src/a/b/c.cpp.cpp2rust");
        assert_eq!(derive_unit_path(&c_dir, &f), "a/b/c");
    }

    // ── write_build_rs ────────────────────────

    #[test]
    fn write_build_rs_creates_file() {
        let tmp = TempDir::new().unwrap();
        write_build_rs(tmp.path(), "my_lib").unwrap();
        let content = std::fs::read_to_string(tmp.path().join("build.rs")).unwrap();
        assert!(content.contains("hicc_build::Build::new()"));
        assert!(content.contains(".rust_file(\"src/lib.rs\")"));
        assert!(content.contains(".compile(\"my_lib\")"));
        assert!(content.contains("fn main()"));
    }
}
