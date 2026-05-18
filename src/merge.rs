//! `merge` command implementation.
//!
//! The `merge` command reads flat per-stem `.rs` files produced by `init`
//! (`<stem>.rs`) and emits:
//!
//! 1. `rust/src.2/<stem>.rs` (per-stem stripped view, for reference)
//! 2. `rust/src.2/lib.rs` (consolidated FFI entry point – replaces the old
//!    `merged_ffi.rs`; contains all `hicc::cpp!` / `import_class!` /
//!    `import_lib!` content aggregated from every translation unit)
//!
//! Design rationale:
//! Since compilation units are already flattened 1 : 1 (one `.rs` per `.cpp`),
//! a separate `merged_ffi.rs` is redundant.  All hicc-essential content
//! (includes, enum defs, type aliases, class bindings, free functions) is
//! written directly into `lib.rs`.  Non-business metadata constants such as
//! `CPP_TYPES`, `CPP_RUST_TYPE_MAPPINGS`, `CLASS_NAMES`, etc. are intentionally
//! excluded from the merged output to keep the generated project lean.
//!
//! `build.rs` references `src/lib.rs` (which after `link_src_to_src2` points to
//! `src.2/lib.rs`) so hicc-build processes the consolidated entry point.

use crate::error::Result;
use anyhow::anyhow;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct MergeOutput {
    /// Path to the consolidated `lib.rs` produced by merge.
    pub lib_path: PathBuf,
    pub group_modules: Vec<String>,
}

#[derive(Default)]
struct ModuleFragments {
    includes: indexmap::IndexSet<String>,
    /// `import_class!` blocks, deduplicated by Rust struct name (first-wins).
    /// Key = Rust struct name; value = full block text.
    import_class_blocks: indexmap::IndexMap<String, String>,
    forward_decls: indexmap::IndexSet<String>,
    fn_items: Vec<String>,
    type_blocks: indexmap::IndexSet<String>,
}

pub fn merge_grouped_modules(
    init_src_dir: &Path,
    out_src2_dir: &Path,
    link_name: &str,
) -> Result<MergeOutput> {
    // Scan for flat <stem>.rs files, excluding lib.rs and any files inside
    // the common/ subdirectory or other subdirectories.
    let mut flat_files: Vec<PathBuf> = fs::read_dir(init_src_dir)
        .map_err(|e| anyhow!("read dir {}: {}", init_src_dir.display(), e))?
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|p| {
            p.is_file()
                && p.extension().and_then(|e| e.to_str()) == Some("rs")
                && p.file_name()
                    .and_then(|n| n.to_str())
                    .map(|n| n != "lib.rs")
                    .unwrap_or(false)
        })
        .collect();
    flat_files.sort();

    if flat_files.is_empty() {
        return Err(anyhow!(
            "no flat <stem>.rs files found in {}; run 'init' first",
            init_src_dir.display()
        ));
    }

    if out_src2_dir.exists() {
        fs::remove_dir_all(out_src2_dir)
            .map_err(|e| anyhow!("remove {}: {}", out_src2_dir.display(), e))?;
    }
    fs::create_dir_all(out_src2_dir)
        .map_err(|e| anyhow!("create {}: {}", out_src2_dir.display(), e))?;

    let mut merged_all = ModuleFragments::default();
    let mut group_modules = Vec::new();

    // Load the aggregate type block from common/types.rs so that enum defs
    // and type aliases (business code) can be included in lib.rs.  Only the
    // business-relevant parts are extracted; CPP_TYPES / CPP_RUST_TYPE_MAPPINGS
    // and similar non-business metadata are intentionally stripped.  The
    // includes_block (common/includes.rs) is also omitted because it contains
    // only internal path metadata (MIDDLEWARE_FILES, …) that has no role in
    // the FFI bindings.
    let common_types_block = {
        let common_types_path = init_src_dir.join("common").join("types.rs");
        if common_types_path.exists() {
            Some(
                fs::read_to_string(&common_types_path)
                    .map_err(|e| anyhow!("read {}: {}", common_types_path.display(), e))?,
            )
        } else {
            None
        }
    };

    for flat_file in &flat_files {
        let stem = flat_file
            .file_stem()
            .and_then(|n| n.to_str())
            .ok_or_else(|| anyhow!("invalid flat file name: {}", flat_file.display()))?
            .to_string();
        let out_file = out_src2_dir.join(format!("{stem}.rs"));
        let fragments = merge_flat_file(flat_file, &out_file, link_name)?;

        for inc in fragments.includes.iter() {
            merged_all.includes.insert(inc.clone());
        }
        for (name, block) in fragments.import_class_blocks.iter() {
            merged_all
                .import_class_blocks
                .insert(name.clone(), block.clone());
        }
        for decl in fragments.forward_decls.iter() {
            merged_all.forward_decls.insert(decl.clone());
        }
        merged_all
            .fn_items
            .extend(fragments.fn_items.iter().cloned());

        group_modules.push(stem);
    }

    let mod_refs: Vec<&str> = group_modules.iter().map(|s| s.as_str()).collect();

    // Write per-stem files for reference (they are NOT re-exported from lib.rs
    // to avoid duplicate symbol definitions across translation units).
    // Users who need per-stem access can read src.2/<stem>.rs directly.
    let _ = mod_refs; // consumed above for the reference note

    // Build the consolidated lib.rs that contains ALL hicc-essential content:
    //   hicc::cpp! include block, enum defs, type aliases, import_class! blocks,
    //   import_lib! block.  Non-business metadata constants (CPP_TYPES,
    //   CPP_RUST_TYPE_MAPPINGS, CLASS_NAMES, …) are intentionally excluded.
    let business_types_block = common_types_block
        .as_deref()
        .map(extract_business_types_block)
        .filter(|s| !s.is_empty());

    let lib_src = render_merged_module(
        &merged_all,
        business_types_block.as_deref(),
        link_name,
        true,
    );
    let lib_path = out_src2_dir.join("lib.rs");
    fs::write(&lib_path, lib_src).map_err(|e| anyhow!("write {}: {}", lib_path.display(), e))?;

    Ok(MergeOutput {
        lib_path,
        group_modules,
    })
}

