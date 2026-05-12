mod ast;
mod capture;
mod codegen;
mod error;
mod layout;
mod merge;
mod selector;

use crate::error::Result;
use anyhow::anyhow;
use clap::{Args, Parser, Subcommand};
use selector::{FileSelector, InteractiveSelector};
use serde::Serialize;
use std::path::{Path, PathBuf};

pub(crate) const SEMANTIC_TYPES_DIR: &str = "types";
pub(crate) const SEMANTIC_INCLUDE_DIR: &str = "include";
pub(crate) const SEMANTIC_FREE_DIR: &str = "free";
pub(crate) const SEMANTIC_CLASS_DIR: &str = "class";
pub(crate) const SEMANTIC_METHOD_DIR: &str = "method";
pub(crate) const SEMANTIC_DIRS: [&str; 5] = [
    SEMANTIC_TYPES_DIR,
    SEMANTIC_INCLUDE_DIR,
    SEMANTIC_FREE_DIR,
    SEMANTIC_CLASS_DIR,
    SEMANTIC_METHOD_DIR,
];

// ---------------------------------------------------------------------------
// CLI definition
// ---------------------------------------------------------------------------

#[derive(Parser)]
#[command(name = "cpp2rust-demo")]
#[command(about = "C++ to Rust FFI generation via clang AST JSON + hicc")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Capture real build commands and generate hicc FFI scaffolding.
    ///
    /// Example:
    ///   cpp2rust-demo init --link mylib -- make -j4
    Init(InitArgs),

    /// Merge per-file FFI files into a single consolidated file.
    ///
    /// Example:
    ///   cpp2rust-demo merge --feature default
    Merge(MergeArgs),
}

#[derive(Args)]
struct InitArgs {
    /// Feature name.
    #[arg(long, default_value = "default")]
    feature: String,

    /// Name of the C++ shared/static library to link against.
    /// Used as the `link_name` in `hicc::import_lib!`.
    #[arg(long)]
    link: String,

    /// Header-only / no-link mode.
    /// Skips linking the external target library in generated `build.rs`.
    #[arg(long = "no-link", alias = "header-only")]
    no_link: bool,

    /// Extra arguments forwarded to clang (e.g. `-std=c++17 -I./include`).
    #[arg(long = "extra-clang-args", value_name = "ARGS")]
    extra_clang_args: Option<String>,

    /// The `clang` binary to use.  Defaults to the `CPP2RUST_CLANG` env var or `clang`.
    #[arg(long, env = "CPP2RUST_CLANG", default_value = "clang")]
    clang: String,

    /// Build command to execute (use after `--`).
    /// Example: cpp2rust-demo init --link mylib -- make -j4
    #[arg(
        trailing_var_arg = true,
        allow_hyphen_values = true,
        required = true,
        value_name = "BUILD_CMD"
    )]
    build_cmd: Vec<String>,
}

#[derive(Args)]
struct MergeArgs {
    /// Feature name (must match a previous `init` run).
    #[arg(long, default_value = "default")]
    feature: String,
}

// ---------------------------------------------------------------------------
// Command implementations
// ---------------------------------------------------------------------------

