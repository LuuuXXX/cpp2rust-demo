//! Merge 命令核心逻辑（Phase 6）
//!
//! 将一个 feature 下按编译单元生成的 `.rs` 文件整理为备份后的镜像输出，
//! 维持与 C++ 项目相同的目录结构。
//!
//! 输出结构（写回同一 feature 目录）：
//! ```text
//! .cpp2rust/<feature>/rust/
//!     ├── src.1/   ← 原始 init 输出的备份
//!     └── src/     ← merge 输出（真实目录，跨平台）
//! ```

pub mod block_parser;

use crate::error::Result;
use anyhow::anyhow;
use block_parser::{parse_unit_rs, ParsedFnBinding, ParsedUnit};
use std::collections::{HashMap, HashSet};
use std::path::Path;
use walkdir::WalkDir;

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
    /// 跨翻译单元去重后保留的模板类体（规范化字符串，供报告使用）
    pub template_bodies: Vec<String>,
    /// 模板特化分组：base template name → 特化类名列表
    /// 例如 `"Stack"` → `["Stack<int>", "Stack<double>"]`
    pub template_groups: HashMap<String, Vec<String>>,
    /// 含 `cpp2rust-todo` 降级标记的 C++ 签名集合（在 merge_units 中顺带收集，无需二次读文件）
    pub degraded_sigs: HashSet<String>,
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

/// 合并多个 unit `.rs` 文件到一个 `MergedSpec`。
///
/// 在读取每个文件时顺带收集降级签名（`degraded_sigs`），避免二次 I/O。
///
/// 返回 `(MergedSpec, Vec<String>)`：
/// - `MergedSpec`：合并后的 FFI 规格
/// - `Vec<String>`：读取失败的文件路径列表（含错误信息），由调用方决定是警告还是报错
pub fn merge_units(unit_rs_paths: &[std::path::PathBuf]) -> (MergedSpec, Vec<String>) {
    let mut spec = MergedSpec::default();
    let mut read_errors: Vec<String> = Vec::new();
    let mut cpp_line_seen: HashSet<String> = HashSet::new();
    let mut template_body_seen: HashSet<String> = HashSet::new();
    // (cpp_sig → rust fn line)：冲突检测
    let mut fn_attr_to_sig: HashMap<String, String> = HashMap::new();
    let mut fwd_decl_seen: HashSet<String> = HashSet::new();
    // (class_name, method_attr) → fn_sig：方法去重 & 冲突检测
    let mut method_seen: HashMap<(String, String), String> = HashMap::new();

    for path in unit_rs_paths {
        let src = match std::fs::read_to_string(path) {
            Ok(s) => s,
            Err(e) => {
                read_errors.push(format!("{}: {}", path.display(), e));
                continue;
            }
        };

        let unit = parse_unit_rs(&src);
        merge_cpp_lines(
            &mut spec,
            &unit,
            &mut cpp_line_seen,
            &mut template_body_seen,
        );
        merge_classes(&mut spec, &unit, &mut method_seen);
        merge_lib(&mut spec, &unit, &mut fn_attr_to_sig, &mut fwd_decl_seen);
        // 顺带收集降级签名，与文件读取合并为一次 I/O
        collect_degraded_sigs_from_str(&src, &mut spec.degraded_sigs);
    }

    (spec, read_errors)
}

/// 从单个文件内容字符串中提取含 `cpp2rust-todo` 标记的 C++ 签名，追加到 `degraded`。
fn collect_degraded_sigs_from_str(content: &str, degraded: &mut HashSet<String>) {
    let lines: Vec<&str> = content.lines().collect();
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        let maybe_sig = if trimmed.starts_with("#[cpp(func") {
            extract_attr_quoted_value(trimmed, "func = ")
        } else if trimmed.starts_with("#[cpp(method") {
            extract_attr_quoted_value(trimmed, "method = ")
        } else {
            None
        };
        if let Some(sig) = maybe_sig {
            // 向上最多扫描 2 行，看是否存在 cpp2rust-todo 注释
            let start = i.saturating_sub(2);
            if lines[start..i].iter().any(|l| l.contains("cpp2rust-todo")) {
                degraded.insert(sig);
            }
        }
    }
}