fn merge_flat_file(
    flat_file: &Path,
    output_file: &Path,
    link_name: &str,
) -> Result<ModuleFragments> {
    let mut fragments = ModuleFragments::default();
    let src = fs::read_to_string(flat_file)
        .map_err(|e| anyhow!("read {}: {}", flat_file.display(), e))?;

    for include in extract_cpp_includes(&src) {
        fragments.includes.insert(include);
    }
    for block in extract_import_class_blocks(&src) {
        let key = class_name_from_block(&block).unwrap_or_else(|| block.clone());
        fragments.import_class_blocks.insert(key, block);
    }
    for block in extract_import_lib_blocks(&src) {
        let (fwd, fns) = parse_lib_block_contents(&block);
        for f in fwd {
            fragments.forward_decls.insert(f);
        }
        fragments.fn_items.extend(fns);
    }

    // Extract enum definitions and emit them before import_class! blocks so
    // that types like ParseErrorCode / SchemaDraft / OpenApiVersion are in
    // scope when Rust compiles the import_class! invocations.
    let enum_block = extract_enum_defs_block(&src);
    if !enum_block.is_empty() {
        fragments.type_blocks.insert(enum_block);
    }

    // Extract C++ typedef / using aliases (business code needed for FFI).
    let alias_block = extract_type_aliases_block(&src);
    if !alias_block.is_empty() {
        fragments.type_blocks.insert(alias_block);
    }

    let rendered = render_merged_module(&fragments, None, link_name, false);
    fs::write(output_file, rendered)
        .map_err(|e| anyhow!("write {}: {}", output_file.display(), e))?;

    Ok(fragments)
}

