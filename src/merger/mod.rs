//! Merge 命令核心逻辑（Phase 6）
//!
//! 将一个或多个 feature 下按编译单元生成的 `.rs` 文件整理为备份后的镜像输出，
//! 维持与 C++ 项目相同的目录结构。
//!
//! **单 feature 模式**：输出写回同一 feature 目录。
//!
//! ```text
//! .cpp2rust/<feature>/rust/
//!     ├── src.1/   ← 原始 init 输出的备份
//!     ├── src.2/   ← merge 输出（目录结构与 init 一致）
//!     └── src      ← symlink → src.2
//! ```
//!
//! **多 feature 模式**：将多个 feature 的编译单元聚合（去重 + 冲突检测），
//! 输出到新的合并目录（各 feature 名以下划线拼接），source feature 保持不变。
//!
//! ```text
//! .cpp2rust/<f1>_<f2>/rust/
//!     └── src/     ← 合并后的 Rust 项目（Cargo.toml、src/lib.rs、src/**/*.rs）
//! ```

pub mod block_parser;

use crate::error::Result;
use crate::layout::FeatureLayout;
use anyhow::anyhow;
use block_parser::{parse_unit_rs, ParsedFnBinding, ParsedUnit};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

// ─────────────────────────────────────────────
//  合并后的中间结构
// ─────────────────────────────────────────────

/// 跨所有 feature / unit 合并后的 FFI 规格
#[derive(Default)]
pub struct MergedSpec {
    /// hicc::cpp! 块的有序内容行（已去重）
    pub cpp_lines: Vec<String>,
    /// 每个类名对应的合并后方法列表
    pub classes: HashMap<String, Vec<ParsedMethod>>,
    /// 每个类名对应的完整属性行（如 `#[cpp(class = "Foo")]` / `#[interface]`）
    pub class_attrs: HashMap<String, String>,
    /// 类名出现顺序（保持稳定输出）
    pub class_order: Vec<String>,
    /// import_lib! 中的前向声明（已去重）
    pub fwd_decls: Vec<String>,
    /// import_lib! 中的函数绑定（已去重）
    pub fn_bindings: Vec<ParsedFnBinding>,
    /// 冲突警告列表
    pub conflicts: Vec<String>,
}

/// 已合并的单个方法
#[derive(Clone)]
pub struct ParsedMethod {
    /// `#[cpp(method = "...")]` 属性行
    pub attr: String,
    /// `fn foo(...);` 行
    pub fn_sig: String,
}

// ─────────────────────────────────────────────
//  主入口：从文件列表合并
// ─────────────────────────────────────────────

/// 收集指定 feature 下所有 unit `.rs` 文件，优先取 `src.1/`（merge 后的备份目录），
/// 不存在时回退到 `src/`（init 直接输出目录）。
///
/// 这一规则与 `run_merge` 中的单 feature 逻辑保持一致：`src.1/` 是经过备份的 init 输出，
/// 保证在重复运行 merge 时始终从原始输出中读取。
pub fn collect_feature_unit_rs_files(layout: &FeatureLayout) -> Vec<PathBuf> {
    let canonical_src = if layout.rust_dir.join("src.1").is_dir() {
        layout.rust_dir.join("src.1")
    } else {
        layout.rust_dir.join("src")
    };
    if canonical_src.exists() {
        collect_unit_rs_files(&canonical_src)
    } else {
        vec![]
    }
}

/// 合并多个 unit `.rs` 文件到一个 `MergedSpec`。
pub fn merge_units(unit_rs_paths: &[std::path::PathBuf]) -> MergedSpec {
    let mut spec = MergedSpec::default();
    let mut cpp_line_seen: HashSet<String> = HashSet::new();
    // (cpp_sig → rust fn line)：冲突检测
    let mut fn_attr_to_sig: HashMap<String, String> = HashMap::new();
    let mut fwd_decl_seen: HashSet<String> = HashSet::new();
    // (class_name, method_attr) → fn_sig：方法去重 & 冲突检测
    let mut method_seen: HashMap<(String, String), String> = HashMap::new();

    for path in unit_rs_paths {
        let src = match std::fs::read_to_string(path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("  Warning: cannot read {}: {}", path.display(), e);
                continue;
            }
        };

        let unit = parse_unit_rs(&src);
        merge_cpp_lines(&mut spec, &unit, &mut cpp_line_seen);
        merge_classes(&mut spec, &unit, &mut method_seen);
        merge_lib(&mut spec, &unit, &mut fn_attr_to_sig, &mut fwd_decl_seen);
    }

    spec
}

