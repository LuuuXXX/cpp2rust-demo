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

/// 统计文件行数（逐行读取，内存高效）。
pub fn count_file_lines(path: &Path) -> usize {
    std::fs::File::open(path)
        .map(|f| BufReader::new(f).lines().count())
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
            // cpp2rust-todo[TAG] 统计
            if let Some(start) = line.find(TODO_MARKER_PREFIX) {
                let rest = &line[start + TODO_MARKER_PREFIX.len()..];
                if let Some(end) = rest.find(']') {
                    let tag = rest[..end].to_string();
                    *todo_tags.entry(tag).or_insert(0) += 1;
                }
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