/// Render the merged module content.
///
/// `business_types_block` — when `Some`, contains only the business-relevant
/// types (enum defs and typedef/using aliases) extracted from `common/types.rs`.
/// Non-business metadata constants (CPP_TYPES, CPP_RUST_TYPE_MAPPINGS, …) are
/// intentionally excluded.  Pass `None` for per-stem reference files.
///
/// `is_lib` — when `true`, the output is destined for `lib.rs` (the
/// consolidated crate entry point); when `false`, for a per-stem reference file
/// in `src.2/<stem>.rs` (not compiled directly by the Rust toolchain).
///
/// The `common/includes.rs` block (MIDDLEWARE_FILES, MIDDLEWARE_BASENAMES, …)
/// is intentionally excluded from all merged outputs because those constants are
/// internal path bookkeeping that has no role in the actual FFI bindings.
/// Likewise, `class/` metadata constants (CLASS_NAMES, CLASS_METHOD_COUNTS, …)
/// are excluded.  All of that information remains available in the non-merged
/// per-stem sources under `rust/src.1/` for inspection purposes.
fn render_merged_module(
    fragments: &ModuleFragments,
    business_types_block: Option<&str>,
    link_name: &str,
    is_lib: bool,
) -> String {
    let mut out = String::new();
    if is_lib {
        out.push_str(
            "// Consolidated FFI entry point – auto-generated by `cpp2rust-demo merge`.\n",
        );
        out.push_str("// Edit build.rs to adjust compiler settings; re-run merge to regenerate.\n");
    } else {
        out.push_str("// Per-stem reference – auto-generated by `cpp2rust-demo merge`.\n");
        out.push_str(
            "// This file is kept for inspection; lib.rs is the active FFI entry point.\n",
        );
    }
    out.push_str("#![allow(non_snake_case, dead_code)]\n\n");

    if !fragments.includes.is_empty() {
        out.push_str("hicc::cpp! {\n");
        for include in fragments.includes.iter() {
            out.push_str(&format!("    {}\n", include));
        }
        out.push_str("}\n\n");
    }

    // Emit business-only type content (enum defs + type aliases) from
    // common/types.rs when building lib.rs (consolidated output).
    if let Some(block) = business_types_block {
        let trimmed = block.trim();
        if !trimmed.is_empty() {
            out.push_str(trimmed);
            out.push_str("\n\n");
        }
    }

    for block in fragments.type_blocks.iter() {
        out.push_str(block.trim());
        out.push_str("\n\n");
    }

    for block in fragments.import_class_blocks.values() {
        out.push_str(block.trim());
        out.push_str("\n\n");
    }

    out.push_str("hicc::import_lib! {\n");
    out.push_str(&format!("    #![link_name = \"{}\"]\n\n", link_name));
    for fwd in fragments.forward_decls.iter() {
        out.push_str(&format!("    {}\n", fwd));
    }
    if !fragments.forward_decls.is_empty() && !fragments.fn_items.is_empty() {
        out.push('\n');
    }
    for item in &fragments.fn_items {
        for line in item.lines() {
            out.push_str("    ");
            out.push_str(line);
            out.push('\n');
        }
        out.push('\n');
    }
    out.push_str("}\n");
    out
}

// ---------------------------------------------------------------------------
// Simple text extraction helpers
// ---------------------------------------------------------------------------

/// Extract `hicc::import_class! { ... }` blocks from `src`.
fn extract_import_class_blocks(src: &str) -> Vec<String> {
    extract_macro_blocks(src, "hicc::import_class!")
}

/// Extract the Rust struct name declared inside an `import_class!` block.
///
/// Looks for the first `class <Name>` line (possibly followed by `:` for
/// inheritance or `{` for the opening brace).  Returns `None` when the block
/// does not contain such a line.
///
/// Used as the deduplication key when merging blocks across multiple translation
/// units so that the same class emitted from two different AST nodes (e.g. once
/// as a child of `ClassTemplateDecl` and once as a standalone specialisation)
/// only appears once in `lib.rs`.
fn class_name_from_block(block: &str) -> Option<String> {
    for line in block.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("class ") {
            // The name ends at the first whitespace, ':', or '{'.
            let end = rest
                .find(|c: char| c == '{' || c == ':' || c.is_ascii_whitespace())
                .unwrap_or(rest.len());
            let name = rest[..end].trim();
            if !name.is_empty() {
                return Some(name.to_string());
            }
        }
    }
    None
}

/// Extract `hicc::import_lib! { ... }` blocks from `src`.
fn extract_import_lib_blocks(src: &str) -> Vec<String> {
    extract_macro_blocks(src, "hicc::import_lib!")
}

