//! post-merge Rust 源码统计模块
//!
//! 扫描生成的 `.rs` 文件，统计 FFI 绑定指标（import_lib! 数、绑定函数数、降级标记等）。

use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

// ─── 字符串常量 ────────────────────────────────────────────────────────────────

pub const IMPORT_LIB_MARKER: &str = "hicc::import_lib!";
pub const IMPORT_CLASS_MARKER: &str = "hicc::import_class!";
pub const FN_BINDING_MARKER: &str = "#[cpp(func =";
pub const LINK_NAME_PREFIX: &str = "link_name = \"";
pub const TODO_MARKER_PREFIX: &str = "cpp2rust-todo[";

/// 统计扫描结果
pub struct RustSrcMetrics {
    pub rs_files: Vec<PathBuf>,
    pub import_lib_files: usize,
    pub import_class_files: usize,
    pub fn_binding_count: usize,
    /// link_name 值中含路径分隔符 '/' 的列表
    pub bad_link_names: Vec<String>,
    /// `#include` 指令总数
    pub include_count: usize,
    /// cpp2rust-todo 降级标记总数
    pub todo_count: usize,
    /// (tag, total_count) 降级标记按 tag 汇总
    pub degraded_tags: Vec<(String, usize)>,
}

/// 从单行代码中提取 `cpp2rust-todo[TAG]` 标签字符串（若存在）。
///
/// 返回的切片借用自入参 `line`，调用方可选择 `.to_string()` 持有或直接比较。
pub fn parse_todo_tag_from_line(line: &str) -> Option<&str> {
    let start = line.find(TODO_MARKER_PREFIX)?;
    let rest = &line[start + TODO_MARKER_PREFIX.len()..];
    let end = rest.find(']')?;
    Some(&rest[..end])
}

/// 统计文件行数。
///
/// 通过统计字节流中 `\n` 数量实现，避免为大文件（如 SQLite 展开后的数万行）
/// 分配 UTF-8 字符串切片，比 `BufReader::lines().count()` 开销更低。
pub fn count_file_lines(path: &Path) -> usize {
    use std::io::Read;
    std::fs::File::open(path)
        .map(|f| {
            BufReader::new(f)
                .bytes()
                .filter(|b| matches!(b, Ok(b'\n')))
                .count()
        })
        .unwrap_or(0)
}

