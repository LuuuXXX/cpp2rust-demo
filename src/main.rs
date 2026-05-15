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

    /// Print `using` alias suggestions for skipped C++ templates.
    ///
    /// Reads the saved clang AST JSON files produced by a previous `init` run
    /// and emits a list of `using Alias = FullType<Args...>;` declarations.
    /// Copy the relevant ones into your C++ header and re-run `init` to unlock
    /// automatic FFI extraction for template specialisations.
    ///
    /// Example:
    ///   cpp2rust-demo suggest-aliases --feature default
    #[command(name = "suggest-aliases")]
    SuggestAliases(SuggestAliasesArgs),
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

    /// Dry-run mode: execute build capture and AST dump but do not write any
    /// files to `rust/src/`.  The interface report is printed to stdout.
    ///
    /// Useful for previewing what cpp2rust-demo would extract without
    /// modifying the project layout.
    #[arg(long = "dry-run")]
    dry_run: bool,

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

    /// Output directory: if specified, copy the merged Rust project here
    /// (src.1 and src.2 are excluded; src symlink is followed).
    #[arg(short = 'o', long, value_name = "DIR")]
    output: Option<PathBuf>,
}

#[derive(Args)]
struct SuggestAliasesArgs {
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
    let dry_run = args.dry_run;
    let build_cmd = &args.build_cmd;

    let cwd = std::env::current_dir().map_err(|e| anyhow!("current_dir: {}", e))?;
    let project_root = layout::find_project_root(&cwd);