/// Extract individual `#include "..."` lines from all `hicc::cpp! { ... }` blocks.
fn extract_cpp_includes(src: &str) -> Vec<String> {
    let mut includes = Vec::new();
    for block in extract_macro_blocks(src, "hicc::cpp!") {
        let inner = strip_block_wrapper(&block);
        for line in inner.lines() {
            let trimmed = line.trim();
            if is_valid_include(trimmed) {
                includes.push(trimmed.to_string());
            }
        }
    }
    includes
}

/// Extract the `#[repr(C)] pub enum ...` section from a flat module source.
///
/// The flat source emitted by `render_flat_module` contains a
/// `// C++ enum / enum class definitions.` section that lists all extracted
/// C++ enums as `#[repr(C)] pub enum` items.  We re-emit that section verbatim
/// in the per-stem merged output so that types such as `ParseErrorCode`,
/// `SchemaDraft`, and `OpenApiVersion` are defined *before* the
/// `hicc::import_class!` invocations that reference them in method signatures.
fn extract_enum_defs_block(src: &str) -> String {
    const ENUM_MARKER: &str = "// C++ enum / enum class definitions.";
    // Stop markers that signal the end of the enum-defs section when seen at
    // the *top level* (brace depth 0).
    const STOP_MARKERS: &[&str] = &[
        "// C++ typedef",
        "pub const ",
        "pub fn ",
        "hicc::",
        "pub mod ",
        "pub use ",
        "pub struct ",
        "pub trait ",
        "pub type ",
    ];

    let start = match src.find(ENUM_MARKER) {
        Some(pos) => pos,
        None => return String::new(),
    };

    let after = &src[start + ENUM_MARKER.len()..];
    let mut end_offset = after.len(); // default: take everything after marker
    let mut brace_depth: i32 = 0;
    let mut consumed = 0usize;

    for line in after.lines() {
        let trimmed = line.trim();
        // Track brace depth so we don't stop inside an `impl` block.
        brace_depth += line.chars().filter(|&c| c == '{').count() as i32;
        brace_depth -= line.chars().filter(|&c| c == '}').count() as i32;

        let line_end = consumed + line.len() + 1; // +1 for '\n'

        // Only check stop markers at the top level (brace depth 0 after
        // processing the line's braces).
        if brace_depth == 0
            && !trimmed.is_empty()
            && STOP_MARKERS.iter().any(|m| trimmed.starts_with(m))
        {
            // Compute offset of the start of this line within `after`.
            end_offset = consumed;
            break;
        }

        consumed = line_end.min(after.len());
    }

    let enum_body = after[..end_offset].trim_end();
    if enum_body.is_empty() {
        return String::new();
    }

    format!("{}\n{}", ENUM_MARKER, enum_body)
}

/// Extract the `// C++ typedef / using aliases.` section from a flat init module.
///
/// Only `pub type` alias declarations are extracted (business code needed for
/// the FFI type hierarchy).  Non-business constants and functions that precede
/// the typedef section in the source are skipped.
fn extract_type_aliases_block(src: &str) -> String {
    const ALIAS_MARKER: &str = "// C++ typedef / using aliases.";
    let start = match src.find(ALIAS_MARKER) {
        Some(pos) => pos,
        None => return String::new(),
    };

    let after = &src[start + ALIAS_MARKER.len()..];
    let mut lines_out = Vec::new();

    for line in after.lines() {
        let trimmed = line.trim();
        // Stop when we hit a hicc macro, module declaration, or another section.
        if trimmed.starts_with("hicc::")
            || trimmed.starts_with("pub mod ")
            || trimmed.starts_with("pub use ")
            || trimmed.starts_with("pub fn ")
            || trimmed.starts_with("pub const ")
            || trimmed.starts_with("pub struct ")
            || trimmed.starts_with("pub trait ")
            || trimmed.starts_with("// Per-group")
            || trimmed.starts_with("// C++ enum")
        {
            break;
        }
        lines_out.push(line);
    }

    // Drop trailing empty lines.
    while lines_out
        .last()
        .map(|l: &&str| l.trim().is_empty())
        .unwrap_or(false)
    {
        lines_out.pop();
    }

    let alias_body = lines_out.join("\n");
    if alias_body.trim().is_empty() {
        return String::new();
    }

    format!("{}\n{}", ALIAS_MARKER, alias_body)
}