fn merge_cpp_lines(spec: &mut MergedSpec, unit: &ParsedUnit, seen: &mut HashSet<String>) {
    for line in &unit.cpp_lines {
        // include 行去重；shim 函数行保留全部（可能有相同内容，但简单起见也去重）
        if !seen.contains(line) {
            seen.insert(line.clone());
            spec.cpp_lines.push(line.clone());
        }
    }
}

fn merge_classes(
    spec: &mut MergedSpec,
    unit: &ParsedUnit,
    method_seen: &mut HashMap<(String, String), String>,
) {
    for cb in &unit.class_blocks {
        if !spec.classes.contains_key(&cb.class_name) {
            spec.class_order.push(cb.class_name.clone());
            spec.classes.insert(cb.class_name.clone(), Vec::new());
            // 首次遇到时记录完整属性行
            if !cb.class_attr.is_empty() {
                spec.class_attrs
                    .insert(cb.class_name.clone(), cb.class_attr.clone());
            }
        }
        let methods = spec.classes.get_mut(&cb.class_name).unwrap();
        for method in &cb.methods {
            let key = (cb.class_name.clone(), method.attr.clone());
            if let Some(existing_sig) = method_seen.get(&key) {
                if existing_sig != &method.fn_sig {
                    spec.conflicts.push(format!(
                        "Class {} method conflict:\n  existing: {}\n  new:      {}",
                        cb.class_name, existing_sig, method.fn_sig
                    ));
                }
                // 已存在，跳过
            } else {
                method_seen.insert(key, method.fn_sig.clone());
                methods.push(ParsedMethod {
                    attr: method.attr.clone(),
                    fn_sig: method.fn_sig.clone(),
                });
            }
        }
    }
}

fn merge_lib(
    spec: &mut MergedSpec,
    unit: &ParsedUnit,
    fn_attr_to_sig: &mut HashMap<String, String>,
    fwd_decl_seen: &mut HashSet<String>,
) {
    if let Some(lib) = &unit.lib_block {
        // 前向声明去重
        for decl in &lib.fwd_decls {
            if fwd_decl_seen.insert(decl.clone()) {
                spec.fwd_decls.push(decl.clone());
            }
        }
        // 函数绑定去重 & 冲突检测
        for fb in &lib.fn_bindings {
            if let Some(existing_sig) = fn_attr_to_sig.get(&fb.attr) {
                if existing_sig != &fb.fn_sig {
                    spec.conflicts.push(format!(
                        "Function binding conflict:\n  attr:     {}\n  existing: {}\n  new:      {}",
                        fb.attr, existing_sig, fb.fn_sig
                    ));
                }
                // 已存在，跳过
            } else {
                fn_attr_to_sig.insert(fb.attr.clone(), fb.fn_sig.clone());
                spec.fn_bindings.push(fb.clone());
            }
        }
    }
}

// ─────────────────────────────────────────────
//  代码生成：MergedSpec → Rust 源代码字符串
// ─────────────────────────────────────────────

/// 将合并后的规格生成 Rust 源文件内容。
/// `link_name`：`import_lib!` 中的 `#![link_name = "..."]` 值。
pub fn emit_merged_rs(spec: &MergedSpec, link_name: &str) -> String {
    let mut out = String::new();

    // ── hicc::cpp! ──────────────────────────────
    out.push_str(&crate::generator::hicc_codegen::emit_cpp_block(
        &spec.cpp_lines,
    ));

    // ── hicc::import_class! (每类一个块) ────────
    for class_name in &spec.class_order {
        let methods = match spec.classes.get(class_name) {
            Some(m) => m,
            None => continue,
        };
        if methods.is_empty() {
            continue;
        }
        // 使用解析时记录的完整属性行，正确保留 destroy= 和 #[interface]
        let default_attr = format!("#[cpp(class = \"{}\")]", class_name);
        let attr_line = spec
            .class_attrs
            .get(class_name)
            .map(|s| s.as_str())
            .unwrap_or(&default_attr);
        out.push('\n');
        out.push_str("hicc::import_class! {\n");
        out.push_str(&format!("    {}\n", attr_line));
        out.push_str(&format!("    class {} {{\n", class_name));
        for m in methods {
            out.push_str(&format!("        {}\n", m.attr));
            out.push_str(&format!("        {}\n", m.fn_sig));
            out.push('\n');
        }
        // 去掉最后一个方法后多余的空行
        if out.ends_with("\n\n") {
            out.pop();
        }
        out.push_str("    }\n");
        out.push_str("}\n");
    }

    // ── hicc::import_lib! ───────────────────────
    let has_lib_content = !spec.fwd_decls.is_empty() || !spec.fn_bindings.is_empty();
    if has_lib_content {
        out.push('\n');
        out.push_str("hicc::import_lib! {\n");
        out.push_str(&format!("    #![link_name = \"{}\"]\n", link_name));

        if !spec.fwd_decls.is_empty() {
            out.push('\n');
            for decl in &spec.fwd_decls {
                out.push_str(&format!("    {}\n", decl));
            }
        }

        for fb in &spec.fn_bindings {
            out.push('\n');
            out.push_str(&format!("    {}\n", fb.attr));
            out.push_str(&format!("    {}\n", fb.fn_sig));
        }

        out.push_str("}\n");
    }

    out
}