fn merge_cpp_lines(
    spec: &mut MergedSpec,
    unit: &ParsedUnit,
    seen: &mut HashSet<String>,
    template_body_seen: &mut HashSet<String>,
) {
    // 先计算本 unit cpp_lines 中所有模板体的行范围
    let template_ranges = block_parser::detect_template_body_ranges(&unit.cpp_lines);

    // template_line_idxs：所有模板体覆盖的行索引（含跳过的），用于区分"模板行"与"普通行"
    // kept_line_idxs：仅属于保留模板体的行索引，O(1) 替代原来的 ranges_to_keep 线性扫描
    let mut template_line_idxs: HashSet<usize> = HashSet::new();
    let mut kept_line_idxs: HashSet<usize> = HashSet::new();
    for (start, end, key) in template_ranges {
        for idx in start..=end {
            template_line_idxs.insert(idx);
        }
        if template_body_seen.contains(&key) {
            // 此模板体在之前的 TU 中已有相同规范化内容，跳过
        } else {
            template_body_seen.insert(key.clone());
            spec.template_bodies.push(key.clone());
            // 预先将此模板体的所有行索引加入 kept_line_idxs，查询时 O(1)
            for idx in start..=end {
                kept_line_idxs.insert(idx);
            }
        }
    }

    // 逐行处理：模板体范围内的行按块整体决定，其他行逐行去重
    for (idx, line) in unit.cpp_lines.iter().enumerate() {
        if template_line_idxs.contains(&idx) {
            if kept_line_idxs.contains(&idx) {
                // 模板体内的行直接追加（不做逐行 seen 检查，整体块已去重）
                spec.cpp_lines.push(line.clone());
            }
            // 否则整个模板块已被跳过
        } else {
            // 非模板行：逐行去重
            if !seen.contains(line) {
                seen.insert(line.clone());
                spec.cpp_lines.push(line.clone());
            }
        }
    }
}