    println!("cpp2rust-demo init");
    println!("  Project   : {}", project_root.display());
    println!("  Feature   : {}", feature);
    println!("  Link name : {}", link_name);
    if no_link {
        println!("  Link mode : header-only (no-link)");
    }
    if dry_run {
        println!("  Mode      : dry-run");
    }
    println!("  Command   : {}", build_cmd.join(" "));
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
        "[init] Captured {} middleware file(s).",
        captured_files.len()
    );

    // ----------------------------------------------------------------
    // Interactive middleware file selection
    // (auto-selects all when stdin is not a terminal, e.g. in CI/scripts)
    // ----------------------------------------------------------------
    let sel = InteractiveSelector;
    let selected_files = sel.select(&captured_files)?;
    println!(
        "[init] Selected {} file(s) for processing.",
        selected_files.len()
    );

    lo.save_selected_files(&selected_files)?;

    if selected_files.is_empty() {
        println!("No middleware files selected – skipping FFI generation.");
        return Ok(());
    }

    let files_to_process = selected_files;

    lo.save_meta(&files_to_process, link_name, no_link)?;

    // Create convenience symlinks in the capture directory so that
    // `hicc::cpp! { #include "entry.cpp" }` resolves correctly.
    // Each `entry.cpp.cpp2rust` file gets a sibling symlink `entry.cpp` →
    // `entry.cpp.cpp2rust` so the generated `#include "entry.cpp"` is found
    // when hicc-build uses the capture directory as an include path.
    // Unix-only; silently skipped on other platforms.
    for mw_file in &files_to_process {
        create_original_name_symlink(mw_file);
    }

    // In dry-run mode, skip creating the Rust project skeleton and all file writes.
    // We still process the AST and generate the report.
    let rust_src_dir = lo.rust_dir.join("src");
    if !dry_run {
        // Create the Rust project skeleton.
        std::fs::create_dir_all(&rust_src_dir)
            .map_err(|e| anyhow!("create {}: {}", rust_src_dir.display(), e))?;
    }

    // Compute deterministic stem names for middleware files.
    let stems: Vec<String> = middleware_stems(&files_to_process);

    // Write Cargo.toml, build.rs, lib.rs for the generated crate.
    let crate_name = format!("cpp2rust-{}-ffi", feature.replace('_', "-"));
    if !dry_run {
        let cargo_toml_path = lo.rust_dir.join("Cargo.toml");
        if !cargo_toml_path.exists() {
            std::fs::write(
                &cargo_toml_path,
                codegen::render_cargo_toml(&crate_name, link_name),
            )
            .map_err(|e| anyhow!("write Cargo.toml: {}", e))?;
        }

        // Prepare shared/common module scaffolding.
        write_common_modules(&rust_src_dir, "", "")?;
    }

    // Process each selected middleware file.
    let mut all_decls = ast::ExtractedDecls::default();
    let mut report_sections: Vec<String> = Vec::new();
    let mut build_rs_sources: Vec<String> = Vec::new();
    let mut lib_modules: Vec<String> = vec!["common".to_string()];
    let mut had_any_shims = false;
    let mut had_any_dynamic_casts = false;
    let mut had_any_placement_new = false;

    for (selected_file, stem) in files_to_process.iter().zip(stems.iter()) {
        let file_basename = selected_file
            .file_name()
            .and_then(|n| n.to_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| selected_file.display().to_string());
        println!("\n[{}]", file_basename);

        // Step 1: dump AST via clang.
        let ast_root = ast::dump_ast(selected_file, &extra_args, &args.clang)?;

        // Save the AST JSON for debugging (even in dry-run so the user can inspect).
        let ast_json_path = lo.ast_dir.join(format!("{}.ast.json", stem));
        let ast_json =
            serde_json::to_string(&ast_root).map_err(|e| anyhow!("serialize AST: {}", e))?;
        std::fs::write(&ast_json_path, &ast_json).map_err(|e| anyhow!("write AST JSON: {}", e))?;

        // Step 2: extract declarations.
        let file_paths: Vec<&Path> = vec![selected_file.as_path()];
        let decls = ast::extract_declarations(&ast_root, &file_paths);

        println!(
            "  {} function(s), {} class(es), {} global(s)",
            decls.functions.len(),
            decls.classes.len(),
            decls.globals.len()
        );

        if decls.functions.is_empty() && decls.classes.is_empty() {
            println!("  ⚠ No declarations found – check include paths or C++ source.");
        }

        let has_shims = !decls.operator_shims.is_empty()
            || decls.skipped.iter().any(|s| s.suggested_shim.is_some());

        // Dynamic cast skeletons are generated whenever any class has public bases.
        let dynamic_casts_src = codegen::render_dynamic_casts_module(&decls, link_name);
        let has_dynamic_casts = !dynamic_casts_src.is_empty();

        // Placement-new skeletons (P4): generated whenever a concrete class has ctors.
        let placement_new_src = codegen::render_placement_new_module(&decls, link_name);
        let has_placement_new = !placement_new_src.is_empty();

        if !dry_run {
            // Render operator shim Rust stubs (active bindings for operator_shims.hpp).
            let shims_rs = if !decls.operator_shims.is_empty() {
                Some(codegen::render_operator_shims_rs(
                    &decls.operator_shims,
                    link_name,
                ))
            } else {
                None
            };

            // Write a single flat `<stem>.rs` file combining all FFI content.
            let flat_src = codegen::render_flat_module(
                &decls,
                link_name,
                &selected_file.display().to_string(),
                shims_rs.as_deref(),
                has_dynamic_casts.then_some(dynamic_casts_src.as_str()),
                has_placement_new.then_some(placement_new_src.as_str()),
            );
            let flat_file_path = rust_src_dir.join(format!("{}.rs", stem));
            std::fs::write(&flat_file_path, &flat_src)
                .map_err(|e| anyhow!("write {}: {}", flat_file_path.display(), e))?;

            // Operator shims C++ header still goes to meta/ for user reference.
            if has_shims {
                let middleware_basename = selected_file
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("middleware.hpp");
                let shims_hpp = codegen::render_operator_shims_hpp(
                    &decls.operator_shims,
                    &decls.skipped,
                    middleware_basename,
                );
                if !shims_hpp.is_empty() {
                    let shims_hpp_path = lo.meta_dir.join("operator_shims.hpp");
                    std::fs::write(&shims_hpp_path, &shims_hpp)
                        .map_err(|e| anyhow!("write operator_shims.hpp: {}", e))?;
                    println!("  → operator_shims.hpp");
                }
            }

            if has_dynamic_casts {
                println!("  → @dynamic_cast starters (in {}.rs)", stem);
            }
            if has_placement_new {
                println!("  → @placement_new starters (in {}.rs)", stem);
            }

            let group_meta = GroupMeta {
                group: stem.to_string(),
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
            let meta_path = rust_src_dir.join(format!("{}.meta.json", stem));
            std::fs::write(
                &meta_path,
                serde_json::to_string_pretty(&group_meta)
                    .map_err(|e| anyhow!("serialize meta: {}", e))?,
            )
            .map_err(|e| anyhow!("write {}: {}", meta_path.display(), e))?;

            build_rs_sources.push(format!("src/{}.rs", stem));
            lib_modules.push(stem.to_string());
            if has_shims {
                had_any_shims = true;
            }
            if has_dynamic_casts {
                had_any_dynamic_casts = true;
            }
            if has_placement_new {
                had_any_placement_new = true;
            }
        } else {
            // dry-run: no file writes.
        }

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
        all_decls.enums.extend(decls.enums);
        all_decls.aliases.extend(decls.aliases);
        all_decls.skipped.extend(decls.skipped);
        all_decls.operator_shims.extend(decls.operator_shims);
    }

    // Write interface report.
    let report = report_sections.join("\n---\n\n");
    if dry_run {
        // In dry-run mode, print the interface report to stdout instead of saving files.
        println!("\n── DRY-RUN: Interface Report ──────────────────────────\n");
        println!("{}", report);
        println!("\n✓ init --dry-run done (no files written to rust/src/).");
    } else {
        let report_path = lo.meta_dir.join("init-interface-report.md");
        std::fs::write(&report_path, &report).map_err(|e| anyhow!("write report: {}", e))?;

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
        }

        // Print success summary.
        let group_count = lib_modules.len().saturating_sub(1); // exclude "common"
        let total_fns = all_decls.functions.len();
        let total_classes = all_decls.classes.len();
        let total_globals = all_decls.globals.len();
        let total_methods: usize = all_decls.classes.iter().map(|c| c.methods.len()).sum();
        let total_skipped = all_decls.skipped.len();

        println!("\n──────────────────────────────────────────────────────");
        println!("  ✓  init completed  (feature: {})", feature);
        println!("──────────────────────────────────────────────────────");
        println!("\nProcessed {} group(s):", group_count);
        for m in lib_modules.iter().filter(|m| m.as_str() != "common") {
            println!("  • {}", m);
        }
        println!("\nExtracted:");
        if total_fns > 0 {
            println!("  {} free function(s)  →  import_lib! bindings", total_fns);
        }
        if total_classes > 0 {
            println!(
                "  {} class(es) / {} method(s)  →  import_class! bindings",
                total_classes, total_methods
            );
        }
        if total_globals > 0 {
            println!(
                "  {} global variable(s)  →  #[cpp(data=...)] bindings",
                total_globals
            );
        }
        if had_any_shims {
            println!(
                "  operator shims  →  meta/operator_shims.hpp (Rust bindings in flat .rs files)"
            );
        }
        if had_any_dynamic_casts {
            println!("  dynamic cast starters  →  embedded in flat .rs files");
        }
        if had_any_placement_new {
            println!("  placement-new starters  →  embedded in flat .rs files");
        }
        if total_skipped > 0 {
            println!(
                "  {} declaration(s) skipped  (see interface report)",
                total_skipped
            );
        }
        println!("\nOutput:");
        println!("  .cpp2rust/{}/", feature);
        println!("    ├── cpp/    captured middleware (*.cpp2rust + *.cpp symlinks)");
        println!("    ├── ast/    clang AST JSON per file");
        println!("    ├── meta/   init-interface-report.md  build metadata");
        println!("    └── rust/   Rust project (Cargo.toml  build.rs  src/)");
        println!("          src/");
        println!("            ├── common/  aggregate type helpers + include metadata");
        for m in lib_modules.iter().filter(|m| m.as_str() != "common") {
            println!("            ├── {}.rs   ← flat FFI file (1:1 per .cpp)", m);
        }
        println!("            └── lib.rs");
        println!("\nNext steps:");
        println!("  1. Review the interface report:");
        println!("       .cpp2rust/{}/meta/init-interface-report.md", feature);
        println!("  2. Run `cpp2rust-demo merge` to produce a single merged_ffi.rs");
        println!("  3. Or use `cpp2rust-demo merge --output <dir>` to export");
    }

    Ok(())
}