fn run_init(args: InitArgs) -> Result<()> {
    let feature = &args.feature;
    let link_name = &args.link;
    let no_link = args.no_link;
    let build_cmd = &args.build_cmd;

    let cwd = std::env::current_dir().map_err(|e| anyhow!("current_dir: {}", e))?;
    let project_root = layout::find_project_root(&cwd);

    println!("=== cpp2rust-demo init ===");
    println!("Project root : {}", project_root.display());
    println!("Feature      : {}", feature);
    println!("Link name    : {}", link_name);
    println!(
        "Link mode    : {}",
        if no_link {
            "header-only/no-link"
        } else {
            "normal"
        }
    );
    println!("Build command: {}", build_cmd.join(" "));
    println!();

    // Create layout directories.
    let lo = layout::FeatureLayout::new(project_root.clone(), feature);
    lo.create_dirs()?;
    lo.save_build_cmd(build_cmd)?;
    // Parse extra clang args.
    let extra_args: Vec<String> = args
        .extra_clang_args
        .as_deref()
        .map(|s| s.split_whitespace().map(|w| w.to_string()).collect())
        .unwrap_or_default();

    // Build hook and run real build command under LD_PRELOAD capture.
    let hook_so = capture::build_hook()?;
    capture::run_with_hook(&cwd, build_cmd, &project_root, &lo.feature_root, &hook_so)?;

    let captured_files = layout::scan_cpp2rust_files(&lo.cpp_dir)?;
    if captured_files.is_empty() {
        return Err(anyhow!(
            "{}",
            concat!(
                "preload hook did not capture any *.cpp2rust middleware files from build command; ",
                "ensure the build command really compiles C++ translation units under the project root"
            )
        ));
    }
    println!(
        "Captured {} middleware file(s) via LD_PRELOAD hook.",
        captured_files.len()
    );

    // ----------------------------------------------------------------
    // Interactive middleware file selection
    // (auto-selects all when stdin is not a terminal, e.g. in CI/scripts)
    // ----------------------------------------------------------------
    let sel = InteractiveSelector;
    let selected_files = sel.select(&captured_files)?;
    println!("{} file(s) selected for this feature", selected_files.len());

    lo.save_selected_files(&selected_files)?;

    if selected_files.is_empty() {
        println!("No middleware files selected – skipping FFI generation.");
        return Ok(());
    }

    let files_to_process = selected_files;

    lo.save_meta(&files_to_process, link_name, no_link)?;

    // Create the Rust project skeleton.
    let rust_src_dir = lo.rust_dir.join("src");
    std::fs::create_dir_all(&rust_src_dir)
        .map_err(|e| anyhow!("create {}: {}", rust_src_dir.display(), e))?;

    // Compute deterministic names for middleware files and grouped module directories.
    let stems: Vec<String> = middleware_stems(&files_to_process);
    let group_modules: Vec<String> = middleware_group_modules(&lo.cpp_dir, &files_to_process);

    // Write Cargo.toml, build.rs, lib.rs for the generated crate.
    let crate_name = format!("cpp2rust-{}-ffi", feature.replace('_', "-"));
    let cargo_toml_path = lo.rust_dir.join("Cargo.toml");
    if !cargo_toml_path.exists() {
        std::fs::write(
            &cargo_toml_path,
            codegen::render_cargo_toml(&crate_name, link_name),
        )
        .map_err(|e| anyhow!("write Cargo.toml: {}", e))?;
        println!("Created {}", cargo_toml_path.display());
    }

    // Prepare shared/common module scaffolding.
    write_common_modules(&rust_src_dir, "", "")?;

    // Process each selected middleware file.
    let mut all_decls = ast::ExtractedDecls::default();
    let mut report_sections: Vec<String> = Vec::new();
    let mut build_rs_sources: Vec<String> = Vec::new();
    let mut lib_modules: Vec<String> = vec!["common".to_string()];

    for ((selected_file, stem), group_module) in files_to_process
        .iter()
        .zip(stems.iter())
        .zip(group_modules.iter())
    {
        println!("Processing {}...", selected_file.display());

        // Step 1: dump AST via clang.
        let ast_root = ast::dump_ast(selected_file, &extra_args, &args.clang)?;

        // Save the AST JSON for debugging.
        let ast_json_path = lo.ast_dir.join(format!("{}.ast.json", stem));
        let ast_json =
            serde_json::to_string(&ast_root).map_err(|e| anyhow!("serialize AST: {}", e))?;
        std::fs::write(&ast_json_path, &ast_json).map_err(|e| anyhow!("write AST JSON: {}", e))?;
        println!("  AST saved → {}", ast_json_path.display());

        // Step 2: extract declarations.
        let file_paths: Vec<&Path> = vec![selected_file.as_path()];
        let decls = ast::extract_declarations(&ast_root, &file_paths);

        println!(
            "  Found {} free function(s), {} class(es)",
            decls.functions.len(),
            decls.classes.len()
        );

        if decls.functions.is_empty() && decls.classes.is_empty() {
            println!("  Warning: no declarations found from this file.");
        }

        // Step 3: generate grouped semantic module source.
        let group_dir = rust_src_dir.join(group_module);
        write_group_scaffold(&group_dir, stem)?;

        let include_mod_path = group_dir.join("include").join("mod.rs");
        let include_src = codegen::render_include_module(&selected_file.display().to_string());
        std::fs::write(&include_mod_path, include_src)
            .map_err(|e| anyhow!("write {}: {}", include_mod_path.display(), e))?;

        // `class/` is the class-level semantic structure layer in v1.
        // Binding macros for instance methods are emitted under `method/`.
        let class_mod_name = format!("cls_{}", stem);
        let class_file_path = group_dir.join("class").join(format!("{class_mod_name}.rs"));
        let class_src = codegen::render_class_module(&decls);
        let has_class = !class_src.trim().is_empty();
        if has_class {
            std::fs::write(&class_file_path, class_src)
                .map_err(|e| anyhow!("write {}: {}", class_file_path.display(), e))?;
            std::fs::write(
                group_dir.join("class").join("mod.rs"),
                codegen::render_lib_rs(&[&class_mod_name]),
            )
            .map_err(|e| anyhow!("write class/mod.rs: {}", e))?;
        }

        // `method/` is the sole layer that carries `hicc::import_class!` blocks in v1.
        let method_mod_name = format!("mtd_{}", stem);
        let method_file_path = group_dir
            .join("method")
            .join(format!("{method_mod_name}.rs"));
        let method_src = codegen::render_method_module(&decls);
        let has_method = !method_src.trim().is_empty();
        if has_method {
            std::fs::write(&method_file_path, method_src)
                .map_err(|e| anyhow!("write {}: {}", method_file_path.display(), e))?;
            std::fs::write(
                group_dir.join("method").join("mod.rs"),
                codegen::render_lib_rs(&[&method_mod_name]),
            )
            .map_err(|e| anyhow!("write method/mod.rs: {}", e))?;
        }

        let free_mod_name = format!("fn_{}", stem);
        let free_file_path = group_dir.join("free").join(format!("{free_mod_name}.rs"));
        let has_free = has_free_bindings(&decls);
        if has_free {
            let free_src = codegen::render_free_module(&decls, link_name);
            std::fs::write(&free_file_path, free_src)
                .map_err(|e| anyhow!("write {}: {}", free_file_path.display(), e))?;
            std::fs::write(
                group_dir.join("free").join("mod.rs"),
                codegen::render_lib_rs(&[&free_mod_name]),
            )
            .map_err(|e| anyhow!("write free/mod.rs: {}", e))?;
        }

        // `types/` is generated as per-group type semantics (inventory + mappings).
        let types_src = codegen::render_types_module(&decls);
        std::fs::write(group_dir.join("types").join("mod.rs"), types_src)
            .map_err(|e| anyhow!("write types/mod.rs: {}", e))?;

        // 主线四: operator shims – write C++ shim header to meta/ and Rust stubs to free/.
        if !decls.operator_shims.is_empty() {
            let middleware_basename = selected_file
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("middleware.hpp");
            let shims_hpp = codegen::render_operator_shims_hpp(&decls.operator_shims, middleware_basename);
            let shims_hpp_path = lo.meta_dir.join("operator_shims.hpp");
            std::fs::write(&shims_hpp_path, &shims_hpp)
                .map_err(|e| anyhow!("write operator_shims.hpp: {}", e))?;
            let shims_rs = codegen::render_operator_shims_rs(&decls.operator_shims, link_name);
            let shims_rs_path = group_dir.join("free").join("shim_ops.rs");
            std::fs::write(&shims_rs_path, &shims_rs)
                .map_err(|e| anyhow!("write shim_ops.rs: {}", e))?;
            println!("  Operator shims → {}", shims_hpp_path.display());
        }

        let group_mod_path = group_dir.join("mod.rs");
        std::fs::write(
            &group_mod_path,
            render_group_mod_rs(has_free, has_class, has_method),
        )
        .map_err(|e| anyhow!("write {}: {}", group_mod_path.display(), e))?;

        let group_meta = GroupMeta {
            group: group_module.to_string(),
            middleware: selected_file.display().to_string(),
            ast: ast_json_path.display().to_string(),
            free_functions: decls
                .functions
                .iter()
                .map(|f| f.qualified_name.clone())
                .collect(),
            classes: decls
                .classes
                .iter()
                .map(|c| c.qualified_name.clone())
                .collect(),
            methods: decls
                .classes
                .iter()
                .flat_map(|c| c.methods.iter().map(|m| m.qualified_name.clone()))
                .collect(),
            globals: decls
                .globals
                .iter()
                .map(|g| g.qualified_name.clone())
                .collect(),
        };
        let meta_path = group_dir.join("meta.json");
        std::fs::write(
            &meta_path,
            serde_json::to_string_pretty(&group_meta)
                .map_err(|e| anyhow!("serialize meta: {}", e))?,
        )
        .map_err(|e| anyhow!("write {}: {}", meta_path.display(), e))?;

        build_rs_sources.push(format!("src/{}/include/mod.rs", group_module));
        if has_free {
            build_rs_sources.push(format!("src/{}/free/{}.rs", group_module, free_mod_name));
        }
        if has_class {
            build_rs_sources.push(format!("src/{}/class/{}.rs", group_module, class_mod_name));
        }
        if has_method {
            build_rs_sources.push(format!(
                "src/{}/method/{}.rs",
                group_module, method_mod_name
            ));
        }
        build_rs_sources.push(format!("src/{}/types/mod.rs", group_module));
        lib_modules.push(group_module.to_string());

        println!("  Grouped module → {}", group_dir.display());

        // Accumulate for the consolidated report.
        let report_section = codegen::render_interface_report(
            &decls,
            link_name,
            &selected_file.display().to_string(),
        );
        report_sections.push(report_section);

        // Merge into all_decls for the report.
        all_decls.functions.extend(decls.functions);
        all_decls.classes.extend(decls.classes);
        all_decls.globals.extend(decls.globals);
        all_decls.skipped.extend(decls.skipped);
        all_decls.operator_shims.extend(decls.operator_shims);
    }

    // Write interface report.
    let report_path = lo.meta_dir.join("init-interface-report.md");
    let report = report_sections.join("\n---\n\n");
    std::fs::write(&report_path, &report).map_err(|e| anyhow!("write report: {}", e))?;
    println!("\nInterface report → {}", report_path.display());

    // build.rs: point hicc-build to grouped semantic files.
    {
        let build_rs_path = lo.rust_dir.join("build.rs");
        let source_refs: Vec<&str> = build_rs_sources.iter().map(|s| s.as_str()).collect();
        let include_dirs = middleware_include_dirs(&files_to_process);
        let inc_refs: Vec<&str> = include_dirs.iter().map(|s| s.as_str()).collect();
        std::fs::write(
            &build_rs_path,
            codegen::render_build_rs(link_name, no_link, &source_refs, &inc_refs),
        )
        .map_err(|e| anyhow!("write build.rs: {}", e))?;
        println!("Created {}", build_rs_path.display());

        // `common/*` carries shared include/type semantics derived from selected
        // middleware and is propagated into global merged output for reuse.
        let common_includes = render_common_includes_module(&files_to_process, &include_dirs);
        let common_types = codegen::render_types_module(&all_decls);
        write_common_modules(&rust_src_dir, &common_includes, &common_types)?;
    }

    // lib.rs: expose common + grouped modules.
    {
        let lib_rs_path = rust_src_dir.join("lib.rs");
        let module_refs: Vec<&str> = lib_modules.iter().map(|s| s.as_str()).collect();
        std::fs::write(&lib_rs_path, codegen::render_lib_rs(&module_refs))
            .map_err(|e| anyhow!("write lib.rs: {}", e))?;
        println!("Created {}", lib_rs_path.display());
    }

    println!("\n✓ cpp2rust-demo init completed successfully!");
    println!("\nOutput structure:");
    println!("  .cpp2rust/{}/", feature);
    println!("    ├── cpp/        (captured preprocessed middleware: *.cpp2rust)");
    println!("    ├── ast/        (clang AST JSON per selected file)");
    println!("    ├── meta/       (build_cmd.txt, selected_files.json, headers.json, init-interface-report.md)");
    println!("    └── rust/       (generated Rust project)");
    println!("        ├── Cargo.toml");
    println!("        ├── build.rs");
    println!("        └── src/");
    println!("            ├── lib.rs");
    println!("            ├── common/...");
    println!("            └── mod_<group>/include|types|free|class|method + meta.json");
    println!();
    println!("Next steps:");
    println!("  1. Review .cpp2rust/{}/rust/src/mod_<group>/", feature);
    println!("  2. Run `cpp2rust-demo merge` to consolidate into src.2");
    println!("  3. Copy the Rust project to your workspace and add the C++ library");

    Ok(())
}

