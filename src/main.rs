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

    /// Merge per-header FFI files into a single consolidated file.
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
    let build_cmd = &args.build_cmd;

    let cwd = std::env::current_dir().map_err(|e| anyhow!("current_dir: {}", e))?;
    let project_root = layout::find_project_root(&cwd);

    println!("=== cpp2rust-demo init ===");
    println!("Project root : {}", project_root.display());
    println!("Feature      : {}", feature);
    println!("Link name    : {}", link_name);
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
                "preload hook did not capture any *2rust middleware files from build command; ",
                "ensure the build command really compiles C/C++ sources under the project root"
            )
        ));
    }
    println!(
        "Captured {} middleware file(s) via LD_PRELOAD hook.",
        captured_files.len()
    );

    // ----------------------------------------------------------------
    // Interactive header selection
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

    lo.save_meta(&files_to_process, link_name)?;

    // Create the Rust project skeleton.
    let rust_src_dir = lo.rust_dir.join("src");
    std::fs::create_dir_all(&rust_src_dir)
        .map_err(|e| anyhow!("create {}: {}", rust_src_dir.display(), e))?;

    // Compute deterministic stem names for middleware files.
    let stems: Vec<String> = middleware_stems(&files_to_process);

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

    // build.rs: list all per-file ffi files + include dirs for middleware files.
    let build_rs_path = lo.rust_dir.join("build.rs");
    {
        let src_files: Vec<String> = stems.iter().map(|s| format!("src/ffi_{}.rs", s)).collect();
        let src_refs: Vec<&str> = src_files.iter().map(|s| s.as_str()).collect();

        // Collect unique parent directories so hicc-build can
        // find the #included middleware files when compiling the adapter code.
        let include_dirs = header_include_dirs(&files_to_process);
        let inc_refs: Vec<&str> = include_dirs.iter().map(|s| s.as_str()).collect();

        std::fs::write(
            &build_rs_path,
            codegen::render_build_rs(link_name, &src_refs, &inc_refs),
        )
        .map_err(|e| anyhow!("write build.rs: {}", e))?;
        println!("Created {}", build_rs_path.display());
    }

    // lib.rs: re-export per-header ffi modules.
    let lib_rs_path = rust_src_dir.join("lib.rs");
    {
        let mod_names: Vec<String> = stems.iter().map(|s| format!("ffi_{}", s)).collect();
        let mod_refs: Vec<&str> = mod_names.iter().map(|s| s.as_str()).collect();
        std::fs::write(&lib_rs_path, codegen::render_lib_rs(&mod_refs))
            .map_err(|e| anyhow!("write lib.rs: {}", e))?;
        println!("Created {}", lib_rs_path.display());
    }

    // Process each selected middleware file.
    let mut all_decls = ast::ExtractedDecls::default();
    let mut report_sections: Vec<String> = Vec::new();

    for (selected_file, stem) in files_to_process.iter().zip(stems.iter()) {
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
            println!("  Warning: no declarations found from this header.");
        }

        // Step 3: generate FFI source.
        let ffi_src = codegen::render_ffi(&decls, link_name, &selected_file.display().to_string());
        let ffi_rs_path = rust_src_dir.join(format!("ffi_{}.rs", stem));
        std::fs::write(&ffi_rs_path, &ffi_src)
            .map_err(|e| anyhow!("write {}: {}", ffi_rs_path.display(), e))?;
        println!("  FFI source → {}", ffi_rs_path.display());

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
    }

    // Write interface report.
    let report_path = lo.meta_dir.join("init-interface-report.md");
    let report = report_sections.join("\n---\n\n");
    std::fs::write(&report_path, &report).map_err(|e| anyhow!("write report: {}", e))?;
    println!("\nInterface report → {}", report_path.display());

    println!("\n✓ cpp2rust-demo init completed successfully!");
    println!("\nOutput structure:");
    println!("  .cpp2rust/{}/", feature);
    println!("    ├── cpp/        (captured preprocessed middleware: *2rust)");
    println!("    ├── ast/        (clang AST JSON per selected file)");
    println!("    ├── meta/       (build_cmd.txt, selected_files.json, headers.json, init-interface-report.md)");
    println!("    └── rust/       (generated Rust project)");
    println!("        ├── Cargo.toml");
    println!("        ├── build.rs");
    println!("        └── src/");
    println!("            ├── lib.rs");
    println!("            └── ffi_<header>.rs  (one per input header)");
    println!();
    println!("Next steps:");
    println!("  1. Review .cpp2rust/{}/rust/src/ffi_*.rs", feature);
    println!("  2. Run `cpp2rust-demo merge` to consolidate into a single file");
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

    let (link_name, stored_files) = lo.load_meta()?;

    let rust_src_dir = lo.rust_dir.join("src");
    if !rust_src_dir.exists() {
        return Err(anyhow!(
            "rust/src not found at {}; run 'init' first",
            rust_src_dir.display()
        ));
    }

    let merged_path = merge::merge_ffi_files(&rust_src_dir, &link_name)?;

    // Recompute unique include dirs from stored selected files.
    let include_dirs = header_include_dirs(&stored_files);
    let inc_refs: Vec<&str> = include_dirs.iter().map(|s| s.as_str()).collect();

    // Update build.rs to reference merged_ffi.rs.
    let build_rs_path = lo.rust_dir.join("build.rs");
    std::fs::write(
        &build_rs_path,
        codegen::render_build_rs(&link_name, &["src/merged_ffi.rs"], &inc_refs),
    )
    .map_err(|e| anyhow!("update build.rs: {}", e))?;
    println!("  Updated {}", build_rs_path.display());

    // Update lib.rs to include merged_ffi module.
    let lib_rs_path = lo.rust_dir.join("src").join("lib.rs");
    std::fs::write(&lib_rs_path, codegen::render_lib_rs(&["merged_ffi"]))
        .map_err(|e| anyhow!("update lib.rs: {}", e))?;
    println!("  Updated {}", lib_rs_path.display());

    // Write a merge report.
    let report_path = lo.meta_dir.join("merge-report.md");
    let report = format!(
        "# Merge Report\n\nFeature: `{feature}`\nLink name: `{link_name}`\n\nMerged output: `{}`\n",
        merged_path.display()
    );
    std::fs::write(&report_path, &report).map_err(|e| anyhow!("write merge report: {}", e))?;

    println!("\n✓ cpp2rust-demo merge completed successfully!");
    println!("\nMerged output:");
    println!("  {}", merged_path.display());
    println!("\nThe merged file combines all import_class! and import_lib! blocks");
    println!("from every ffi_*.rs file into a single consolidated ffi.");
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

/// Collect unique parent directories from a list of header paths, sorted for
/// deterministic output.  Used to populate `build.include(...)` calls in the
/// generated `build.rs` so hicc-build can find the `#include`d headers.
fn header_include_dirs(headers: &[PathBuf]) -> Vec<String> {
    let mut dirs: Vec<String> = headers
        .iter()
        .filter_map(|h| h.parent().map(|p| p.display().to_string()))
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    dirs.sort();
    dirs
}

fn middleware_stems(paths: &[PathBuf]) -> Vec<String> {
    use std::collections::HashMap;

    let mut counts: HashMap<String, usize> = HashMap::new();
    for path in paths {
        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();
        *counts.entry(stem).or_insert(0) += 1;
    }

    paths
        .iter()
        .map(|path| {
            let stem = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .to_string();
            if counts.get(&stem).copied().unwrap_or(0) <= 1 {
                stem
            } else {
                format!("{}_{}", stem, stable_short_path_hash(path))
            }
        })
        .collect()
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
