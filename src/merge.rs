//! `merge` command implementation.
//!
//! The `merge` command reads grouped semantic modules produced by `init`
//! (`mod_<group>/include|types|free|class|method`) and emits:
//!
//! 1. `rust/src.2/mod_<group>.rs` (merged per-group view)
//! 2. `rust/src.2/lib.rs`
//! 3. `rust/src.2/merged_ffi.rs` (global consolidated view)
//!
//! Current v1 merge semantics:
//! - `include/` contributes `hicc::cpp!` include lines
//! - `types/` contributes type semantic blocks (inventory + C++/Rust mappings)
//! - `method/` contributes `hicc::import_class!` method-binding blocks
//! - `free/` contributes `hicc::import_lib!` fn items and class forward declarations
//! - `class/` contributes class-level semantic structure blocks to merged outputs
//! - `common/*` contributes shared semantic blocks to global merged output
//! - `global/` is explicitly deferred outside the current v1 closure scope

use crate::error::Result;
use anyhow::anyhow;
use std::fs;
use std::path::{Path, PathBuf};

const TYPES_PLACEHOLDER_COMMENT: &str = "// Type helpers";

#[derive(Debug)]
pub struct MergeOutput {
    pub merged_path: PathBuf,
    pub group_modules: Vec<String>,
}

#[derive(Default)]
struct ModuleFragments {
    includes: indexmap::IndexSet<String>,
    import_class_blocks: Vec<String>,
    forward_decls: indexmap::IndexSet<String>,
    fn_items: Vec<String>,
    type_blocks: indexmap::IndexSet<String>,
    class_blocks: indexmap::IndexSet<String>,
}

#[derive(Default)]
struct CommonFragments {
    includes_block: Option<String>,
    types_block: Option<String>,
}