fn run_merge(args: MergeArgs) -> Result<()> {
    let feature = &args.feature;

    let cwd = std::env::current_dir().map_err(|e| anyhow!("current_dir: {}", e))?;
    let project_root = layout::find_project_root(&cwd);

    println!("=== cpp2rust-demo merge ===");
    println!("Project root : {}", project_root.display());
    println!("Feature      : {}", feature);
    println!();

    let lo = layout::FeatureLayout::new(project_root.clone(), feature);

    if !lo.feature_root.exists() {
        return Err(anyhow!(
            "feature '{}' not found at {}; run 'init' first",
            feature,
            lo.feature_root.display()
        ));
    }

    let (link_name, no_link, stored_files) = lo.load_meta()?;

    let rust_src_dir = lo.rust_dir.join("src");
    if !rust_src_dir.exists() {
        return Err(anyhow!(
            "rust/src not found at {}; run 'init' first",
            rust_src_dir.display()
        ));
    }

    let merged_src2 = lo.rust_dir.join("src.2");
    let merged = merge::merge_grouped_modules(&rust_src_dir, &merged_src2, &link_name)?;

    // Recompute unique include dirs from stored selected files.
    let include_dirs = middleware_include_dirs(&stored_files);
    let inc_refs: Vec<&str> = include_dirs.iter().map(|s| s.as_str()).collect();

    // Update build.rs to reference merged_ffi.rs through the active view path `src/`.
    // After link_src_to_src2(), `rust/src` is a symlink to `rust/src.2`, so keeping
    // this path stable means build.rs always targets the current active source view.
    let build_rs_path = lo.rust_dir.join("build.rs");
    std::fs::write(
        &build_rs_path,
        codegen::render_build_rs(&link_name, no_link, &["src/merged_ffi.rs"], &inc_refs),
    )
    .map_err(|e| anyhow!("update build.rs: {}", e))?;
    println!("  Updated {}", build_rs_path.display());

    link_src_to_src2(&lo.rust_dir)?;

    // Write a merge report.
    let report_path = lo.meta_dir.join("merge-report.md");
    let report = format!(
        "# Merge Report\n\nFeature: `{feature}`\nLink name: `{link_name}`\n\nMerged groups: {}\n\nMerged output: `{}`\n",
        merged.group_modules.len(),
        merged.merged_path.display()
    );
    std::fs::write(&report_path, &report).map_err(|e| anyhow!("write merge report: {}", e))?;

    println!("\n✓ cpp2rust-demo merge completed successfully!");
    println!("\nMerged output:");
    println!("  {}", merged.merged_path.display());
    println!("\nThe merged output now lives under rust/src.2 (with rust/src -> src.2).");
    println!("build.rs keeps using src/... paths so it always targets the active source view.");
    println!(
        "It combines grouped include/method/free binding content plus types/class/common semantic inventories."
    );
    println!();
    println!("To use in your project:");
    println!("  1. Copy .cpp2rust/{}/rust/ to your workspace", feature);
    println!("  2. Add it as a Cargo dependency or inline the merged_ffi.rs");
    println!("  3. Adjust build.rs to point to your C++ library");

    Ok(())
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Collect unique parent directories from a list of middleware file paths, sorted for
/// deterministic output.  Used to populate `build.include(...)` calls in the
/// generated `build.rs` so hicc-build can find the `#include`d middleware files.
fn middleware_include_dirs(middleware_files: &[PathBuf]) -> Vec<String> {
    let mut dirs: Vec<String> = middleware_files
        .iter()
        .filter_map(|file| file.parent().map(|p| p.display().to_string()))
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    dirs.sort();
    dirs
}

#[derive(Serialize)]
struct GroupMeta {
    group: String,
    middleware: String,
    ast: String,
    free_functions: Vec<String>,
    classes: Vec<String>,
    methods: Vec<String>,
    globals: Vec<String>,
}

fn write_common_modules(rust_src_dir: &Path, includes_src: &str, types_src: &str) -> Result<()> {
    let common_dir = rust_src_dir.join("common");
    std::fs::create_dir_all(&common_dir)
        .map_err(|e| anyhow!("create {}: {}", common_dir.display(), e))?;
    std::fs::write(
        common_dir.join("mod.rs"),
        "pub mod includes;\npub mod types;\n",
    )
    .map_err(|e| anyhow!("write common/mod.rs: {}", e))?;
    std::fs::write(common_dir.join("includes.rs"), includes_src)
        .map_err(|e| anyhow!("write common/includes.rs: {}", e))?;
    std::fs::write(common_dir.join("types.rs"), types_src)
        .map_err(|e| anyhow!("write common/types.rs: {}", e))?;
    Ok(())
}

fn write_group_scaffold(group_dir: &Path, stem: &str) -> Result<()> {
    for sub in SEMANTIC_DIRS {
        std::fs::create_dir_all(group_dir.join(sub))
            .map_err(|e| anyhow!("create {}: {}", group_dir.join(sub).display(), e))?;
    }
    std::fs::write(
        group_dir.join("types").join("mod.rs"),
        format!("// Source stem: {}\n", stem),
    )
    .map_err(|e| anyhow!("write types/mod.rs: {}", e))?;
    std::fs::write(group_dir.join("method").join("mod.rs"), "")
        .map_err(|e| anyhow!("write method/mod.rs: {}", e))?;
    Ok(())
}

fn render_group_mod_rs(has_free: bool, has_class: bool, has_method: bool) -> String {
    let mut out = String::from("pub mod include;\npub mod types;\n");
    if has_free {
        out.push_str("pub mod free;\n");
    }
    if has_class {
        out.push_str("pub mod class;\n");
    }
    if has_method {
        out.push_str("pub mod method;\n");
    }
    if has_free {
        out.push_str("pub use free::*;\n");
    }
    if has_class {
        out.push_str("pub use class::*;\n");
    }
    if has_method {
        out.push_str("pub use method::*;\n");
    }
    out
}

fn has_free_bindings(decls: &ast::ExtractedDecls) -> bool {
    !decls.functions.is_empty()
        || !decls.classes.is_empty()
        || !decls.globals.is_empty()
        || decls
            .classes
            .iter()
            .flat_map(|c| c.methods.iter())
            .any(|m| m.is_static)
}

fn render_common_includes_module(middleware_files: &[PathBuf], include_dirs: &[String]) -> String {
    let files: Vec<String> = middleware_files
        .iter()
        .map(|p| p.display().to_string())
        .collect();
    let basenames: Vec<String> = middleware_files
        .iter()
        .map(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or_default()
                .to_string()
        })
        .collect();
    let mut out =
        String::from("// Shared include context derived from selected middleware files.\n");
    out.push_str("pub const MIDDLEWARE_FILES: &[&str] = &[\n");
    for file in &files {
        out.push_str(&format!("    {:?},\n", file));
    }
    out.push_str("];\n\n");
    out.push_str("pub const MIDDLEWARE_BASENAMES: &[&str] = &[\n");
    for name in &basenames {
        out.push_str(&format!("    {:?},\n", name));
    }
    out.push_str("];\n\n");
    out.push_str("pub const MIDDLEWARE_FILE_BASENAME_PAIRS: &[(&str, &str)] = &[\n");
    for (file, name) in files.iter().zip(basenames.iter()) {
        out.push_str(&format!("    ({:?}, {:?}),\n", file, name));
    }
    out.push_str("];\n\n");
    out.push_str("pub const INCLUDE_DIRS: &[&str] = &[\n");
    for dir in include_dirs {
        out.push_str(&format!("    {:?},\n", dir));
    }
    out.push_str("];\n");
    out.push_str("pub const CPP_INCLUDE_LINES: &[&str] = &[\n");
    for name in &basenames {
        let include_line = format!("#include \"{}\"", name);
        out.push_str(&format!("    {:?},\n", include_line));
    }
    out.push_str("];\n\n");
    out.push_str(
        "pub fn include_line_for(basename: &str) -> Option<&'static str> {\n\
    MIDDLEWARE_BASENAMES\n\
        .iter()\n\
        .position(|name| *name == basename)\n\
        .map(|idx| CPP_INCLUDE_LINES[idx])\n\
}\n\
\n\
pub fn has_include_dir(dir: &str) -> bool {\n\
    INCLUDE_DIRS.iter().any(|d| *d == dir)\n\
}\n",
    );
    out
}

