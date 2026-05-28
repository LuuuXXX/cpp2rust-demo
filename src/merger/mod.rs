//! Merge 命令核心逻辑（Phase 6）
//!
//! 将一个或多个 feature 下按编译单元生成的 `.rs` 文件合并为去重后的单文件输出：
//! 单个 `hicc::cpp!` + 每类一个 `hicc::import_class!` + 单个 `hicc::import_lib!`

pub mod block_parser;

use block_parser::{parse_unit_rs, ParsedFnBinding, ParsedUnit};
use std::collections::{HashMap, HashSet};
use std::path::Path;

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

/// 合并多个 unit `.rs` 文件到一个 `MergedSpec`。
/// `merge_link_name`：合并后 `import_lib!` 中的 `#![link_name = "..."]` 值。
pub fn merge_units(
    unit_rs_paths: &[std::path::PathBuf],
    merge_link_name: &str,
) -> MergedSpec {
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
                eprintln!(
                    "  Warning: cannot read {}: {}",
                    path.display(), e
                );
                continue;
            }
        };

        let unit = parse_unit_rs(&src);
        merge_cpp_lines(&mut spec, &unit, &mut cpp_line_seen);
        merge_classes(&mut spec, &unit, &mut method_seen);
        merge_lib(&mut spec, &unit, &mut fn_attr_to_sig, &mut fwd_decl_seen);
    }

    // merge_link_name 通过 emit_merged_rs 的参数传递，无需存储在 MergedSpec 中
    drop(merge_link_name);

    spec
}

fn merge_cpp_lines(
    spec: &mut MergedSpec,
    unit: &ParsedUnit,
    seen: &mut HashSet<String>,
) {
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
    out.push_str("hicc::cpp! {\n");
    for line in &spec.cpp_lines {
        if line.is_empty() {
            out.push('\n');
        } else {
            out.push_str("    ");
            out.push_str(line);
            out.push('\n');
        }
    }
    out.push_str("}\n");

    // ── hicc::import_class! (每类一个块) ────────
    for class_name in &spec.class_order {
        let methods = match spec.classes.get(class_name) {
            Some(m) => m,
            None => continue,
        };
        if methods.is_empty() {
            continue;
        }
        out.push('\n');
        out.push_str("hicc::import_class! {\n");
        out.push_str(&format!("    #[cpp(class = \"{}\")]\n", class_name));
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

/// 扫描 `src_dir` 下所有 `*.rs` 文件，返回路径列表（排序）。
/// 排除 `lib.rs`（汇总模块，不含 FFI 块）。
pub fn collect_unit_rs_files(src_dir: &Path) -> Vec<std::path::PathBuf> {
    let mut result = Vec::new();
    let rd = match std::fs::read_dir(src_dir) {
        Ok(r) => r,
        Err(_) => return result,
    };
    for entry in rd.flatten() {
        let p = entry.path();
        if p.extension().and_then(|e| e.to_str()) == Some("rs") {
            let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if name != "lib.rs" {
                result.push(p);
            }
        }
    }
    result.sort();
    result
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

        let spec = merge_units(&[p1, p2], "merged");
        // foo.h 应只出现一次
        let foo_count = spec.cpp_lines.iter().filter(|l| l.contains("foo.h")).count();
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

        let spec = merge_units(&[p1, p2], "merged");
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

        let spec = merge_units(&[p1, p2], "merged");
        assert_eq!(spec.fn_bindings.len(), 1, "duplicate fn binding should be deduped");
        assert_eq!(spec.fwd_decls.len(), 1, "duplicate fwd_decl should be deduped");
    }
}