fn run_merge(args: MergeArgs) -> Result<()> {
    let feature = &args.feature;

    let cwd = std::env::current_dir().map_err(|e| anyhow!("current_dir: {}", e))?;
    let project_root = layout::find_project_root(&cwd);

    println!("cpp2rust-demo merge");
    println!("  Project : {}", project_root.display());
    println!("  Feature : {}", feature);
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

    link_src_to_src2(&lo.rust_dir)?;

    // Write a merge report.
    let report_path = lo.meta_dir.join("merge-report.md");
    let report = format!(
        "# Merge Report\n\nFeature: `{feature}`\nLink name: `{link_name}`\n\nMerged groups: {}\n\nMerged output: `{}`\n",
        merged.group_modules.len(),
        merged.merged_path.display()
    );
    std::fs::write(&report_path, &report).map_err(|e| anyhow!("write merge report: {}", e))?;

    println!("\n──────────────────────────────────────────────────────");
    println!("  ✓  merge completed  (feature: {})", feature);
    println!("──────────────────────────────────────────────────────");
    println!("\nMerged {} group(s):", merged.group_modules.len());
    for g in &merged.group_modules {
        println!("  • {}", g);
    }
    println!("\nOutput layout:");
    println!("  rust/src     → src.2   (build.rs uses this active view)");
    println!("  rust/src.1/  per-file init modules (backup, 1:1 per .cpp)");
    println!("  rust/src.2/");
    println!("    ├── lib.rs");
    for g in &merged.group_modules {
        println!("    ├── {}.rs", g);
    }
    println!("    └── merged_ffi.rs   ← main FFI entry point");
    println!("  meta/merge-report.md");
    println!("\nNext steps:");
    println!("  1. Use merged_ffi.rs as your FFI entry point:");
    println!("       .cpp2rust/{}/rust/src/merged_ffi.rs", feature);
    println!("  2. Copy the Rust project to your workspace:");
    println!(
        "       cp -r .cpp2rust/{}/rust/ /path/to/my-project/",
        feature
    );
    println!("  3. Adjust build.rs to point to your C++ library");

    if let Some(output_dir) = &args.output {
        copy_merge_output(&lo.rust_dir, output_dir)?;

        // Also copy relevant meta files (operator_shims.hpp, init-interface-report.md)
        // into <output>/meta/ so users have everything they need in one place.
        copy_meta_files(&lo.meta_dir, output_dir)?;
        println!("\n  Exported to: {}", output_dir.display());
    }

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

/// Create a symlink `<basename_without_cpp2rust>` → `<basename>` in the same
/// directory as `mw_file`, so that `hicc::cpp! { #include "entry.cpp" }` resolves
/// correctly when the capture directory is on the include path.
///
/// On non-Unix platforms this is a no-op (silently skipped).
fn create_original_name_symlink(mw_file: &Path) {
    #[cfg(unix)]
    {
        let file_name = match mw_file.file_name().and_then(|n| n.to_str()) {
            Some(n) => n,
            None => return,
        };
        let original_name = match file_name.strip_suffix(".cpp2rust") {
            Some(n) => n,
            None => return,
        };
        let parent = match mw_file.parent() {
            Some(p) => p,
            None => return,
        };
        let symlink_path = parent.join(original_name);
        if !symlink_path.exists() {
            // Target is a relative name so the symlink stays valid if the
            // capture directory is moved.
            if let Err(e) = std::os::unix::fs::symlink(file_name, &symlink_path) {
                eprintln!(
                    "  Warning: could not create symlink {} → {}: {}",
                    symlink_path.display(),
                    file_name,
                    e
                );
            }
        }
    }
}

/// Copy relevant meta files (`operator_shims.hpp`, `init-interface-report.md`)
/// from `meta_dir` into `<output_dir>/meta/` so users get a self-contained
/// export that includes the C++ shim header they need to compile against.
fn copy_meta_files(meta_dir: &Path, output_dir: &Path) -> Result<()> {
    const RELEVANT: &[&str] = &["operator_shims.hpp", "init-interface-report.md"];

    let out_meta = output_dir.join("meta");
    let mut copied_any = false;

    for name in RELEVANT {
        let src = meta_dir.join(name);
        if !src.exists() {
            continue;
        }
        if !copied_any {
            std::fs::create_dir_all(&out_meta)
                .map_err(|e| anyhow!("create {}: {}", out_meta.display(), e))?;
            copied_any = true;
        }
        let dst = out_meta.join(name);
        std::fs::copy(&src, &dst)
            .map_err(|e| anyhow!("copy {} to {}: {}", src.display(), dst.display(), e))?;
    }

    Ok(())
}

/// Copy the merged Rust project from `rust_dir` to `output_dir`.
///
/// Entries named `src.1` or `src.2` are skipped.
/// The `src` symlink (which points to `src.2`) is followed so that a real
/// `src/` directory is written into the output.
fn copy_merge_output(rust_dir: &Path, output_dir: &Path) -> Result<()> {
    let rust_dir_canon = rust_dir
        .canonicalize()
        .map_err(|e| anyhow!("canonicalize {}: {}", rust_dir.display(), e))?;
    let output_abs = if output_dir.is_absolute() {
        output_dir.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(|e| anyhow!("current_dir: {}", e))?
            .join(output_dir)
    };

    let mut created_output_root: Option<PathBuf> = None;
    if output_abs.exists() {
        if !output_abs.is_dir() {
            return Err(anyhow!(
                "output path {} exists and is not a directory",
                output_abs.display()
            ));
        }
        let mut existing_entries = std::fs::read_dir(&output_abs)
            .map_err(|e| anyhow!("read dir {}: {}", output_abs.display(), e))?;
        if existing_entries
            .next()
            .transpose()
            .map_err(|e| anyhow!("read entry in {}: {}", output_abs.display(), e))?
            .is_some()
        {
            return Err(anyhow!(
                "output dir {} already exists and is not empty",
                output_abs.display()
            ));
        }
    } else {
        created_output_root = Some(first_missing_ancestor(&output_abs));
        std::fs::create_dir_all(&output_abs)
            .map_err(|e| anyhow!("create output dir {}: {}", output_abs.display(), e))?;
    }

    let output_canon = output_abs
        .canonicalize()
        .map_err(|e| anyhow!("canonicalize {}: {}", output_abs.display(), e))?;

    if output_canon.starts_with(&rust_dir_canon) {
        if let Some(created_root) = &created_output_root {
            cleanup_created_empty_dirs(&output_abs, created_root);
        }
        return Err(anyhow!(
            "output dir {} must not be inside rust dir {}",
            output_canon.display(),
            rust_dir_canon.display()
        ));
    }

    let skip = ["src.1", "src.2"];

    for entry in std::fs::read_dir(rust_dir)
        .map_err(|e| anyhow!("read dir {}: {}", rust_dir.display(), e))?
    {
        let entry = entry.map_err(|e| anyhow!("read entry: {}", e))?;
        let name = entry.file_name();
        let name_str = name.to_string_lossy();

        if skip.contains(&name_str.as_ref()) {
            continue;
        }

        let src_path = entry.path();
        let dst_path = output_canon.join(&name);

        // Follow symlinks (e.g. `src` → `src.2`)
        let actual_path = if src_path.is_symlink() {
            src_path
                .canonicalize()
                .map_err(|e| anyhow!("canonicalize {}: {}", src_path.display(), e))?
        } else {
            src_path.clone()
        };

        if actual_path.is_dir() {
            copy_dir_recursive(&actual_path, &dst_path)?;
        } else {
            std::fs::copy(&actual_path, &dst_path).map_err(|e| {
                anyhow!(
                    "copy {} to {}: {}",
                    actual_path.display(),
                    dst_path.display(),
                    e
                )
            })?;
        }
    }

    Ok(())
}

fn first_missing_ancestor(path: &Path) -> PathBuf {
    let mut first_missing = path.to_path_buf();
    let mut current = path;
    loop {
        if current.exists() {
            break;
        }
        first_missing = current.to_path_buf();
        match current.parent() {
            Some(parent) => current = parent,
            None => break,
        }
    }
    first_missing
}

fn cleanup_created_empty_dirs(path: &Path, created_root: &Path) {
    let mut current = Some(path);
    while let Some(dir) = current {
        match std::fs::remove_dir(dir) {
            Ok(()) => {}
            Err(_) => break,
        }
        if dir == created_root {
            break;
        }
        current = dir.parent();
    }
}

/// Recursively copy a directory tree from `src` to `dst`.
fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<()> {
    std::fs::create_dir_all(dst).map_err(|e| anyhow!("create dir {}: {}", dst.display(), e))?;
    for entry in std::fs::read_dir(src).map_err(|e| anyhow!("read dir {}: {}", src.display(), e))? {
        let entry = entry.map_err(|e| anyhow!("read entry: {}", e))?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            std::fs::copy(&src_path, &dst_path).map_err(|e| {
                anyhow!(
                    "copy {} to {}: {}",
                    src_path.display(),
                    dst_path.display(),
                    e
                )
            })?;
        }
    }
    Ok(())
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

/// Implementation of the `suggest-aliases` subcommand.
///
/// Reads the clang AST JSON files saved by a previous `init` run and scans
/// for `ClassTemplateDecl` / `ClassTemplateSpecializationDecl` nodes that do
/// not yet have a `typedef`/`using` alias in the user's header.  For each
/// such template, prints ready-to-copy `using` declarations so the user can
/// paste them into the header and re-run `init` to unlock automatic extraction.
fn run_suggest_aliases(args: SuggestAliasesArgs) -> Result<()> {
    let cwd = std::env::current_dir().map_err(|e| anyhow!("current_dir: {}", e))?;
    let project_root = layout::find_project_root(&cwd);
    let lo = layout::FeatureLayout::new(project_root.clone(), &args.feature);

    if !lo.ast_dir.exists() {
        return Err(anyhow!(
            "AST directory not found at {}; run 'init' first",
            lo.ast_dir.display()
        ));
    }

    println!("cpp2rust-demo suggest-aliases");
    println!("  Project : {}", project_root.display());
    println!("  Feature : {}", args.feature);
    println!();

    // Collect AST JSON files.
    let ast_files: Vec<PathBuf> = std::fs::read_dir(&lo.ast_dir)
        .map_err(|e| anyhow!("read dir {}: {}", lo.ast_dir.display(), e))?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("json") {
                Some(path)
            } else {
                None
            }
        })
        .collect();

    if ast_files.is_empty() {
        println!("No AST JSON files found. Run 'init' first.");
        return Ok(());
    }

    let mut total_suggestions = 0usize;

    for ast_path in &ast_files {
        let json_str = std::fs::read_to_string(ast_path)
            .map_err(|e| anyhow!("read {}: {}", ast_path.display(), e))?;
        let ast_root: ast::AstNode = serde_json::from_str(&json_str)
            .map_err(|e| anyhow!("parse AST JSON {}: {}", ast_path.display(), e))?;

        // Build the alias registry to know which templates already have aliases.
        let alias_registry = ast::AliasRegistry::collect_from_ast(&ast_root);

        // Collect suggestions.
        let suggestions = collect_alias_suggestions_from_ast(&ast_root, &alias_registry, &[]);

        if !suggestions.is_empty() {
            println!("## From `{}`\n", ast_path.display());
            println!("Add these `using` declarations to your C++ header, then re-run `cpp2rust-demo init`:\n");
            println!("```cpp");
            for (template_name, specializations) in &suggestions {
                println!("// Template: {}", template_name);
                for (i, spec_type) in specializations.iter().enumerate() {
                    let bare = spec_type
                        .split('<')
                        .next()
                        .unwrap_or(spec_type)
                        .rsplit("::")
                        .next()
                        .unwrap_or(spec_type)
                        .trim();
                    // Use a consistent numeric suffix: MyFoo_1, MyFoo_2, ...
                    let alias_name = format!("My{}_{}", bare, i + 1);
                    println!("using {} = {};", alias_name, spec_type);
                }
            }
            println!("```\n");
            total_suggestions += suggestions.len();
        }
    }

    if total_suggestions == 0 {
        println!("No unaliased template specialisations found.");
        println!("Either all templates already have aliases, or no concrete specialisations");
        println!("were visible in the captured translation units.");
        println!("Tip: add explicit template instantiations (e.g. `template class Foo<int>;`)");
        println!("to make specialisations visible in the AST, then re-run `init`.");
    } else {
        println!("Found {} template(s) without aliases.", total_suggestions);
        println!("After adding the suggested aliases, re-run:");
        println!("  cpp2rust-demo init --link <libname> -- <build-command>");
    }

    Ok(())
}