fn merge_classes(
    spec: &mut MergedSpec,
    unit: &ParsedUnit,
    method_seen: &mut HashMap<(String, String), String>,
) {
    for cb in &unit.class_blocks {
        let methods = match spec.classes.entry(cb.class_name.clone()) {
            std::collections::hash_map::Entry::Vacant(e) => {
                spec.class_order.push(cb.class_name.clone());
                // 首次遇到时记录完整属性行
                if !cb.class_attr.is_empty() {
                    spec.class_attrs
                        .insert(cb.class_name.clone(), cb.class_attr.clone());
                }
                e.insert(Vec::new())
            }
            std::collections::hash_map::Entry::Occupied(e) => e.into_mut(),
        };
        // 模板特化分组
        if let Some(base) = &cb.template_base {
            spec.template_groups
                .entry(base.clone())
                .or_default()
                .push(cb.class_name.clone());
        }
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
        for (i, m) in methods.iter().enumerate() {
            out.push_str(&format!("        {}\n", m.attr));
            out.push_str(&format!("        {}\n", m.fn_sig));
            if i + 1 < methods.len() {
                out.push('\n');
            }
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
    let mut result: Vec<std::path::PathBuf> = WalkDir::new(src_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            let p = e.path();
            p.extension().and_then(|ext| ext.to_str()) == Some("rs")
                && !matches!(
                    p.file_name().and_then(|n| n.to_str()),
                    Some("lib.rs") | Some("mod.rs")
                )
        })
        .map(|e| e.path().to_path_buf())
        .collect();
    result.sort();
    result
}

/// 扫描单元 `.rs` 文件，返回紧跟在 `cpp2rust-todo` 注释之后的 C++ 签名集合。
///
/// 注：调用 `merge_units` 时已在读文件过程中顺带收集（`MergedSpec::degraded_sigs`），
/// 此函数供只有文件路径、无 `MergedSpec` 的独立场景使用。
pub fn extract_degraded_sigs(unit_files: &[std::path::PathBuf]) -> HashSet<String> {
    let mut degraded: HashSet<String> = HashSet::new();
    for path in unit_files {
        let Ok(content) = std::fs::read_to_string(path) else {
            continue;
        };
        collect_degraded_sigs_from_str(&content, &mut degraded);
    }
    degraded
}

/// 从形如 `#[cpp(func = "sig")]` 的属性行中提取引号内的值。
/// 也供 `main` 模块使用，以避免重复实现相同逻辑。
pub fn extract_attr_quoted_value(line: &str, key: &str) -> Option<String> {
    let pos = line.find(key)?;
    let rest = &line[pos + key.len()..];
    let start = rest.find('"')? + 1;
    let end = rest[start..].find('"')?;
    Some(rest[start..start + end].to_string())
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
/// - 首次运行：`src/` → rename → `src.1/`；复制 `src.1/` → `src.2/`（暂存）；rename `src.2` → `src`
/// - 重复运行：删除旧 `src/`；重新复制 `src.1/` → `src.2/`（暂存）；rename `src.2` → `src`
///
/// `src` 始终是真实目录（非 symlink），跨 Linux / macOS / Windows 三平台行为一致。
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
        // 历史遗留 symlink（从旧版本迁移）：删除
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

    // ── 原子性地将暂存目录 src.2 rename 为 src ──
    std::fs::rename(&src2, &src)
        .map_err(|e| anyhow!("rename {} → {}: {}", src2.display(), src.display(), e))?;

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

        let (spec, _) = merge_units(&[p1, p2]);
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

        let (spec, _) = merge_units(&[p1, p2]);
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

        let (spec, _) = merge_units(&[p1, p2]);
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
    fn merge_in_place_creates_backup_and_real_dir() {
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
        // src 是真实目录（非 symlink）
        assert!(
            rust_dir.join("src").is_dir(),
            "src should be a real directory"
        );
        assert!(
            !rust_dir.join("src").is_symlink(),
            "src should not be a symlink"
        );
        // src.2 已被 rename 为 src，不再存在
        assert!(
            !rust_dir.join("src.2").exists(),
            "src.2 should have been renamed to src"
        );
        // src 可正常访问
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

        // 子目录结构在 src 中保留（src.2 已被 rename 为 src）
        assert!(
            rust_dir.join("src/utils/foo.rs").exists(),
            "subdirectory structure preserved in src"
        );
        // src.1 中也保留
        assert!(rust_dir.join("src.1/utils/foo.rs").exists());
    }

    #[test]
    fn merge_in_place_rerun_keeps_src1() {
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
        // src 是真实目录，可正常访问
        assert!(rust_dir.join("src/lib.rs").exists());
        assert!(
            !rust_dir.join("src").is_symlink(),
            "src should not be a symlink"
        );
    }

    #[test]
    fn merge_in_place_errors_when_src_missing() {
        let tmp = tempfile::TempDir::new().unwrap();
        // rust_dir 下既没有 src 也没有 src.1
        let result = merge_in_place(tmp.path());
        assert!(result.is_err(), "should error when src does not exist");
    }

    // ── extract_degraded_sigs ──────────────────

    #[test]
    fn extract_degraded_sigs_detects_fn_todo() {
        let src = r#"hicc::import_lib! {
    #![link_name = "foo"]

    // cpp2rust-todo[FP]: 含函数指针参数
    #[cpp(func = "void cb(void (*fn)(int))")]
    fn cb(fn_: usize);

    #[cpp(func = "Foo* foo_new()")]
    fn foo_new() -> *mut Foo;
}
"#;
        let dir = tempfile::TempDir::new().unwrap();
        let p = dir.path().join("unit.rs");
        std::fs::write(&p, src).unwrap();

        let degraded = extract_degraded_sigs(&[p]);
        assert!(
            degraded.contains("void cb(void (*fn)(int))"),
            "fn with todo comment should be degraded"
        );
        assert!(
            !degraded.contains("Foo* foo_new()"),
            "fn without todo comment should not be degraded"
        );
    }

    #[test]
    fn extract_degraded_sigs_detects_method_todo() {
        let src = r#"hicc::import_class! {
    #[cpp(class = "Bar")]
    class Bar {
        // cpp2rust-todo[FP]: fn ptr method
        #[cpp(method = "void set_cb(void (*f)(int))")]
        fn set_cb(&mut self, f: usize);

        #[cpp(method = "int get() const")]
        fn get(&self) -> i32;
    }
}
"#;
        let dir = tempfile::TempDir::new().unwrap();
        let p = dir.path().join("unit.rs");
        std::fs::write(&p, src).unwrap();

        let degraded = extract_degraded_sigs(&[p]);
        assert!(
            degraded.contains("void set_cb(void (*f)(int))"),
            "method with todo should be degraded"
        );
        assert!(
            !degraded.contains("int get() const"),
            "method without todo should not be degraded"
        );
    }

    #[test]
    fn extract_degraded_sigs_empty_when_no_todos() {
        let src = r#"hicc::import_lib! {
    #![link_name = "foo"]

    #[cpp(func = "int add(int a, int b)")]
    fn add(a: i32, b: i32) -> i32;
}
"#;
        let dir = tempfile::TempDir::new().unwrap();
        let p = dir.path().join("unit.rs");
        std::fs::write(&p, src).unwrap();

        let degraded = extract_degraded_sigs(&[p]);
        assert!(degraded.is_empty(), "no todos means no degraded sigs");
    }

    // ── merge_units 边界用例 ───────────────────

    #[test]
    fn merge_units_empty_input_returns_default_spec() {
        let (spec, _) = merge_units(&[]);
        assert!(spec.cpp_lines.is_empty());
        assert!(spec.classes.is_empty());
        assert!(spec.fn_bindings.is_empty());
        assert!(spec.fwd_decls.is_empty());
        assert!(spec.conflicts.is_empty());
        assert!(spec.degraded_sigs.is_empty());
    }

    #[test]
    fn merge_units_single_file_preserves_content() {
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

hicc::import_lib! {
    #![link_name = "foo"]
    #[cpp(func = "Foo* foo_new()")]
    fn foo_new() -> *mut Foo;
}
"#;
        let dir = tempfile::TempDir::new().unwrap();
        let p = dir.path().join("unit.rs");
        std::fs::write(&p, src).unwrap();

        let (spec, _) = merge_units(&[p]);
        assert_eq!(spec.cpp_lines.len(), 1, "cpp_lines should contain 1 include");
        assert!(spec.cpp_lines[0].contains("foo.h"));
        assert!(spec.classes.contains_key("Foo"));
        assert_eq!(spec.classes["Foo"].len(), 1, "Foo should have 1 method");
        assert_eq!(spec.fn_bindings.len(), 1);
        assert!(spec.fn_bindings[0].attr.contains("foo_new"));
    }

    #[test]
    fn merge_units_conflict_warning_on_method_sig_mismatch() {
        // 两个文件中相同 class 的相同方法但返回类型不同 → 应生成冲突警告
        let src1 = r#"hicc::import_class! {
    #[cpp(class = "Bar")]
    class Bar {
        #[cpp(method = "int compute() const")]
        fn compute(&self) -> i32;
    }
}
"#;
        let src2 = r#"hicc::import_class! {
    #[cpp(class = "Bar")]
    class Bar {
        #[cpp(method = "int compute() const")]
        fn compute(&self) -> i64;
    }
}
"#;
        let dir = tempfile::TempDir::new().unwrap();
        let p1 = dir.path().join("unit1.rs");
        let p2 = dir.path().join("unit2.rs");
        std::fs::write(&p1, src1).unwrap();
        std::fs::write(&p2, src2).unwrap();

        let (spec, _) = merge_units(&[p1, p2]);
        // 冲突：相同 method attr 但 fn_sig 不一致
        assert!(
            !spec.conflicts.is_empty(),
            "应检测到方法签名冲突：{:?}",
            spec.conflicts
        );
    }

    #[test]
    fn merge_units_fn_binding_conflict_detected() {
        // 两个文件中相同函数 attr 但 fn_sig 不同 → 应生成函数绑定冲突
        let src1 = r#"hicc::import_lib! {
    #![link_name = "mylib"]
    #[cpp(func = "int add(int a, int b)")]
    fn add(a: i32, b: i32) -> i32;
}
"#;
        let src2 = r#"hicc::import_lib! {
    #![link_name = "mylib"]
    #[cpp(func = "int add(int a, int b)")]
    fn add(a: i64, b: i64) -> i64;
}
"#;
        let dir = tempfile::TempDir::new().unwrap();
        let p1 = dir.path().join("unit1.rs");
        let p2 = dir.path().join("unit2.rs");
        std::fs::write(&p1, src1).unwrap();
        std::fs::write(&p2, src2).unwrap();

        let (spec, _) = merge_units(&[p1, p2]);
        assert!(
            !spec.conflicts.is_empty(),
            "应检测到函数绑定冲突：{:?}",
            spec.conflicts
        );
        assert!(
            spec.conflicts[0].contains("Function binding conflict"),
            "冲突消息应包含 'Function binding conflict'，实际：{}",
            spec.conflicts[0]
        );
    }

    #[test]
    fn merge_units_cpp_lines_dedup_across_files() {
        // 三个文件中 cpp 内容有重叠，合并后应去重
        let make_src = |extra: &str| {
            format!(
                r#"hicc::cpp! {{
    #include "common.h"
    {}
}}
"#,
                extra
            )
        };
        let src1 = make_src("#include \"a.h\"");
        let src2 = make_src("#include \"common.h\""); // 重复
        let src3 = make_src("#include \"b.h\"");

        let dir = tempfile::TempDir::new().unwrap();
        let p1 = dir.path().join("u1.rs");
        let p2 = dir.path().join("u2.rs");
        let p3 = dir.path().join("u3.rs");
        std::fs::write(&p1, src1).unwrap();
        std::fs::write(&p2, src2).unwrap();
        std::fs::write(&p3, src3).unwrap();

        let (spec, _) = merge_units(&[p1, p2, p3]);
        let common_count = spec
            .cpp_lines
            .iter()
            .filter(|l| l.contains("common.h"))
            .count();
        assert_eq!(common_count, 1, "common.h 应只出现一次，实际 cpp_lines: {:?}", spec.cpp_lines);
    }

    // ── collect_degraded_sigs_from_str 边界测试 ──────────────────────────────

    #[test]
    fn collect_degraded_sigs_normal_case() {
        // 签名前 2 行有 cpp2rust-todo 注释时应被收集
        let content = "\
// cpp2rust-todo[FP] 含函数指针\n\
#[cpp(func = \"void foo()\")]\n\
fn foo();\n";
        let mut degraded = HashSet::new();
        collect_degraded_sigs_from_str(content, &mut degraded);
        assert!(degraded.contains("void foo()"), "应收集到降级签名，实际: {:?}", degraded);
    }

    #[test]
    fn collect_degraded_sigs_no_todo_no_collect() {
        // 没有 cpp2rust-todo 注释时不收集
        let content = "\
#[cpp(func = \"void bar()\")]\n\
fn bar();\n";
        let mut degraded = HashSet::new();
        collect_degraded_sigs_from_str(content, &mut degraded);
        assert!(degraded.is_empty(), "无降级注释时不应收集，实际: {:?}", degraded);
    }

    #[test]
    fn collect_degraded_sigs_sig_at_line_0() {
        // 签名恰好在文件第 0 行（最顶部），saturating_sub(2) = 0，向上扫描范围 [0..0] 为空
        // 前面没有可扫描的行 → 不应收集（也不应 panic）
        let content = "#[cpp(func = \"void first()\")]\nfn first();\n";
        let mut degraded = HashSet::new();
        collect_degraded_sigs_from_str(content, &mut degraded);
        assert!(degraded.is_empty(), "第 0 行的签名前无 todo 注释，不应收集，实际: {:?}", degraded);
    }

    #[test]
    fn collect_degraded_sigs_sig_at_line_1() {
        // 签名在第 1 行（第二行），saturating_sub(2) = 0，向上扫描范围 [0..1] 只有一行
        // 第 0 行有 todo 注释 → 应收集
        let content = "// cpp2rust-todo[OP]\n#[cpp(func = \"int op()\")]\nfn op() -> i32;\n";
        let mut degraded = HashSet::new();
        collect_degraded_sigs_from_str(content, &mut degraded);
        assert!(degraded.contains("int op()"), "第 1 行签名前有 todo 注释，应收集，实际: {:?}", degraded);
    }

    #[test]
    fn collect_degraded_sigs_method_attr() {
        // method 属性同样应被识别
        let content = "\
// cpp2rust-todo[VM]\n\
#[cpp(method = \"void set(int) volatile\")]\n\
fn set(&mut self, v: i32);\n";
        let mut degraded = HashSet::new();
        collect_degraded_sigs_from_str(content, &mut degraded);
        assert!(
            degraded.contains("void set(int) volatile"),
            "method 属性应被收集，实际: {:?}",
            degraded
        );
    }

    #[test]
    fn collect_degraded_sigs_todo_3_lines_above_not_collected() {
        // todo 注释距签名超过 2 行，不应收集
        let content = "\
// cpp2rust-todo[FP]\n\
// comment 1\n\
// comment 2\n\
#[cpp(func = \"void far()\")]\n\
fn far();\n";
        let mut degraded = HashSet::new();
        collect_degraded_sigs_from_str(content, &mut degraded);
        assert!(degraded.is_empty(), "距离超 2 行的 todo 不应触发收集，实际: {:?}", degraded);
    }
}