/// Extract only the business-relevant content from `common/types.rs`:
/// enum definitions and typedef/using alias declarations.
///
/// Strips non-business metadata constants (`CPP_TYPE_COUNT`, `CPP_TYPES`,
/// `CPP_RUST_TYPE_MAPPINGS`, `rust_type_for`, `has_cpp_type`) so that the
/// generated project remains lean.
fn extract_business_types_block(common_types_src: &str) -> String {
    let enum_block = extract_enum_defs_block(common_types_src);
    let alias_block = extract_type_aliases_block(common_types_src);

    let mut out = String::new();
    if !enum_block.is_empty() {
        out.push_str(enum_block.trim());
        out.push('\n');
    }
    if !alias_block.is_empty() {
        if !out.is_empty() {
            out.push('\n');
        }
        out.push_str(alias_block.trim());
        out.push('\n');
    }
    out
}

fn is_valid_include(line: &str) -> bool {
    if !line.starts_with("#include") {
        return false;
    }
    let rest = line["#include".len()..].trim();
    (rest.starts_with('"') && rest.ends_with('"') && rest.len() >= 2)
        || (rest.starts_with('<') && rest.ends_with('>') && rest.len() >= 2)
}

fn extract_macro_blocks(src: &str, macro_prefix: &str) -> Vec<String> {
    let mut blocks = Vec::new();
    let mut search_from = 0;

    while let Some(start) = src[search_from..].find(macro_prefix) {
        let abs_start = search_from + start;
        let brace_start = match src[abs_start..].find('{') {
            Some(b) => abs_start + b,
            None => break,
        };

        let mut depth = 0usize;
        let mut end = brace_start;
        for (i, ch) in src[brace_start..].char_indices() {
            match ch {
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        end = brace_start + i + 1;
                        break;
                    }
                }
                _ => {}
            }
        }

        blocks.push(src[abs_start..end].to_string());
        search_from = end;
    }

    blocks
}

/// Parse the inner contents of an `import_lib! { ... }` block into
/// (forward_declarations, fn_items).
fn parse_lib_block_contents(block: &str) -> (Vec<String>, Vec<String>) {
    let inner = strip_block_wrapper(block);

    let mut forward_decls = Vec::new();
    let mut fn_items = Vec::new();
    let mut current_item = String::new();
    let mut in_item = false;
    // Buffer for accumulating consecutive comment lines.
    let mut comment_buf = String::new();

    for line in inner.lines() {
        let trimmed = line.trim();

        // Skip the link_name inner attribute.
        if trimmed.starts_with("#![link_name") {
            continue;
        }

        if trimmed.starts_with("//") {
            // Accumulate comment lines; we decide what to keep once the block ends.
            if !comment_buf.is_empty() {
                comment_buf.push('\n');
            }
            comment_buf.push_str(trimmed);
            continue;
        }

        // Flush the comment buffer now that we've hit a non-comment line.
        // Preserve @make_proxy skeleton comment blocks so they survive merge.
        if !comment_buf.is_empty() {
            if comment_buf.contains("@make_proxy skeleton") {
                fn_items.push(comment_buf.clone());
            }
            comment_buf.clear();
        }

        if trimmed.starts_with("class ") && trimmed.ends_with(';') && !trimmed.contains("//") {
            forward_decls.push(trimmed.to_string());
            continue;
        }

        if trimmed.is_empty() && !in_item {
            continue;
        }

        // Any attribute line starts a new fn item.
        let starts_item = trimmed.starts_with("#[cpp(func")
            || trimmed.starts_with("#[cpp(data")
            || trimmed.starts_with("#[interface(")
            || trimmed.starts_with("#[member(");

        if starts_item || (in_item && !trimmed.is_empty()) {
            in_item = true;
            if !current_item.is_empty() {
                current_item.push('\n');
            }
            current_item.push_str(trimmed);

            if trimmed.ends_with(';') {
                fn_items.push(current_item.trim().to_string());
                current_item.clear();
                in_item = false;
            }
        }
    }

    // Flush any trailing comment block.
    if !comment_buf.is_empty() && comment_buf.contains("@make_proxy skeleton") {
        fn_items.push(comment_buf);
    }

    (forward_decls, fn_items)
}