/// Recursively walk an AST and collect (template_name, [specialization_types])
/// pairs for templates that do not yet have a `typedef`/`using` alias.
fn collect_alias_suggestions_from_ast(
    node: &ast::AstNode,
    alias_registry: &ast::AliasRegistry,
    namespace: &[String],
) -> Vec<(String, Vec<String>)> {
    let mut results: Vec<(String, Vec<String>)> = Vec::new();

    match node.kind.as_str() {
        "NamespaceDecl" => {
            if let Some(ref ns_name) = node.name {
                let mut ns = namespace.to_vec();
                ns.push(ns_name.clone());
                for child in node.inner.iter().flatten() {
                    results.extend(collect_alias_suggestions_from_ast(
                        child,
                        alias_registry,
                        &ns,
                    ));
                }
            }
        }

        "ClassTemplateDecl" => {
            if let Some(template_name) = node.name.as_deref() {
                // Skip if the template already has an alias registered.
                if !alias_registry.has_template_alias(template_name) {
                    let mut spec_types: Vec<String> = Vec::new();
                    for child in node.inner.iter().flatten() {
                        if child.kind == "ClassTemplateSpecializationDecl"
                            && child.complete_definition.unwrap_or(false)
                        {
                            if let Some(ref ti) = child.type_info {
                                let qt = ti.qual_type.trim();
                                let qt = qt
                                    .strip_prefix("struct ")
                                    .or_else(|| qt.strip_prefix("class "))
                                    .unwrap_or(qt)
                                    .trim()
                                    .to_string();
                                if !qt.is_empty() && !spec_types.contains(&qt) {
                                    spec_types.push(qt);
                                }
                            }
                        }
                    }
                    if !spec_types.is_empty() {
                        let qualified = if namespace.is_empty() {
                            template_name.to_string()
                        } else {
                            format!("{}::{}", namespace.join("::"), template_name)
                        };
                        results.push((qualified, spec_types));
                    }
                }
            }
        }

        _ => {
            for child in node.inner.iter().flatten() {
                results.extend(collect_alias_suggestions_from_ast(
                    child,
                    alias_registry,
                    namespace,
                ));
            }
        }
    }

    results
}