pub fn merge_grouped_modules(
    init_src_dir: &Path,
    out_src2_dir: &Path,
    link_name: &str,
) -> Result<MergeOutput> {
    let mut group_dirs: Vec<PathBuf> = fs::read_dir(init_src_dir)
        .map_err(|e| anyhow!("read dir {}: {}", init_src_dir.display(), e))?
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|p| {
            p.is_dir()
                && p.file_name()
                    .and_then(|n| n.to_str())
                    .map(|n| n.starts_with("mod_"))
                    .unwrap_or(false)
        })
        .collect();
    group_dirs.sort();

    if group_dirs.is_empty() {
        return Err(anyhow!(
            "no mod_<group> directories found in {}; run 'init' first",
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
    let common = load_common_fragments(init_src_dir)?;
    let mut group_modules = Vec::new();

    for group_dir in &group_dirs {
        let group_name = group_dir
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| anyhow!("invalid group directory name: {}", group_dir.display()))?
            .to_string();
        let out_file = out_src2_dir.join(format!("{group_name}.rs"));
        let fragments = merge_group_module(group_dir, &out_file, link_name)?;

        for inc in fragments.includes.iter() {
            merged_all.includes.insert(inc.clone());
        }
        for block in &fragments.import_class_blocks {
            merged_all.import_class_blocks.push(block.clone());
        }
        for decl in fragments.forward_decls.iter() {
            merged_all.forward_decls.insert(decl.clone());
        }
        merged_all
            .fn_items
            .extend(fragments.fn_items.iter().cloned());
        for class_block in fragments.class_blocks.iter() {
            merged_all.class_blocks.insert(class_block.clone());
        }

        group_modules.push(group_name);
    }

    let mod_refs: Vec<&str> = group_modules.iter().map(|s| s.as_str()).collect();
    fs::write(
        out_src2_dir.join("lib.rs"),
        crate::codegen::render_lib_rs(&mod_refs),
    )
    .map_err(|e| anyhow!("write {}/lib.rs: {}", out_src2_dir.display(), e))?;

    let merged_src = render_merged_module(&merged_all, &common, link_name, true);
    let merged_path = out_src2_dir.join("merged_ffi.rs");
    fs::write(&merged_path, merged_src)
        .map_err(|e| anyhow!("write {}: {}", merged_path.display(), e))?;

    Ok(MergeOutput {
        merged_path,
        group_modules,
    })
}

fn merge_group_module(
    group_dir: &Path,
    output_file: &Path,
    link_name: &str,
) -> Result<ModuleFragments> {
    let mut fragments = ModuleFragments::default();
    for semantic_dir in crate::SEMANTIC_DIRS {
        let dir = group_dir.join(semantic_dir);
        if !dir.exists() {
            continue;
        }
        let mut rs_files: Vec<PathBuf> = fs::read_dir(&dir)
            .map_err(|e| anyhow!("read dir {}: {}", dir.display(), e))?
            .filter_map(|entry| entry.ok().map(|e| e.path()))
            .filter(|p| p.is_file() && p.extension().and_then(|e| e.to_str()) == Some("rs"))
            .collect();
        rs_files.sort();

        for file in &rs_files {
            let src =
                fs::read_to_string(file).map_err(|e| anyhow!("read {}: {}", file.display(), e))?;

            if semantic_dir == crate::SEMANTIC_TYPES_DIR
                && file.file_name().and_then(|n| n.to_str()) == Some("mod.rs")
            {
                let trimmed = src.trim();
                if !trimmed.is_empty() && !trimmed.starts_with(TYPES_PLACEHOLDER_COMMENT) {
                    fragments.type_blocks.insert(trimmed.to_string());
                }
            }
            if semantic_dir == crate::SEMANTIC_CLASS_DIR
                && file.file_name().and_then(|n| n.to_str()) != Some("mod.rs")
            {
                let trimmed = src.trim();
                if !trimmed.is_empty() {
                    fragments.class_blocks.insert(trimmed.to_string());
                }
            }

            for include in extract_cpp_includes(&src) {
                fragments.includes.insert(include);
            }
            for block in extract_import_class_blocks(&src) {
                fragments.import_class_blocks.push(block);
            }
            for block in extract_import_lib_blocks(&src) {
                let (fwd, fns) = parse_lib_block_contents(&block);
                for f in fwd {
                    fragments.forward_decls.insert(f);
                }
                fragments.fn_items.extend(fns);
            }
        }
    }

    let rendered = render_merged_module(&fragments, &CommonFragments::default(), link_name, false);
    fs::write(output_file, rendered)
        .map_err(|e| anyhow!("write {}: {}", output_file.display(), e))?;

    Ok(fragments)
}

fn render_merged_module(
    fragments: &ModuleFragments,
    common: &CommonFragments,
    link_name: &str,
    is_global: bool,
) -> String {
    let mut out = String::new();
    if is_global {
        out.push_str("// Merged FFI – auto-generated by `cpp2rust-demo merge`.\n");
    } else {
        out.push_str("// Group-merged FFI – auto-generated by `cpp2rust-demo merge`.\n");
    }
    out.push_str("#![allow(non_snake_case, dead_code)]\n\n");

    if is_global {
        if let Some(includes_block) = &common.includes_block {
            out.push_str("// Shared include semantics from init common/includes.rs\n");
            out.push_str(includes_block.trim());
            out.push_str("\n\n");
        }
        if let Some(types_block) = &common.types_block {
            out.push_str("// Shared type semantics from init common/types.rs\n");
            out.push_str(types_block.trim());
            out.push_str("\n\n");
        }
    }

    if !fragments.includes.is_empty() {
        out.push_str("hicc::cpp! {\n");
        for include in fragments.includes.iter() {
            out.push_str(&format!("    {}\n", include));
        }
        out.push_str("}\n\n");
    }

    for block in fragments.type_blocks.iter() {
        out.push_str(block.trim());
        out.push_str("\n\n");
    }

    for block in fragments.class_blocks.iter() {
        out.push_str(block.trim());
        out.push_str("\n\n");
    }

    for block in &fragments.import_class_blocks {
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

fn load_common_fragments(init_src_dir: &Path) -> Result<CommonFragments> {
    let common_dir = init_src_dir.join("common");
    if !common_dir.exists() {
        return Ok(CommonFragments::default());
    }

    let includes_path = common_dir.join("includes.rs");
    let types_path = common_dir.join("types.rs");

    let includes_block = if includes_path.exists() {
        Some(
            fs::read_to_string(&includes_path)
                .map_err(|e| anyhow!("read {}: {}", includes_path.display(), e))?,
        )
    } else {
        None
    };

    let types_block = if types_path.exists() {
        Some(
            fs::read_to_string(&types_path)
                .map_err(|e| anyhow!("read {}: {}", types_path.display(), e))?,
        )
    } else {
        None
    };

    Ok(CommonFragments {
        includes_block,
        types_block,
    })
}

// ---------------------------------------------------------------------------
// Simple text extraction helpers
// ---------------------------------------------------------------------------

/// Extract `hicc::import_class! { ... }` blocks from `src`.
fn extract_import_class_blocks(src: &str) -> Vec<String> {
    extract_macro_blocks(src, "hicc::import_class!")
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

    for line in inner.lines() {
        let trimmed = line.trim();

        // Skip the link_name inner attribute.
        if trimmed.starts_with("#![link_name") {
            continue;
        }

        // Skip comment lines (including the @make_proxy notice and global var header).
        if trimmed.starts_with("//") {
            continue;
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
}