fn strip_block_wrapper(block: &str) -> &str {
    let open = block.find('{').unwrap_or(0);
    let close = block.rfind('}').unwrap_or(block.len());
    &block[open + 1..close]
}

mod indexmap {
    #[derive(Default)]
    pub struct IndexSet<T> {
        items: Vec<T>,
    }

    impl<T: Eq + Clone> IndexSet<T> {
        pub fn insert(&mut self, value: T) -> bool {
            if !self.items.contains(&value) {
                self.items.push(value);
                true
            } else {
                false
            }
        }

        pub fn iter(&self) -> impl Iterator<Item = &T> {
            self.items.iter()
        }

        pub fn is_empty(&self) -> bool {
            self.items.is_empty()
        }
    }

    /// Insertion-ordered map that keeps the **first** value inserted for any
    /// given key (subsequent inserts with the same key are silently ignored).
    ///
    /// Used to deduplicate `import_class!` blocks by Rust struct name so that
    /// if the same class is extracted from multiple places in the AST, only
    /// the first occurrence appears in the merged output.
    #[derive(Default)]
    pub struct IndexMap<K, V> {
        items: Vec<(K, V)>,
    }

    impl<K: Eq + Clone, V: Clone> IndexMap<K, V> {
        /// Insert `(key, value)` if `key` is not already present.
        /// Returns `true` when the entry was newly inserted.
        pub fn insert(&mut self, key: K, value: V) -> bool {
            if self.items.iter().any(|(k, _)| k == &key) {
                return false;
            }
            self.items.push((key, value));
            true
        }

        /// Iterate over `(key, value)` pairs in insertion order.
        pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
            self.items.iter().map(|(k, v)| (k, v))
        }

        /// Iterate over values in insertion order.
        pub fn values(&self) -> impl Iterator<Item = &V> {
            self.items.iter().map(|(_, v)| v)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_import_class_block() {
        let src = r#"
hicc::import_class! {
    #[cpp(class = "Widget")]
    class Widget {
        #[cpp(method = "void update(double, double)")]
        fn update(&mut self, x: f64, y: f64);
    }
}
"#;
        let blocks = extract_import_class_blocks(src);
        assert_eq!(blocks.len(), 1);
        assert!(blocks[0].contains("Widget"));
    }

    #[test]
    fn extract_import_lib_block() {
        let src = r#"
hicc::import_lib! {
    #![link_name = "mylib"]

    class Widget;

    #[cpp(func = "int add(int, int)")]
    fn add(a: i32, b: i32) -> i32;
}
"#;
        let blocks = extract_import_lib_blocks(src);
        assert_eq!(blocks.len(), 1);

        let (fwd, fns) = parse_lib_block_contents(&blocks[0]);
        assert_eq!(fwd, vec!["class Widget;"]);
        assert_eq!(fns.len(), 1);
        assert!(fns[0].contains("fn add"));
    }

    #[test]
    fn merge_deduplicates_forward_decls() {
        let src1 = r#"
hicc::import_lib! {
    #![link_name = "mylib"]

    class Widget;

    #[cpp(func = "int foo()")]
    fn foo() -> i32;
}
"#;
        let src2 = r#"
hicc::import_lib! {
    #![link_name = "mylib"]

    class Widget;

    #[cpp(func = "int bar()")]
    fn bar() -> i32;
}
"#;
        let mut fwd: indexmap::IndexSet<String> = Default::default();
        let mut fns: Vec<String> = Vec::new();
        for block in extract_import_lib_blocks(src1)
            .iter()
            .chain(extract_import_lib_blocks(src2).iter())
        {
            let (f, i) = parse_lib_block_contents(block);
            for fd in f {
                fwd.insert(fd);
            }
            fns.extend(i);
        }
        let fwd_vec: Vec<&String> = fwd.iter().collect();
        assert_eq!(fwd_vec.len(), 1, "Widget should appear only once");
        assert_eq!(fns.len(), 2);
    }

    /// Regression test: two `import_class!` blocks for the same Rust struct
    /// name must be deduplicated in the merged output.
    ///
    /// This reproduces the `error[E0428]: the name 'other' is defined multiple times`
    /// error that occurs when the same `ClassTemplateSpecializationDecl`
    /// appears in both a `ClassTemplateDecl` child position and as a
    /// standalone top-level node, causing the method codegen to emit two
    /// identical `class other { ... }` blocks that the merge naively
    /// concatenates.
    #[test]
    fn merge_deduplicates_import_class_blocks_by_class_name() {
        let block1 = r#"hicc::import_class! {
    #[cpp(class = "new_allocator<_Tp1>", ctor = "other()")]
    class other {
        #[cpp(method = "void deallocate(char *, unsigned long)")]
        fn deallocate(&mut self, p: *mut i8, n: usize);
    }
}"#;
        let block2 = r#"hicc::import_class! {
    #[cpp(class = "new_allocator<_Tp1>", ctor = "other()")]
    class other {
        #[cpp(method = "void deallocate(char *, unsigned long)")]
        fn deallocate(&mut self, p: *mut i8, n: usize);
    }
}"#;

