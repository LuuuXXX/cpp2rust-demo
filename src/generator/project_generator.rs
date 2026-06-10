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
                "警告：模块路径冲突 '{}'：该节点既是文件也被当作目录使用",
                parts[0]
            );
        }
    }
}

/// 生成 `pub mod xxx;\npub use self::xxx::*;\n` 声明列表。
///
/// 同时生成重新导出（`pub use self::xxx::*`），使各 unit 模块可通过
/// `use crate::*;` 访问兄弟模块中定义的类型（如跨文件的 `hicc::import_class!` 类型引用）。
fn generate_mod_declarations(tree: &BTreeMap<String, ModuleNode>) -> String {
    let mut content = String::new();
    for name in tree.keys() {
        content.push_str(&format!("pub mod {};\n", name));
        content.push_str(&format!("pub use self::{}::*;\n", name));
    }
    if content.is_empty() {
        content.push_str("// 未选择任何单元。\n");
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
/// 4. **仅当第一级路径分量名为 `src` 时将其去掉**，以避免与 Rust crate 自身的 `src/`
///    目录叠加产生 `rust/src/src/…` 的双重路径。其他名称（`tests/`、`shim/`、`lib/`、
///    `example/` 等）完整保留，使 Rust `src/` 下的目录结构与 C++ 项目保持一致。
///    若文件直接位于 `c_dir` 下（无父级目录），则不做去除；
/// 5. 对每个路径分量执行 [`sanitize_mod_ident`]。
///
/// # 示例
///
/// | c_dir 内的文件                            | 结果                  |
/// |------------------------------------------|-----------------------|
/// | `src/utils/foo.cpp.cpp2rust`             | `utils/foo`           |
/// | `src/main.cpp.cpp2rust`                  | `main`                |
/// | `main.cpp.cpp2rust`（项目根）            | `main`                |
/// | `src/my-mod/foo-bar.cpp.cpp2rust`        | `my_mod/foo_bar`      |
/// | `tests/bar.cpp.cpp2rust`                 | `tests/bar`           |
/// | `shim/baz.cpp.cpp2rust`                  | `shim/baz`            |
/// | `lib/utils/bar.cpp.cpp2rust`             | `lib/utils/bar`       |
/// | `shim/allocators/alloc.cpp.cpp2rust`     | `shim/allocators/alloc` |
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
            // 仅去掉名为 "src" 的第一级分量，消除 rust/src/src/… 双重路径问题。
            // 其他目录名（tests、shim、lib 等）完整保留，与 C++ 项目结构保持一致。
            if parts.first().map(|s| s.as_str()) == Some("src") {
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
/// 使用 edition 2021，`cc` 作为唯一 build-dependency 直接编译 C++ shim 文件。
/// hicc 依赖已移除，生成的 Rust FFI 代码使用标准 `extern "C"` 块，无跨平台兼容性问题。
pub fn write_cargo_toml(rust_dir: &Path, feature_name: &str) -> Result<()> {
    let content = format!(
        r#"[package]
name = "{feature_name}"
version = "0.1.0"
edition = "2021"

[lib]
name = "{lib_name}"
path = "src/lib.rs"

[build-dependencies]
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

/// 写出 `tests/smoke_test.rs`，内容为冒烟测试代码。
///
/// Cargo 自动识别 `tests/*.rs` 为集成测试，无需在 `Cargo.toml` 中显式声明。
pub fn write_smoke_test(rust_dir: &Path, content: &str) -> Result<()> {
    let tests_dir = rust_dir.join("tests");
    std::fs::create_dir_all(&tests_dir)
        .map_err(|e| anyhow!("create tests dir {}: {}", tests_dir.display(), e))?;
    let path = tests_dir.join("smoke_test.rs");
    std::fs::write(&path, content).map_err(|e| anyhow!("write {}: {}", path.display(), e))
}

/// 计算从 `base_dir`（绝对路径）到 `target`（绝对路径）的相对路径。
///
/// 如规范化失败，则原样返回 `target`（绝对路径）作为保底。
fn relative_from(base_dir: &Path, target: &Path) -> PathBuf {
    let base = base_dir
        .canonicalize()
        .unwrap_or_else(|_| base_dir.to_path_buf());
    let tgt = target
        .canonicalize()
        .unwrap_or_else(|_| target.to_path_buf());

    let base_comps: Vec<_> = base.components().collect();
    let tgt_comps: Vec<_> = tgt.components().collect();

    let common = base_comps
        .iter()
        .zip(tgt_comps.iter())
        .take_while(|(a, b)| a == b)
        .count();

    let up_count = base_comps.len() - common;
    let mut result = PathBuf::new();
    for _ in 0..up_count {
        result.push("..");
    }
    for c in tgt_comps[common..].iter() {
        result.push(c);
    }
    if result.as_os_str().is_empty() {
        result.push(".");
    }
    result
}

/// 将路径转换为正斜杠字符串，Windows 下避免反斜杠在字符串字面量中引起歧义。
fn path_to_fwd_slash(p: &Path) -> String {
    p.to_string_lossy().replace('\\', "/")
}

/// 写出 `build.rs`，调用 `cc::Build` 完成 C++ shim 编译。
///
/// * `cpp_sources`   — 原始 C++ shim 源文件的绝对路径列表，会通过 `build.file()` 编译进库。
/// * `include_dirs`  — 编译 C++ shim 所需的头文件搜索路径（绝对路径），来自 `.opts` 文件。
///
/// 生成的 `build.rs` 使用 `cc::Build` 直接编译 C++ 源文件，无需 hicc 桥接宏：
/// 1. 创建 `cc::Build`，开启 C++ 模式，添加 include 目录和源文件；
/// 2. 调用 `build.compile(lib_name)` 将全部代码编译进静态库（cc 自动 emit link 指令）；
/// 3. 手动 emit C++ 标准库链接指令（macOS 链接 `c++`，Linux 等链接 `stdc++`，MSVC 自动处理）。
///
/// `unit_rs_paths` 参数保留以维持函数签名向后兼容，当前实现中不使用（extern "C" 绑定
/// 不需要 hicc_build 扫描 .rs 文件生成 C++ 桥接代码）。
pub fn write_build_rs(
    rust_dir: &Path,
    lib_name: &str,
    unit_rs_paths: &[String],
    cpp_sources: &[PathBuf],
    include_dirs: &[PathBuf],
) -> Result<()> {
    let _ = unit_rs_paths; // extern "C" 模式不需要 rust_file() 调用

    // 将绝对路径转换为相对于 rust_dir 的相对路径，以保证生成项目在同一仓库内的可移植性
    let rel_includes: Vec<String> = include_dirs
        .iter()
        .map(|d| path_to_fwd_slash(&relative_from(rust_dir, d)))
        .collect();
    let rel_sources: Vec<String> = cpp_sources
        .iter()
        .map(|s| path_to_fwd_slash(&relative_from(rust_dir, s)))
        .collect();

    // include 目录和 C++ 源文件配置行
    let mut setup_lines = String::new();
    for inc in &rel_includes {
        setup_lines.push_str(&format!("    build.include(\"{inc}\");\n"));
    }
    for src in &rel_sources {
        setup_lines.push_str(&format!("    build.file(\"{src}\");\n"));
    }

    let content = format!(
        "\
fn main() {{
    let mut build = cc::Build::new();
    build.cpp(true);
{setup_lines}    build.compile(\"{lib_name}\");

    #[cfg(target_os = \"macos\")]
    println!(\"cargo::rustc-link-lib=c++\");
    #[cfg(not(any(target_os = \"macos\", all(target_os = \"windows\", target_env = \"msvc\"))))]
    println!(\"cargo::rustc-link-lib=stdc++\");
}}
"
    );
    let path = rust_dir.join("build.rs");
    std::fs::write(&path, content).map_err(|e| anyhow!("write {}: {}", path.display(), e))
}

// ─────────────────────────────────────────────
//  多 feature 合并项目生成
// ─────────────────────────────────────────────

/// 为多 feature 合并项目写出 Cargo.toml。
///
/// `combined_name` 为各 feature 名称以 `_` 连接的组合名称（如 `feat1_feat2`），
/// 用作 `package.name` 和 `[lib] name`。
/// 生成的项目在 `[features]` 中列出每个 feature，
/// 支持 `cargo build --features <feature>` 按需构建对应代码。
pub fn write_multi_feature_cargo_toml(
    rust_dir: &Path,
    combined_name: &str,
    feature_names: &[&str],
) -> Result<()> {
    let lib_name = combined_name.replace('-', "_");
    let features_section = feature_names
        .iter()
        .map(|f| format!("{} = []", f))
        .collect::<Vec<_>>()
        .join("\n");

    let content = format!(
        r#"[package]
name = "{combined_name}"
version = "0.1.0"
edition = "2021"

[lib]
name = "{lib_name}"
path = "src/lib.rs"

[features]
{features_section}

[build-dependencies]
cc = "1.0"
"#,
        combined_name = combined_name,
        lib_name = lib_name,
        features_section = features_section,
    );
    let path = rust_dir.join("Cargo.toml");
    std::fs::write(&path, content).map_err(|e| anyhow!("write {}: {}", path.display(), e))
}

/// 为多 feature 合并项目写出 `src/lib.rs`。
///
/// 每个 feature 对应一个条件编译的顶层模块：
/// ```rust
/// #[cfg(feature = "feat")]
/// pub mod feat;
/// ```
pub fn write_multi_feature_lib_rs(rust_dir: &Path, feature_names: &[&str]) -> Result<()> {
    let src_dir = rust_dir.join("src");
    std::fs::create_dir_all(&src_dir)
        .map_err(|e| anyhow!("create src dir {}: {}", src_dir.display(), e))?;

    let mut content = String::new();
    for feature in feature_names {
        content.push_str(&format!(
            "#[cfg(feature = \"{feature}\")]\npub mod {feature};\n"
        ));
    }
    if content.is_empty() {
        content.push_str("// 未选择任何 feature。\n");
    }

    let lib_rs_path = src_dir.join("lib.rs");
    std::fs::write(&lib_rs_path, &content)
        .map_err(|e| anyhow!("write {}: {}", lib_rs_path.display(), e))
}

/// 为多 feature 合并项目写出 `build.rs`。
///
/// 每个 feature 对应一个条件编译的 `cc::Build` 调用，负责编译该 feature 的 C++ 源文件。
/// 注：C++ 源文件路径需在合并后手动填入（`cpp2rust-todo[BUILD]` 标记处）。
pub fn write_multi_feature_build_rs(rust_dir: &Path, feature_names: &[&str]) -> Result<()> {
    let mut body = String::new();
    for feature in feature_names {
        let lib_name = feature.replace('-', "_");
        body.push_str(&format!(
            "    if cfg!(feature = \"{feature}\") {{\n\
             \x20       // cpp2rust-todo[BUILD]: 添加 C++ 源文件路径，例如：\n\
             \x20       // cc::Build::new().cpp(true).file(\"path/to/src.cpp\").compile(\"{lib_name}\");\n\
             \x20       let _ = \"{lib_name}\"; // 占位，替换为实际 cc::Build 调用\n\
             \x20   }}\n"
        ));
    }
    let content = format!("fn main() {{\n{body}}}\n", body = body);
    let path = rust_dir.join("build.rs");
    std::fs::write(&path, content).map_err(|e| anyhow!("write {}: {}", path.display(), e))
}

/// 将单个 feature 的源文件复制到多 feature 合并项目的子模块目录下。
///
/// - `feature_src_dir`：feature 的 `rust/src/` 目录（初始化输出）
/// - `dest_dir`：目标 `src/<feature>/` 目录
/// - `feature_name`：feature 名称，用于将 `use crate::` 重写为 `use crate::<feature>::`
///
/// `lib.rs` 将被复制为 `mod.rs`；其余文件保持原名不变。
/// 所有 `.rs` 文件中的 `use crate::` 将被替换为 `use crate::<feature>::`，
/// 以适应嵌套在 feature 模块下的新路径结构。
pub fn copy_feature_src_to_module(
    feature_src_dir: &Path,
    dest_dir: &Path,
    feature_name: &str,
) -> Result<()> {
    copy_feature_src_recursive(feature_src_dir, dest_dir, feature_name)
}

fn copy_feature_src_recursive(src: &Path, dst: &Path, feature_name: &str) -> Result<()> {
    std::fs::create_dir_all(dst).map_err(|e| anyhow!("create dir {}: {}", dst.display(), e))?;
    for entry in std::fs::read_dir(src).map_err(|e| anyhow!("read dir {}: {}", src.display(), e))? {
        let entry = entry.map_err(|e| anyhow!("read entry: {}", e))?;
        let from = entry.path();
        let file_name = entry.file_name();
        let file_name_str = file_name.to_string_lossy();

        if from.is_dir() {
            let to = dst.join(&*file_name_str);
            copy_feature_src_recursive(&from, &to, feature_name)?;
        } else if from.extension().and_then(|e| e.to_str()) == Some("rs") {
            // lib.rs → mod.rs（成为 feature 子模块的入口）
            let dest_name = if file_name_str == "lib.rs" {
                "mod.rs".to_string()
            } else {
                file_name_str.into_owned()
            };
            let to = dst.join(&dest_name);
            let content = std::fs::read_to_string(&from)
                .map_err(|e| anyhow!("read {}: {}", from.display(), e))?;
            // 逐行处理：仅对以 `use crate::` 开头的实际 use 语句重写路径，
            // 避免误替换注释或字符串字面量中的 `use crate::` 文本。
            let rewritten: String = content
                .lines()
                .map(|line| {
                    let trimmed = line.trim_start();
                    if trimmed.starts_with("use crate::") {
                        line.replacen("use crate::", &format!("use crate::{}::", feature_name), 1)
                    } else {
                        line.to_string()
                    }
                })
                .collect::<Vec<_>>()
                .join("\n");
            // 保留原文件末尾换行符
            let rewritten = if content.ends_with('\n') {
                format!("{}\n", rewritten)
            } else {
                rewritten
            };
            std::fs::write(&to, rewritten).map_err(|e| anyhow!("write {}: {}", to.display(), e))?;
        }
    }
    Ok(())
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

    /// lib.rs 应同时生成 `pub use self::xxx::*;`，使跨模块类型可见
    #[test]
    fn write_lib_rs_flat_includes_pub_use_reexports() {
        let tmp = TempDir::new().unwrap();
        write_lib_rs(tmp.path(), &["unit_a".to_string(), "unit_b".to_string()]).unwrap();
        let lib = std::fs::read_to_string(tmp.path().join("src/lib.rs")).unwrap();
        assert!(
            lib.contains("pub use self::unit_a::*;"),
            "lib.rs 应包含 pub use self::unit_a::*;\n实际内容:\n{}",
            lib
        );
        assert!(
            lib.contains("pub use self::unit_b::*;"),
            "lib.rs 应包含 pub use self::unit_b::*;\n实际内容:\n{}",
            lib
        );
    }

    #[test]
    fn write_lib_rs_empty() {
        let tmp = TempDir::new().unwrap();
        write_lib_rs(tmp.path(), &[]).unwrap();
        let lib = std::fs::read_to_string(tmp.path().join("src/lib.rs")).unwrap();
        assert!(lib.contains("// 未选择任何单元。"));
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
        // lib.rs 同时应有重新导出
        assert!(lib.contains("pub use self::utils::*;"));
        assert!(lib.contains("pub use self::main::*;"));

        // utils/mod.rs 声明 foo 和 bar，并重新导出
        let utils_mod = std::fs::read_to_string(tmp.path().join("src/utils/mod.rs")).unwrap();
        assert!(utils_mod.contains("pub mod foo;"));
        assert!(utils_mod.contains("pub mod bar;"));
        assert!(utils_mod.contains("pub use self::foo::*;"));
        assert!(utils_mod.contains("pub use self::bar::*;"));
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
        // <c_dir>/src/utils/foo.cpp.cpp2rust → "utils/foo"（"src" 被去掉）
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
        // <c_dir>/foo.cpp.cpp2rust → "foo"（无父级，不做去除）
        let f = c_dir.join("foo.cpp.cpp2rust");
        assert_eq!(derive_unit_path(&c_dir, &f), "foo");
    }

    #[test]
    fn derive_unit_path_sanitizes_idents() {
        let tmp = TempDir::new().unwrap();
        let c_dir = tmp.path().join("c");
        // 连字符、特殊字符均被替换为下划线；"src" 被去掉
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

    #[test]
    fn derive_unit_path_tests_dir_preserved() {
        let tmp = TempDir::new().unwrap();
        let c_dir = tmp.path().join("c");
        // <c_dir>/tests/bar.cpp.cpp2rust → "tests/bar"（非 "src"，完整保留）
        let f = c_dir.join("tests/bar.cpp.cpp2rust");
        assert_eq!(derive_unit_path(&c_dir, &f), "tests/bar");
    }

    #[test]
    fn derive_unit_path_shim_dir_preserved() {
        let tmp = TempDir::new().unwrap();
        let c_dir = tmp.path().join("c");
        // <c_dir>/shim/foo.cpp.cpp2rust → "shim/foo"（非 "src"，完整保留）
        let f = c_dir.join("shim/foo.cpp.cpp2rust");
        assert_eq!(derive_unit_path(&c_dir, &f), "shim/foo");
    }

    #[test]
    fn derive_unit_path_lib_dir_preserved() {
        let tmp = TempDir::new().unwrap();
        let c_dir = tmp.path().join("c");
        // <c_dir>/lib/utils/bar.cpp.cpp2rust → "lib/utils/bar"（非 "src"，完整保留）
        let f = c_dir.join("lib/utils/bar.cpp.cpp2rust");
        assert_eq!(derive_unit_path(&c_dir, &f), "lib/utils/bar");
    }

    #[test]
    fn derive_unit_path_non_src_deep_nesting_preserved() {
        let tmp = TempDir::new().unwrap();
        let c_dir = tmp.path().join("c");
        // <c_dir>/shim/a/b/c.cpp.cpp2rust → "shim/a/b/c"（非 "src"，多级完整保留）
        let f = c_dir.join("shim/a/b/c.cpp.cpp2rust");
        assert_eq!(derive_unit_path(&c_dir, &f), "shim/a/b/c");
    }

    // ── write_build_rs ────────────────────────

    #[test]
    fn write_build_rs_creates_file() {
        let tmp = TempDir::new().unwrap();
        write_build_rs(tmp.path(), "my_lib", &[], &[], &[]).unwrap();
        let content = std::fs::read_to_string(tmp.path().join("build.rs")).unwrap();
        assert!(content.contains("cc::Build::new()"), "应包含 cc::Build::new()");
        assert!(content.contains("build.compile(\"my_lib\")"), "应包含 build.compile(\"my_lib\")");
        assert!(content.contains("fn main()"), "应包含 fn main()");
        // 不再手动 emit link 指令（cc 自动 emit），但仍 emit C++ 标准库
        assert!(
            content.contains("cargo::rustc-link-lib=stdc++") || content.contains("cargo::rustc-link-lib=c++"),
            "应包含 C++ 标准库链接指令"
        );
        // 不应包含 hicc_build 或 rust_file 调用
        assert!(!content.contains("hicc_build"), "不应含 hicc_build");
        assert!(!content.contains("rust_file"), "不应含 rust_file（无编译单元）");
    }

    #[test]
    fn write_build_rs_with_units_and_cpp() {
        let tmp = TempDir::new().unwrap();
        // 用 tmp.path() 本身充当伪 C++ 源文件及 include 目录（不需要实际存在的内容）
        let cpp_src = tmp.path().join("foo.cpp");
        std::fs::write(&cpp_src, "").unwrap();
        let inc_dir = tmp.path().to_path_buf();

        write_build_rs(
            tmp.path(),
            "rj",
            &["allocator_ffi".to_string(), "shim/doc_ffi".to_string()],
            &[cpp_src],
            &[inc_dir],
        )
        .unwrap();
        let content = std::fs::read_to_string(tmp.path().join("build.rs")).unwrap();
        // extern "C" 模式不需要 rust_file 调用
        assert!(!content.contains("rust_file"), "extern \"C\" 模式不应含 rust_file");
        assert!(content.contains("build.file("), "应含 build.file()");
        assert!(content.contains("build.include("), "应含 build.include()");
        assert!(content.contains("build.cpp(true)"), "应含 build.cpp(true)");
        // cc::Build::compile 自动 emit 静态库 link 指令，不需要手动 println
        assert!(!content.contains("cargo::rustc-link-lib=rj"), "cc::Build 自动处理库链接，不应手动 emit");
    }

    // ── write_cargo_toml ──────────────────────

    #[test]
    fn write_cargo_toml_contains_cc_build_dep() {
        let tmp = TempDir::new().unwrap();
        write_cargo_toml(tmp.path(), "my_feature").unwrap();
        let content = std::fs::read_to_string(tmp.path().join("Cargo.toml")).unwrap();
        assert!(
            content.contains("name = \"my_feature\""),
            "package.name 应为 my_feature"
        );
        assert!(
            content.contains("cc = \"1.0\""),
            "应包含 cc build-dependency"
        );
        assert!(
            content.contains("[build-dependencies]"),
            "应包含 [build-dependencies] 段"
        );
        // 不应包含 hicc 依赖（已移除）
        assert!(
            !content.contains("hicc"),
            "不应包含 hicc 依赖（已改为 extern \"C\" + cc）"
        );
        assert!(
            content.contains("edition = \"2021\""),
            "应使用 edition 2021"
        );
    }

    // ── 多 feature 生成 ────────────────────────

    #[test]
    fn write_multi_feature_cargo_toml_contains_features() {
        let tmp = TempDir::new().unwrap();
        write_multi_feature_cargo_toml(tmp.path(), "feat1_feat2", &["feat1", "feat2"]).unwrap();
        let content = std::fs::read_to_string(tmp.path().join("Cargo.toml")).unwrap();
        assert!(content.contains("[features]"));
        assert!(content.contains("feat1 = []"));
        assert!(content.contains("feat2 = []"));
        assert!(content.contains("name = \"feat1_feat2\""));
        assert!(
            content.contains("cc = \"1.0\""),
            "多 feature Cargo.toml 应包含 cc build-dependency"
        );
        // 不应包含 hicc 依赖
        assert!(
            !content.contains("hicc"),
            "不应包含 hicc 依赖（已改为 extern \"C\" + cc）"
        );
    }

    #[test]
    fn write_multi_feature_cargo_toml_hyphen_to_underscore_lib_name() {
        // combined_name 中含连字符时，[lib] name 应将 '-' 替换为 '_'
        let tmp = TempDir::new().unwrap();
        write_multi_feature_cargo_toml(tmp.path(), "my-feat_other", &["my-feat", "other"]).unwrap();
        let content = std::fs::read_to_string(tmp.path().join("Cargo.toml")).unwrap();
        assert!(
            content.contains("name = \"my-feat_other\""),
            "package.name 应保留原始连字符"
        );
        assert!(
            content.contains("name = \"my_feat_other\""),
            "[lib] name 应将连字符替换为下划线"
        );
    }

    #[test]
    fn write_multi_feature_lib_rs_conditional_mods() {
        let tmp = TempDir::new().unwrap();
        write_multi_feature_lib_rs(tmp.path(), &["feat1", "feat2"]).unwrap();
        let content = std::fs::read_to_string(tmp.path().join("src/lib.rs")).unwrap();
        assert!(content.contains("#[cfg(feature = \"feat1\")]"));
        assert!(content.contains("pub mod feat1;"));
        assert!(content.contains("#[cfg(feature = \"feat2\")]"));
        assert!(content.contains("pub mod feat2;"));
    }

    #[test]
    fn write_multi_feature_build_rs_per_feature_blocks() {
        let tmp = TempDir::new().unwrap();
        write_multi_feature_build_rs(tmp.path(), &["feat1", "feat2"]).unwrap();
        let content = std::fs::read_to_string(tmp.path().join("build.rs")).unwrap();
        assert!(content.contains("cfg!(feature = \"feat1\")"));
        assert!(content.contains("\"feat1\""));
        assert!(content.contains("cfg!(feature = \"feat2\")"));
        assert!(content.contains("\"feat2\""));
        assert!(content.contains("fn main()"));
        // 不应包含 hicc_build（已移除）
        assert!(!content.contains("hicc_build"), "不应含 hicc_build");
    }

    #[test]
    fn write_multi_feature_build_rs_hyphen_to_underscore() {
        let tmp = TempDir::new().unwrap();
        write_multi_feature_build_rs(tmp.path(), &["my-feat"]).unwrap();
        let content = std::fs::read_to_string(tmp.path().join("build.rs")).unwrap();
        // lib_name 含下划线（连字符已替换）
        assert!(content.contains("my_feat"), "连字符应替换为下划线");
    }

    #[test]
    fn copy_feature_src_to_module_lib_becomes_mod() {
        let tmp = TempDir::new().unwrap();
        let src = tmp.path().join("src");
        std::fs::create_dir_all(&src).unwrap();
        std::fs::write(src.join("lib.rs"), "pub mod foo;\n").unwrap();
        std::fs::write(src.join("foo.rs"), "// foo\n").unwrap();

        let dest = tmp.path().join("out/feat1");
        copy_feature_src_to_module(&src, &dest, "feat1").unwrap();

        assert!(dest.join("mod.rs").exists(), "lib.rs should become mod.rs");
        assert!(!dest.join("lib.rs").exists());
        assert!(dest.join("foo.rs").exists());
    }

    #[test]
    fn copy_feature_src_to_module_rewrites_use_crate() {
        let tmp = TempDir::new().unwrap();
        let src = tmp.path().join("src");
        std::fs::create_dir_all(&src).unwrap();
        // use 语句应被重写；注释行不应被修改
        std::fs::write(
            src.join("lib.rs"),
            "use crate::utils::Foo;\n// use crate::old_code\n",
        )
        .unwrap();

        let dest = tmp.path().join("out/feat1");
        copy_feature_src_to_module(&src, &dest, "feat1").unwrap();

        let content = std::fs::read_to_string(dest.join("mod.rs")).unwrap();
        assert!(
            content.contains("use crate::feat1::utils::Foo;"),
            "use crate:: should be rewritten to use crate::feat1::"
        );
        assert!(
            content.contains("// use crate::old_code"),
            "comment lines should not be rewritten"
        );
    }

    #[test]
    fn copy_feature_src_to_module_preserves_subdirs() {
        let tmp = TempDir::new().unwrap();
        let src = tmp.path().join("src");
        let sub = src.join("utils");
        std::fs::create_dir_all(&sub).unwrap();
        std::fs::write(src.join("lib.rs"), "pub mod utils;\n").unwrap();
        std::fs::write(sub.join("mod.rs"), "pub mod bar;\n").unwrap();
        std::fs::write(sub.join("bar.rs"), "// bar\n").unwrap();

        let dest = tmp.path().join("out/feat1");
        copy_feature_src_to_module(&src, &dest, "feat1").unwrap();

        assert!(dest.join("mod.rs").exists());
        assert!(dest.join("utils/mod.rs").exists());
        assert!(dest.join("utils/bar.rs").exists());
    }
}