fn middleware_group_modules(cpp_dir: &Path, paths: &[PathBuf]) -> Vec<String> {
    use std::collections::HashMap;

    let mut counts: HashMap<String, usize> = HashMap::new();
    for path in paths {
        let group = middleware_group_module(cpp_dir, path);
        *counts.entry(group).or_insert(0) += 1;
    }

    paths
        .iter()
        .map(|path| {
            let group = middleware_group_module(cpp_dir, path);
            if counts.get(&group).copied().unwrap_or(0) <= 1 {
                group
            } else {
                format!("{}_{}", group, stable_short_path_hash(path))
            }
        })
        .collect()
}

fn middleware_group_module(cpp_dir: &Path, middleware_path: &Path) -> String {
    let rel = middleware_path
        .strip_prefix(cpp_dir)
        .unwrap_or(middleware_path);
    let mut tokens: Vec<String> = Vec::new();
    for component in rel.components() {
        let raw = component.as_os_str().to_string_lossy().to_string();
        if raw == "." || raw == ".." {
            continue;
        }
        tokens.push(raw);
    }
    if let Some(last) = tokens.last_mut() {
        let no_suffix = last.strip_suffix(".cpp2rust").unwrap_or(last).to_string();
        let stem = Path::new(&no_suffix)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or(&no_suffix)
            .to_string();
        *last = stem;
    }

    let normalized = tokens
        .into_iter()
        .map(|part| sanitize_module_part(&part))
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("_");
    if normalized.is_empty() {
        "mod_group".to_string()
    } else {
        format!("mod_{}", normalized)
    }
}