fn main() {
    let cli = Cli::parse();
    let result = match cli.command {
        Commands::Init(args) => run_init(args),
        Commands::Merge(args) => run_merge(args),
        Commands::SuggestAliases(args) => run_suggest_aliases(args),
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
    use tempfile::TempDir;

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
            is_variadic: false,
            is_rvalue: false,
        }
    }

    #[test]
    fn flat_module_contains_cpp_include_and_import_lib() {
        let decls = ExtractedDecls {
            functions: vec![make_function("foo", false)],
            ..ExtractedDecls::default()
        };
        let src = crate::codegen::render_flat_module(
            &decls,
            "mylib",
            "/tmp/mylib.cpp.cpp2rust",
            None,
            None,
            None,
        );
        assert!(
            src.contains("hicc::cpp!"),
            "flat file must have hicc::cpp! block"
        );
        assert!(
            src.contains("import_lib!"),
            "flat file must have import_lib! block"
        );
    }

    #[test]
    fn flat_module_appends_shim_ops_when_provided() {
        let decls = ExtractedDecls::default();
        let shim_content = "hicc::import_lib! { #![link_name = \"mylib\"] fn shim_foo(); }";
        let src = crate::codegen::render_flat_module(
            &decls,
            "mylib",
            "/tmp/mylib.cpp.cpp2rust",
            Some(shim_content),
            None,
            None,
        );
        assert!(
            src.contains("shim_foo"),
            "shim ops should be appended to flat file"
        );
    }

    #[test]
    fn flat_module_omits_optional_sections_when_none() {
        let decls = ExtractedDecls::default();
        let src = crate::codegen::render_flat_module(
            &decls,
            "mylib",
            "/tmp/mylib.cpp.cpp2rust",
            None,
            None,
            None,
        );
        assert!(
            !src.contains("shim"),
            "no shim section when shim_ops is None"
        );
        assert!(
            !src.contains("@dynamic_cast"),
            "no cast section when dynamic_casts is None"
        );
        assert!(
            !src.contains("@placement_new"),
            "no pn section when placement_new is None"
        );
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

    #[test]
    fn merge_output_dir_args() {
        let args = Cli::try_parse_from(["cpp2rust-demo", "merge", "-o", "out-dir"]).unwrap();
        let Commands::Merge(merge) = args.command else {
            panic!("expected Merge");
        };
        assert_eq!(merge.output, Some(PathBuf::from("out-dir")));

        let args =
            Cli::try_parse_from(["cpp2rust-demo", "merge", "--output", "other-dir"]).unwrap();
        let Commands::Merge(merge) = args.command else {
            panic!("expected Merge");
        };
        assert_eq!(merge.output, Some(PathBuf::from("other-dir")));
    }

    #[cfg(unix)]
    #[test]
    fn copy_merge_output_skips_src1_src2_and_follows_src_symlink() {
        let tmp = TempDir::new().unwrap();
        let rust_dir = tmp.path().join("rust");
        let output_dir = tmp.path().join("out");

        std::fs::create_dir_all(rust_dir.join("src.1")).unwrap();
        std::fs::create_dir_all(rust_dir.join("src.2")).unwrap();
        std::fs::write(rust_dir.join("src.1").join("old.rs"), "// old").unwrap();
        std::fs::write(rust_dir.join("src.2").join("lib.rs"), "pub fn merged() {}").unwrap();
        std::fs::write(rust_dir.join("build.rs"), "// build").unwrap();

        std::os::unix::fs::symlink(rust_dir.join("src.2"), rust_dir.join("src")).unwrap();

        copy_merge_output(&rust_dir, &output_dir).unwrap();

        assert!(output_dir.join("build.rs").exists());
        assert!(output_dir.join("src").join("lib.rs").exists());
        assert!(!output_dir.join("src.1").exists());
        assert!(!output_dir.join("src.2").exists());
        assert!(
            !std::fs::symlink_metadata(output_dir.join("src"))
                .unwrap()
                .file_type()
                .is_symlink(),
            "copied src should be a real directory, not a symlink"
        );
    }

    #[test]
    fn copy_merge_output_rejects_output_inside_rust_dir() {
        let tmp = TempDir::new().unwrap();
        let rust_dir = tmp.path().join("rust");
        let output_dir = rust_dir.join("out");
        std::fs::create_dir_all(&rust_dir).unwrap();

        let err = copy_merge_output(&rust_dir, &output_dir).unwrap_err();
        assert!(
            err.to_string().contains("must not be inside rust dir"),
            "unexpected error: {err}"
        );
    }

    #[cfg(unix)]
    #[test]
    fn copy_merge_output_rejects_output_inside_rust_dir_via_symlink() {
        let tmp = TempDir::new().unwrap();
        let rust_dir = tmp.path().join("rust");
        std::fs::create_dir_all(&rust_dir).unwrap();
        let link_dir = tmp.path().join("link-to-rust");
        std::os::unix::fs::symlink(&rust_dir, &link_dir).unwrap();
        let output_dir = link_dir.join("out");

        let err = copy_merge_output(&rust_dir, &output_dir).unwrap_err();
        assert!(
            err.to_string().contains("must not be inside rust dir"),
            "unexpected error: {err}"
        );
        assert!(
            !rust_dir.join("out").exists(),
            "created output dir under rust should be cleaned up on rejection"
        );
    }

    #[cfg(unix)]
    #[test]
    fn copy_merge_output_rejects_existing_symlinked_output_without_deleting_it() {
        let tmp = TempDir::new().unwrap();
        let rust_dir = tmp.path().join("rust");
        std::fs::create_dir_all(rust_dir.join("out")).unwrap();
        let link_dir = tmp.path().join("link-to-rust");
        std::os::unix::fs::symlink(&rust_dir, &link_dir).unwrap();
        let output_dir = link_dir.join("out");

        let err = copy_merge_output(&rust_dir, &output_dir).unwrap_err();
        assert!(
            err.to_string().contains("must not be inside rust dir"),
            "unexpected error: {err}"
        );
        assert!(
            rust_dir.join("out").exists(),
            "existing output dir should not be deleted on rejection"
        );
    }

    #[cfg(unix)]
    #[test]
    fn copy_merge_output_rejects_nested_output_inside_rust_dir_and_cleans_parents() {
        let tmp = TempDir::new().unwrap();
        let rust_dir = tmp.path().join("rust");
        std::fs::create_dir_all(&rust_dir).unwrap();
        let link_dir = tmp.path().join("link-to-rust");
        std::os::unix::fs::symlink(&rust_dir, &link_dir).unwrap();
        let output_dir = link_dir.join("out").join("nested");

        let err = copy_merge_output(&rust_dir, &output_dir).unwrap_err();
        assert!(
            err.to_string().contains("must not be inside rust dir"),
            "unexpected error: {err}"
        );
        assert!(
            !rust_dir.join("out").exists(),
            "newly created parent dirs under rust should be cleaned up on rejection"
        );
    }

    #[test]
    fn copy_merge_output_rejects_non_empty_output_dir() {
        let tmp = TempDir::new().unwrap();
        let rust_dir = tmp.path().join("rust");
        let src_dir = rust_dir.join("src");
        let output_dir = tmp.path().join("out");
        std::fs::create_dir_all(&src_dir).unwrap();
        std::fs::write(src_dir.join("lib.rs"), "pub fn merged() {}").unwrap();
        std::fs::create_dir_all(&output_dir).unwrap();
        std::fs::write(output_dir.join("stale.txt"), "stale").unwrap();

        let err = copy_merge_output(&rust_dir, &output_dir).unwrap_err();
        assert!(
            err.to_string().contains("already exists and is not empty"),
            "unexpected error: {err}"
        );
    }
}