/// 递归扫描 `src_dir` 下所有 `*.rs` 文件，返回路径列表（排序）。
/// 排除 `lib.rs`（汇总模块）和 `mod.rs`（目录声明文件），只返回实际 unit 文件。
pub fn collect_unit_rs_files(src_dir: &Path) -> Vec<std::path::PathBuf> {
    let mut result = Vec::new();
    collect_unit_rs_recursive(src_dir, &mut result);
    result.sort();
    result
}

fn collect_unit_rs_recursive(dir: &Path, result: &mut Vec<std::path::PathBuf>) {
    let rd = match std::fs::read_dir(dir) {
        Ok(r) => r,
        Err(_) => return,
    };
    for entry in rd.flatten() {
        let p = entry.path();
        if p.is_dir() {
            collect_unit_rs_recursive(&p, result);
        } else if p.extension().and_then(|e| e.to_str()) == Some("rs") {
            let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if name != "lib.rs" && name != "mod.rs" {
                result.push(p);
            }
        }
    }
}

// ─────────────────────────────────────────────
//  目录操作：copy_dir_all + merge_in_place
// ─────────────────────────────────────────────

/// 递归复制目录 `src` 的全部内容到 `dst`（`dst` 不必预先存在）。
pub fn copy_dir_all(src: &Path, dst: &Path) -> Result<()> {
    std::fs::create_dir_all(dst).map_err(|e| anyhow!("create dir {}: {}", dst.display(), e))?;
    for entry in std::fs::read_dir(src).map_err(|e| anyhow!("read dir {}: {}", src.display(), e))? {
        let entry = entry.map_err(|e| anyhow!("read entry: {}", e))?;
        let from = entry.path();
        let to = dst.join(entry.file_name());
        if from.is_dir() {
            copy_dir_all(&from, &to)?;
        } else {
            std::fs::copy(&from, &to)
                .map_err(|e| anyhow!("copy {} → {}: {}", from.display(), to.display(), e))?;
        }
    }
    Ok(())
}