fn sanitize_module_part(part: &str) -> String {
    let mut out = String::with_capacity(part.len());
    let mut last_underscore = false;
    for ch in part.chars() {
        let mapped = if ch.is_ascii_alphanumeric() { ch } else { '_' };
        if mapped == '_' {
            if !last_underscore {
                out.push('_');
            }
            last_underscore = true;
        } else {
            out.push(mapped.to_ascii_lowercase());
            last_underscore = false;
        }
    }
    out.trim_matches('_').to_string()
}

fn link_src_to_src2(rust_dir: &Path) -> Result<()> {
    let src = rust_dir.join("src");
    let src1 = rust_dir.join("src.1");
    let src2 = rust_dir.join("src.2");

    if src.is_symlink() {
        std::fs::remove_file(&src).map_err(|e| anyhow!("remove {}: {}", src.display(), e))?;
    } else {
        let _ = std::fs::remove_dir_all(&src1);
        std::fs::rename(&src, &src1)
            .map_err(|e| anyhow!("rename {} to {}: {}", src.display(), src1.display(), e))?;
    }

    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(&src2, &src)
            .map_err(|e| anyhow!("symlink {} to {}: {}", src.display(), src2.display(), e))?;
    }
    #[cfg(not(unix))]
    {
        return Err(anyhow!(
            "symlink from rust/src to rust/src.2 is only supported on Unix systems"
        ));
    }
    Ok(())
}