/// 扫描 `rust_src` 目录下所有 `.rs` 文件，统计 FFI 绑定指标。
pub fn collect_rust_src_metrics(rust_src: &Path) -> RustSrcMetrics {
    let mut rs_files: Vec<PathBuf> = WalkDir::new(rust_src)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
        .map(|e| e.into_path())
        .collect();
    rs_files.sort();

    let mut import_lib_files = 0usize;
    let mut import_class_files = 0usize;
    let mut fn_binding_count = 0usize;
    let mut bad_link_names: Vec<String> = Vec::new();
    let mut include_count = 0usize;
    let mut todo_tags: std::collections::HashMap<String, usize> = std::collections::HashMap::new();

    for path in &rs_files {
        let Ok(content) = std::fs::read_to_string(path) else {
            continue;
        };
        // 对每个文件做单趟 line 循环，合并原来的 contains/matches 预扫描与 lines() 遍历
        let mut found_import_lib = false;
        let mut found_import_class = false;
        for line in content.lines() {
            let trimmed = line.trim();
            if !found_import_lib && trimmed.contains(IMPORT_LIB_MARKER) {
                found_import_lib = true;
            }
            if !found_import_class && trimmed.contains(IMPORT_CLASS_MARKER) {
                found_import_class = true;
            }
            fn_binding_count += trimmed.matches(FN_BINDING_MARKER).count();
            // 仅统计行首 #include（trimmed 以 "#include" 开头即不是注释行）
            if trimmed.starts_with("#include") {
                include_count += 1;
            }
            // link_name = "..." 提取
            if let Some(pos) = trimmed.find(LINK_NAME_PREFIX) {
                let rest = &trimmed[pos + LINK_NAME_PREFIX.len()..];
                if let Some(end) = rest.find('"') {
                    let name = &rest[..end];
                    if name.contains('/') {
                        bad_link_names.push(name.to_string());
                    }
                }
            }
            // cpp2rust-todo[TAG] 统计（复用公共解析函数）
            if let Some(tag) = parse_todo_tag_from_line(line) {
                *todo_tags.entry(tag.to_string()).or_insert(0) += 1;
            }
        }
        if found_import_lib {
            import_lib_files += 1;
        }
        if found_import_class {
            import_class_files += 1;
        }
    }

    let mut degraded_tags: Vec<(String, usize)> = todo_tags.into_iter().collect();
    degraded_tags.sort_by(|a, b| a.0.cmp(&b.0));
    let todo_count: usize = degraded_tags.iter().map(|(_, c)| c).sum();

    RustSrcMetrics {
        rs_files,
        import_lib_files,
        import_class_files,
        fn_binding_count,
        bad_link_names,
        include_count,
        todo_count,
        degraded_tags,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_file(dir: &std::path::Path, name: &str, content: &str) {
        std::fs::write(dir.join(name), content).unwrap();
    }

    #[test]
    fn metrics_basic_counts() {
        let dir = tempfile::TempDir::new().unwrap();
        write_file(
            dir.path(),
            "unit1.rs",
            r#"hicc::import_lib! {
    #![link_name = "foo"]
    #[cpp(func = "void bar()")]
    fn bar();
    #[cpp(func = "void baz()")]
    fn baz();
}
"#,
        );
        write_file(
            dir.path(),
            "unit2.rs",
            r#"hicc::import_class! {
    #[cpp(class = "Foo")]
    class Foo {}
}
// cpp2rust-todo[FP] skipped
// cpp2rust-todo[OP] another
"#,
        );

        let m = collect_rust_src_metrics(dir.path());
        assert_eq!(m.import_lib_files, 1);
        assert_eq!(m.import_class_files, 1);
        assert_eq!(m.fn_binding_count, 2);
        assert_eq!(m.todo_count, 2);
        let tags: std::collections::HashMap<_, _> = m.degraded_tags.iter().cloned().collect();
        assert_eq!(tags["FP"], 1);
        assert_eq!(tags["OP"], 1);
    }

    #[test]
    fn metrics_empty_directory() {
        let dir = tempfile::TempDir::new().unwrap();
        let m = collect_rust_src_metrics(dir.path());
        assert_eq!(m.rs_files.len(), 0);
        assert_eq!(m.import_lib_files, 0);
        assert_eq!(m.fn_binding_count, 0);
        assert_eq!(m.todo_count, 0);
    }

    #[test]
    fn metrics_bad_link_name_detected() {
        let dir = tempfile::TempDir::new().unwrap();
        write_file(
            dir.path(),
            "unit.rs",
            r#"hicc::import_lib! {
    #![link_name = "sub/foo"]
    #[cpp(func = "void go()")]
    fn go();
}
"#,
        );
        let m = collect_rust_src_metrics(dir.path());
        assert_eq!(m.bad_link_names, vec!["sub/foo"]);
    }

    #[test]
    fn count_file_lines_correct() {
        let dir = tempfile::TempDir::new().unwrap();
        let p = dir.path().join("test.rs");
        std::fs::write(&p, "line1\nline2\nline3\n").unwrap();
        assert_eq!(count_file_lines(&p), 3);
    }

    // ── parse_todo_tag_from_line ──────────────────────────────────────────────

    #[test]
    fn parse_todo_tag_from_line_detects_tag() {
        assert_eq!(
            parse_todo_tag_from_line("// cpp2rust-todo[FP]: 需手动处理"),
            Some("FP")
        );
        assert_eq!(
            parse_todo_tag_from_line("    // cpp2rust-todo[LONG_DOUBLE] comment"),
            Some("LONG_DOUBLE")
        );
    }

    #[test]
    fn parse_todo_tag_from_line_returns_none_when_absent() {
        assert_eq!(parse_todo_tag_from_line("// 普通注释"), None);
        assert_eq!(parse_todo_tag_from_line("let x = 1;"), None);
        assert_eq!(parse_todo_tag_from_line(""), None);
    }

    #[test]
    fn parse_todo_tag_from_line_unclosed_bracket_returns_none() {
        // 未闭合的 `[` 不应匹配
        assert_eq!(parse_todo_tag_from_line("// cpp2rust-todo[OPEN"), None);
    }
}