/// 将 `rust_dir/src`（init 输出）整理为带备份的 merge 输出：
///
/// - 首次运行：`src/` → rename → `src.1/`；复制 `src.1/` → `src.2/`；建 symlink `src → src.2`
/// - 重复运行：删除旧 symlink；重新复制 `src.1/` → `src.2/`；重建 symlink `src → src.2`
///
/// 目录结构始终维持与 C++ 项目一致的子目录层级。
pub fn merge_in_place(rust_dir: &Path) -> Result<()> {
    let src = rust_dir.join("src");
    let src1 = rust_dir.join("src.1");
    let src2 = rust_dir.join("src.2");

    // ── 确定 canonical（init 原始输出）来源 ──
    let canonical_src: std::path::PathBuf = if src1.is_dir() {
        // 已有备份（重复运行）：直接从 src.1 读取
        src1.clone()
    } else if src.is_dir() && !src.is_symlink() {
        // 首次运行：src 是真实目录
        src.clone()
    } else {
        return Err(anyhow!(
            "rust/src not found at {}; run init first",
            src.display()
        ));
    };

    // ── 清理旧的 src.2（重复运行时覆写）──
    if src2.exists() || src2.is_symlink() {
        std::fs::remove_dir_all(&src2).map_err(|e| anyhow!("remove {}: {}", src2.display(), e))?;
    }

    // ── 复制 canonical → src.2（维持目录结构）──
    copy_dir_all(&canonical_src, &src2)?;

    // ── 处理 src ──
    if src.is_symlink() {
        // 重复运行：删除旧 symlink
        std::fs::remove_file(&src)
            .map_err(|e| anyhow!("remove symlink {}: {}", src.display(), e))?;
    } else if src.is_dir() {
        // 首次运行：备份 src → src.1
        if src1.exists() {
            std::fs::remove_dir_all(&src1)
                .map_err(|e| anyhow!("remove {}: {}", src1.display(), e))?;
        }
        std::fs::rename(&src, &src1)
            .map_err(|e| anyhow!("rename {} → {}: {}", src.display(), src1.display(), e))?;
    }

    // ── 建 symlink src → src.2（相对路径）──
    #[cfg(unix)]
    std::os::unix::fs::symlink("src.2", &src).map_err(|e| anyhow!("symlink src → src.2: {}", e))?;
    #[cfg(not(unix))]
    return Err(anyhow!(
        "merge_in_place requires symlink support, which is only available on Unix-like systems \
         (Linux, macOS); Windows is not supported"
    ));

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merge_deduplicates_includes() {
        let src1 = r#"hicc::cpp! {
    #include "foo.h"
    #include "bar.h"
}
"#;
        let src2 = r#"hicc::cpp! {
    #include "foo.h"
    #include "baz.h"
}
"#;
        let dir = tempfile::TempDir::new().unwrap();
        let p1 = dir.path().join("unit1.rs");
        let p2 = dir.path().join("unit2.rs");
        std::fs::write(&p1, src1).unwrap();
        std::fs::write(&p2, src2).unwrap();

        let spec = merge_units(&[p1, p2]);
        // foo.h 应只出现一次
        let foo_count = spec
            .cpp_lines
            .iter()
            .filter(|l| l.contains("foo.h"))
            .count();
        assert_eq!(foo_count, 1);
        // bar.h 和 baz.h 各出现一次
        assert!(spec.cpp_lines.iter().any(|l| l.contains("bar.h")));
        assert!(spec.cpp_lines.iter().any(|l| l.contains("baz.h")));
    }

    #[test]
    fn merge_deduplicates_class_methods() {
        let src = r#"hicc::cpp! {
    #include "foo.h"
}

hicc::import_class! {
    #[cpp(class = "Foo")]
    class Foo {
        #[cpp(method = "int get() const")]
        fn get(&self) -> i32;

    }
}
"#;
        let dir = tempfile::TempDir::new().unwrap();
        let p1 = dir.path().join("unit1.rs");
        let p2 = dir.path().join("unit2.rs");
        std::fs::write(&p1, src).unwrap();
        std::fs::write(&p2, src).unwrap();

        let spec = merge_units(&[p1, p2]);
        let foo_methods = spec.classes.get("Foo").unwrap();
        assert_eq!(foo_methods.len(), 1, "duplicate method should be deduped");
    }

    #[test]
    fn merge_deduplicates_fn_bindings() {
        let src = r#"hicc::cpp! {
    #include "foo.h"
}