fn middleware_stems(paths: &[PathBuf]) -> Vec<String> {
    use std::collections::HashMap;

    let mut counts: HashMap<String, usize> = HashMap::new();
    for path in paths {
        let stem = middleware_stem(path);
        *counts.entry(stem).or_insert(0) += 1;
    }

    paths
        .iter()
        .map(|path| {
            let stem = middleware_stem(path);
            if counts.get(&stem).copied().unwrap_or(0) <= 1 {
                stem
            } else {
                format!("{}_{}", stem, stable_short_path_hash(path))
            }
        })
        .collect()
}

fn middleware_stem(path: &Path) -> String {
    let file_name = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown");
    let no_suffix = file_name
        .strip_suffix(".cpp2rust")
        .unwrap_or(file_name)
        .to_string();
    Path::new(&no_suffix)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string()
}

/// Build a stable, short hash suffix from the full path.
///
/// We intentionally use a tiny in-tree FNV-1a style hash (offset basis
/// `1469598103934665603`, prime `1099511628211`) to avoid extra dependencies
/// while keeping deterministic output across runs.  Only the lower 32 bits are
/// kept so generated file names stay readable (`<stem>_<8hex>`).
fn stable_short_path_hash(path: &Path) -> String {
    let mut hash: u64 = 1469598103934665603;
    for b in path.to_string_lossy().as_bytes() {
        hash ^= *b as u64;
        hash = hash.wrapping_mul(1099511628211);
    }
    format!("{:08x}", (hash & 0xffff_ffff) as u32)
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

fn main() {
    let cli = Cli::parse();
    let result = match cli.command {
        Commands::Init(args) => run_init(args),
        Commands::Merge(args) => run_merge(args),
    };
    if let Err(e) = result {
        eprintln!("Error: {:#}", e);
        std::process::exit(1);
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{ClassIR, ExtractedDecls, FunctionIR};
    use clap::CommandFactory;

    #[test]
    fn cli_help_does_not_panic() {
        Cli::command().debug_assert();
    }

    #[test]
    fn init_requires_link() {
        let result = Cli::try_parse_from(["cpp2rust-demo", "init", "--", "make", "-j4"]);
        assert!(result.is_err());
    }

    #[test]
    fn init_requires_build_cmd() {
        let result = Cli::try_parse_from(["cpp2rust-demo", "init", "--link", "mylib"]);
        assert!(result.is_err());
    }

    #[test]
    fn init_parses_build_command_correctly() {
        let args = Cli::try_parse_from([
            "cpp2rust-demo",
            "init",
            "--link",
            "mylib",
            "--",
            "make",
            "-j4",
        ])
        .unwrap();
        let Commands::Init(init) = args.command else {
            panic!("expected Init");
        };
        assert_eq!(init.feature, "default");
        assert_eq!(init.link, "mylib");
        assert!(!init.no_link);
        assert_eq!(init.build_cmd, vec!["make", "-j4"]);
    }

    #[test]
    fn init_custom_feature() {
        let args = Cli::try_parse_from([
            "cpp2rust-demo",
            "init",
            "--feature",
            "myfeature",
            "--link",
            "mylib",
            "--",
            "cmake",
            "--build",
            "build",
        ])
        .unwrap();
        let Commands::Init(init) = args.command else {
            panic!("expected Init");
        };
        assert_eq!(init.feature, "myfeature");
        assert_eq!(init.build_cmd, vec!["cmake", "--build", "build"]);
    }

    #[test]
    fn init_accepts_hyphenated_build_args() {
        let args = Cli::try_parse_from([
            "cpp2rust-demo",
            "init",
            "--link",
            "mylib",
            "--",
            "clang",
            "-x",
            "c++",
            "-fsyntax-only",
            "header.hpp",
        ])
        .unwrap();
        let Commands::Init(init) = args.command else {
            panic!("expected Init");
        };
        assert_eq!(
            init.build_cmd,
            vec!["clang", "-x", "c++", "-fsyntax-only", "header.hpp"]
        );
    }

    #[test]
    fn init_accepts_no_link_aliases() {
        let args = Cli::try_parse_from([
            "cpp2rust-demo",
            "init",
            "--link",
            "rapidjson",
            "--no-link",
            "--",
            "make",
        ])
        .unwrap();
        let Commands::Init(init) = args.command else {
            panic!("expected Init");
        };
        assert!(init.no_link);

        let args = Cli::try_parse_from([
            "cpp2rust-demo",
            "init",
            "--link",
            "rapidjson",
            "--header-only",
            "--",
            "make",
        ])
        .unwrap();
        let Commands::Init(init) = args.command else {
            panic!("expected Init");
        };
        assert!(init.no_link);
    }

    #[test]
    fn middleware_stem_strips_cpp2rust_and_original_extension() {
        let p = PathBuf::from("/tmp/mylib.cpp.cpp2rust");
        assert_eq!(middleware_stem(&p), "mylib");
    }

    #[test]
    fn middleware_stems_hashes_on_collisions() {
        let paths = vec![
            PathBuf::from("/a/lib.cpp.cpp2rust"),
            PathBuf::from("/b/lib.cc.cpp2rust"),
        ];
        let stems = middleware_stems(&paths);
        assert_eq!(stems.len(), 2);
        assert!(stems[0].starts_with("lib_"));
        assert!(stems[1].starts_with("lib_"));
        assert_ne!(stems[0], stems[1]);
    }

    #[test]
    fn middleware_group_module_uses_relative_path_and_stem() {
        let cpp_dir = PathBuf::from("/tmp/.cpp2rust/default/cpp");
        let path = cpp_dir.join("src/foo/bar.cpp.cpp2rust");
        let group = middleware_group_module(&cpp_dir, &path);
        assert_eq!(group, "mod_src_foo_bar");
    }

    #[test]
    fn render_group_mod_rs_tracks_real_content_flags() {
        let src = render_group_mod_rs(true, false, false);
        assert!(src.contains("pub mod include;"));
        assert!(src.contains("pub mod types;"));
        assert!(src.contains("pub mod free;"));
        assert!(!src.contains("pub mod class;"));
        assert!(src.contains("pub use free::*;"));
        assert!(!src.contains("pub use class::*;"));
    }

    fn make_function(name: &str, is_static: bool) -> FunctionIR {
        FunctionIR {
            name: name.to_string(),
            rust_name: name.to_string(),
            return_type: "int".to_string(),
            rust_return_type: "i32".to_string(),
            params: vec![],
            qualified_name: name.to_string(),
            cpp_signature: format!("int {}()", name),
            is_const: false,
            is_static,
            is_virtual: false,
            is_pure: false,
            class_name: None,
        }
    }

    #[test]
    fn has_free_bindings_detects_free_functions() {
        let decls = ExtractedDecls {
            functions: vec![make_function("foo", false)],
            ..ExtractedDecls::default()
        };
        assert!(has_free_bindings(&decls));
    }

    #[test]
    fn has_free_bindings_detects_class_forward_decls_requirement() {
        let decls = ExtractedDecls {
            classes: vec![ClassIR {
                name: "Widget".to_string(),
                qualified_name: "Widget".to_string(),
                methods: vec![make_function("update", false)],
                ..ClassIR::default()
            }],
            ..ExtractedDecls::default()
        };
        assert!(has_free_bindings(&decls));
    }

    #[test]
    fn has_free_bindings_false_for_empty_decls() {
        let decls = ExtractedDecls::default();
        assert!(!has_free_bindings(&decls));
    }

    #[test]
    fn merge_default_feature() {
        let args = Cli::try_parse_from(["cpp2rust-demo", "merge"]).unwrap();
        let Commands::Merge(merge) = args.command else {
            panic!("expected Merge");
        };
        assert_eq!(merge.feature, "default");
    }

    #[test]
    fn merge_custom_feature() {
        let args =
            Cli::try_parse_from(["cpp2rust-demo", "merge", "--feature", "myfeature"]).unwrap();
        let Commands::Merge(merge) = args.command else {
            panic!("expected Merge");
        };
        assert_eq!(merge.feature, "myfeature");
    }
}