        let mut fragments = ModuleFragments::default();
        for block in [block1, block2] {
            let key = class_name_from_block(block)
                .expect("test block must contain a `class <Name>` line");
            fragments.import_class_blocks.insert(key, block.to_string());
        }

        let rendered = render_merged_module(&fragments, None, "mylib", false);

        // Count occurrences of `class other` in the rendered output.
        let count = rendered.matches("class other").count();
        assert_eq!(
            count, 1,
            "`class other` must appear exactly once in merged output; \
             rendered:\n{rendered}"
        );
    }

    /// `class_name_from_block` must correctly extract the Rust struct name.
    #[test]
    fn test_class_name_from_block() {
        let block = r#"hicc::import_class! {
    #[cpp(class = "Widget")]
    class Widget {
        fn update(&mut self);
    }
}"#;
        assert_eq!(class_name_from_block(block), Some("Widget".to_string()));

        // Inheritance suffix.
        let block_with_bases = r#"hicc::import_class! {
    #[interface]
    class IFoo: IBase {
        fn method(&mut self);
    }
}"#;
        assert_eq!(
            class_name_from_block(block_with_bases),
            Some("IFoo".to_string())
        );

        // Abstract block with opening brace on same line.
        let block_brace = r#"hicc::import_class! {
    #[interface]
    class Bar {
    }
}"#;
        assert_eq!(class_name_from_block(block_brace), Some("Bar".to_string()));
    }

    /// `parse_lib_block_contents` must preserve `@make_proxy skeleton` comment
    /// blocks so they survive the merge step and appear in `lib.rs`.
    #[test]
    fn parse_lib_block_preserves_make_proxy_skeleton() {
        let src = r#"hicc::import_lib! {
    #![link_name = "mylib"]

    // @make_proxy skeleton for `IFoo` — uncomment and replace
    // `YourConcreteImpl` with a concrete C++ class that derives from it.
    // #[cpp(func = "YourConcreteImpl @make_proxy<YourConcreteImpl>()")]
    // #[interface(name = "IFoo")]
    // fn new_i_foo_proxy(intf: hicc::Interface<YourConcreteImpl>) -> YourConcreteImpl;

}"#;
        let blocks = extract_import_lib_blocks(src);
        assert_eq!(blocks.len(), 1);

        let (fwd, fns) = parse_lib_block_contents(&blocks[0]);
        assert!(fwd.is_empty(), "no forward decls expected");
        assert_eq!(
            fns.len(),
            1,
            "skeleton comment block should be preserved as one item"
        );
        assert!(
            fns[0].contains("make_proxy"),
            "preserved item should contain 'make_proxy': {:?}",
            fns[0]
        );
    }

    /// Regular comment lines (non-skeleton) must still be stripped during merge.
    #[test]
    fn parse_lib_block_strips_regular_comments() {
        let src = r#"hicc::import_lib! {
    #![link_name = "mylib"]

    // @make_proxy support: required for wrapping Rust structs as C++ interfaces.

    #[cpp(func = "int add(int, int)")]
    fn add(a: i32, b: i32) -> i32;
}"#;
        let blocks = extract_import_lib_blocks(src);
        let (fwd, fns) = parse_lib_block_contents(&blocks[0]);
        assert!(fwd.is_empty());
        // Only the real fn binding should remain; the support comment should be dropped.
        assert_eq!(fns.len(), 1);
        assert!(fns[0].contains("fn add"));
    }
}