hicc::import_lib! {
    #![link_name = "foo"]

    class Foo;

    #[cpp(func = "Foo* foo_new()")]
    fn foo_new() -> *mut Foo;
}
"#;
        let dir = tempfile::TempDir::new().unwrap();
        let p1 = dir.path().join("unit1.rs");
        let p2 = dir.path().join("unit2.rs");
        std::fs::write(&p1, src).unwrap();
        std::fs::write(&p2, src).unwrap();

        let spec = merge_units(&[p1, p2]);
        assert_eq!(
            spec.fn_bindings.len(),
            1,
            "duplicate fn binding should be deduped"
        );
        assert_eq!(
            spec.fwd_decls.len(),
            1,
            "duplicate fwd_decl should be deduped"
        );
    }

    // ── collect_unit_rs_files ──────────────────

    #[test]
    fn collect_unit_rs_files_flat() {
        let dir = tempfile::TempDir::new().unwrap();
        std::fs::write(dir.path().join("unit1.rs"), "").unwrap();
        std::fs::write(dir.path().join("unit2.rs"), "").unwrap();
        std::fs::write(dir.path().join("lib.rs"), "").unwrap();

        let files = collect_unit_rs_files(dir.path());
        assert_eq!(files.len(), 2);
        assert!(files.iter().any(|p| p.ends_with("unit1.rs")));
        assert!(files.iter().any(|p| p.ends_with("unit2.rs")));
    }

    #[test]
    fn collect_unit_rs_files_excludes_mod_rs() {
        let dir = tempfile::TempDir::new().unwrap();
        let sub = dir.path().join("sub");
        std::fs::create_dir_all(&sub).unwrap();
        std::fs::write(sub.join("mod.rs"), "").unwrap();
        std::fs::write(sub.join("foo.rs"), "").unwrap();

        let files = collect_unit_rs_files(dir.path());
        assert_eq!(files.len(), 1);
        assert!(files[0].ends_with("foo.rs"));
    }

    #[test]
    fn collect_unit_rs_files_recursive() {
        let dir = tempfile::TempDir::new().unwrap();
        // 模拟层级结构：src/utils/foo.rs, src/main.rs, flat.rs
        let utils = dir.path().join("src").join("utils");
        std::fs::create_dir_all(&utils).unwrap();
        std::fs::write(utils.join("foo.rs"), "").unwrap();
        std::fs::write(dir.path().join("src").join("main.rs"), "").unwrap();
        std::fs::write(dir.path().join("flat.rs"), "").unwrap();
        std::fs::write(dir.path().join("lib.rs"), "").unwrap();
        std::fs::write(dir.path().join("src").join("mod.rs"), "").unwrap();

        let files = collect_unit_rs_files(dir.path());
        assert_eq!(files.len(), 3, "should find foo.rs, main.rs, flat.rs");
    }

    #[test]
    fn collect_unit_rs_files_same_stem_different_dirs() {
        // 验证同名文件位于不同子目录时均能被找到（无冲突）
        let dir = tempfile::TempDir::new().unwrap();
        let a = dir.path().join("a");
        let b = dir.path().join("b");
        std::fs::create_dir_all(&a).unwrap();
        std::fs::create_dir_all(&b).unwrap();
        std::fs::write(a.join("foo.rs"), "// a").unwrap();
        std::fs::write(b.join("foo.rs"), "// b").unwrap();

        let files = collect_unit_rs_files(dir.path());
        assert_eq!(files.len(), 2);
    }

    // ── copy_dir_all ───────────────────────────

    #[test]
    fn copy_dir_all_copies_flat_files() {
        let tmp = tempfile::TempDir::new().unwrap();
        let src = tmp.path().join("src");
        let dst = tmp.path().join("dst");
        std::fs::create_dir(&src).unwrap();
        std::fs::write(src.join("lib.rs"), "// lib").unwrap();
        std::fs::write(src.join("foo.rs"), "// foo").unwrap();

        copy_dir_all(&src, &dst).unwrap();

        assert_eq!(
            std::fs::read_to_string(dst.join("lib.rs")).unwrap(),
            "// lib"
        );
        assert_eq!(
            std::fs::read_to_string(dst.join("foo.rs")).unwrap(),
            "// foo"
        );
    }

    #[test]
    fn copy_dir_all_preserves_subdirectory_structure() {
        let tmp = tempfile::TempDir::new().unwrap();
        let src = tmp.path().join("src");
        let utils = src.join("utils");
        std::fs::create_dir_all(&utils).unwrap();
        std::fs::write(src.join("lib.rs"), "// lib").unwrap();
        std::fs::write(utils.join("foo.rs"), "// foo").unwrap();

        let dst = tmp.path().join("dst");
        copy_dir_all(&src, &dst).unwrap();

        assert!(dst.join("lib.rs").exists());
        assert!(dst.join("utils").is_dir());
        assert_eq!(
            std::fs::read_to_string(dst.join("utils/foo.rs")).unwrap(),
            "// foo"
        );
    }

    // ── merge_in_place ─────────────────────────

    #[test]
    fn merge_in_place_creates_backup_and_symlink() {
        let tmp = tempfile::TempDir::new().unwrap();
        let rust_dir = tmp.path().to_path_buf();
        let src = rust_dir.join("src");
        std::fs::create_dir(&src).unwrap();
        std::fs::write(src.join("lib.rs"), "// lib").unwrap();

        merge_in_place(&rust_dir).unwrap();

        // src.1 是 init 输出的备份
        assert!(
            rust_dir.join("src.1").is_dir(),
            "src.1 should be backup dir"
        );
        assert!(rust_dir.join("src.1/lib.rs").exists());
        // src.2 是 merge 输出
        assert!(
            rust_dir.join("src.2").is_dir(),
            "src.2 should be merge output"
        );
        assert!(rust_dir.join("src.2/lib.rs").exists());
        // src 是 symlink
        assert!(rust_dir.join("src").is_symlink(), "src should be a symlink");
        // symlink 可正常访问
        assert!(rust_dir.join("src/lib.rs").exists());
    }

    #[test]
    fn merge_in_place_maintains_directory_structure() {
        let tmp = tempfile::TempDir::new().unwrap();
        let rust_dir = tmp.path().to_path_buf();
        let src = rust_dir.join("src");
        let utils = src.join("utils");
        std::fs::create_dir_all(&utils).unwrap();
        std::fs::write(src.join("lib.rs"), "// lib").unwrap();
        std::fs::write(utils.join("foo.rs"), "// foo").unwrap();

        merge_in_place(&rust_dir).unwrap();

        // 子目录结构在 src.2 中保留
        assert!(
            rust_dir.join("src.2/utils/foo.rs").exists(),
            "subdirectory structure preserved"
        );
        // 通过 symlink 可正常访问
        assert!(rust_dir.join("src/utils/foo.rs").exists());
    }

    #[test]
    fn merge_in_place_rerun_updates_symlink_keeps_src1() {
        let tmp = tempfile::TempDir::new().unwrap();
        let rust_dir = tmp.path().to_path_buf();
        let src = rust_dir.join("src");
        std::fs::create_dir(&src).unwrap();
        std::fs::write(src.join("lib.rs"), "// original").unwrap();

        // 首次 merge
        merge_in_place(&rust_dir).unwrap();
        // 重复运行
        merge_in_place(&rust_dir).unwrap();

        // src.1 仍保留 init 原始内容
        let content = std::fs::read_to_string(rust_dir.join("src.1/lib.rs")).unwrap();
        assert_eq!(
            content, "// original",
            "src.1 should retain original init output"
        );
        // src.2 正常存在
        assert!(rust_dir.join("src.2/lib.rs").exists());
        // src 仍是 symlink
        assert!(rust_dir.join("src").is_symlink());
    }

    #[test]
    fn merge_in_place_errors_when_src_missing() {
        let tmp = tempfile::TempDir::new().unwrap();
        // rust_dir 下既没有 src 也没有 src.1
        let result = merge_in_place(tmp.path());
        assert!(result.is_err(), "should error when src does not exist");
    }

    // ── collect_feature_unit_rs_files ──────────

    #[test]
    fn collect_feature_unit_rs_files_prefers_src1() {
        let tmp = tempfile::TempDir::new().unwrap();
        let layout = crate::layout::FeatureLayout::new(tmp.path().to_path_buf(), "feat");
        layout.create_dirs().unwrap();

        // 建立 src.1/（merge 后的备份）和 src/（init 输出）
        let src1 = layout.rust_dir.join("src.1");
        let src = layout.rust_dir.join("src");
        std::fs::create_dir_all(&src1).unwrap();
        std::fs::create_dir_all(&src).unwrap();
        std::fs::write(src1.join("from_src1.rs"), "").unwrap();
        std::fs::write(src.join("from_src.rs"), "").unwrap();

        let files = collect_feature_unit_rs_files(&layout);
        // 应只从 src.1 取，不包含 src/ 中的文件
        assert_eq!(files.len(), 1);
        assert!(files[0].ends_with("from_src1.rs"));
    }

    #[test]
    fn collect_feature_unit_rs_files_falls_back_to_src() {
        let tmp = tempfile::TempDir::new().unwrap();
        let layout = crate::layout::FeatureLayout::new(tmp.path().to_path_buf(), "feat");
        layout.create_dirs().unwrap();

        // 只有 src/，没有 src.1/
        let src = layout.rust_dir.join("src");
        std::fs::create_dir_all(&src).unwrap();
        std::fs::write(src.join("unit_a.rs"), "").unwrap();
        std::fs::write(src.join("unit_b.rs"), "").unwrap();

        let files = collect_feature_unit_rs_files(&layout);
        assert_eq!(files.len(), 2);
    }

    #[test]
    fn collect_feature_unit_rs_files_returns_empty_when_neither_exists() {
        let tmp = tempfile::TempDir::new().unwrap();
        let layout = crate::layout::FeatureLayout::new(tmp.path().to_path_buf(), "feat");
        layout.create_dirs().unwrap();
        // rust_dir 下没有 src 也没有 src.1
        let files = collect_feature_unit_rs_files(&layout);
        assert!(files.is_empty());
    }
}
