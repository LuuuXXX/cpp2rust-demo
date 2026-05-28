use anyhow::anyhow;
use clap::{Args, Parser, Subcommand};
use cpp2rust_demo::ast_parser;
use cpp2rust_demo::capture;
use cpp2rust_demo::error::Result;
use cpp2rust_demo::extractor;
use cpp2rust_demo::generator::hicc_codegen;
use cpp2rust_demo::generator::project_generator;
use cpp2rust_demo::layout::{self, FeatureLayout};
use cpp2rust_demo::selector::{FileSelector, InteractiveSelector};

#[derive(Parser)]
#[command(name = "cpp2rust-demo")]
#[command(about = "Minimal C++-to-Rust workflow: build capture + Rust scaffolding")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Capture a C++ build and prepare Rust scaffolding inputs
    Init(InitArgs),
    /// Merge generated per-symbol outputs into module-level files
    Merge(MergeArgs),
    /// Parse a .cpp2rust file and print the AST structure (for debugging)
    Parse(ParseArgs),
}

#[derive(Args)]
struct InitArgs {
    /// Feature name (default: "default")
    #[arg(long, default_value = "default")]
    feature: String,

    /// Skip files that fail later phases (stub in Phase T/0)
    #[arg(long)]
    skip_failed: bool,

    /// Build command to execute (use after '--')
    /// Example: cpp2rust-demo init -- make -j4
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
    /// Feature name (default: "default")
    #[arg(long, default_value = "default")]
    feature: String,
}

#[derive(Args)]
struct ParseArgs {
    /// Path to the .cpp2rust file to parse
    file: std::path::PathBuf,
}

fn run_parse(args: ParseArgs) -> Result<()> {
    let file = &args.file;
    if !file.exists() {
        return Err(anyhow!("file not found: {}", file.display()));
    }
    println!("Parsing: {}", file.display());
    println!();
    let ast = ast_parser::parse_preprocessed(file)?;
    ast.print_tree();
    Ok(())
}

fn run_merge(args: MergeArgs) -> Result<()> {
    let feature = &args.feature;
    let cwd = std::env::current_dir().map_err(|e| anyhow!("current_dir: {}", e))?;
    let project_root = layout::find_project_root(&cwd);
    let feature_root = project_root.join(".cpp2rust").join(feature);

    println!("=== cpp2rust-demo merge ===");
    println!("Project root : {}", project_root.display());
    println!("Feature      : {}", feature);
    println!();

    if !feature_root.exists() {
        return Err(anyhow!(
            "feature '{}' not found at {}; run init first",
            feature,
            feature_root.display()
        ));
    }

    println!("Merge not yet implemented (future phase).");
    println!("Existing feature root: {}", feature_root.display());
    Ok(())
}

fn run_init(args: InitArgs) -> Result<()> {
    let feature = &args.feature;
    let build_cmd = &args.build_cmd;

    let cwd = std::env::current_dir().map_err(|e| anyhow!("current_dir: {}", e))?;
    let project_root = layout::find_project_root(&cwd);

    println!("=== cpp2rust-demo init ===");
    println!("Project root : {}", project_root.display());
    println!("Feature      : {}", feature);
    println!("Skip failed  : {}", args.skip_failed);
    println!("Build command: {}", build_cmd.join(" "));
    println!();

    let lo = FeatureLayout::new(project_root.clone(), feature);
    lo.create_dirs()?;
    lo.save_build_cmd(build_cmd)?;

    let hook_so = capture::build_hook()?;
    capture::run_with_hook(&cwd, build_cmd, &project_root, &lo.feature_root, &hook_so)?;

    let captured = layout::scan_cpp2rust_files(&lo.c_dir)?;
    println!("\nCaptured {} .cpp2rust file(s)", captured.len());

    if captured.is_empty() {
        println!("Warning: no .cpp2rust files were generated.");
        println!("Make sure your build command compiles C++ files.");
        return Ok(());
    }

    let sel = InteractiveSelector;
    let selected = sel.select(&captured)?;
    println!("{} file(s) selected for this feature", selected.len());

    lo.save_selected_files(&selected)?;

    if selected.is_empty() {
        println!("No files selected – skipping code generation.");
        return Ok(());
    }

    println!("\nRunning AST parser and code generation on selected files...");
    let mut unit_names: Vec<String> = Vec::new();
    let mut parse_errors = 0usize;

    for path in &selected {
        // 从 `.cpp2rust` 路径推导原始 `.cpp` 路径
        // hook 命名规则：<c_dir>/<relative_from_project_root>.cpp2rust
        // 例：<c_dir>/src/foo.cpp.cpp2rust → project_root/src/foo.cpp
        let original_cpp = {
            let rel = path.strip_prefix(&lo.c_dir).unwrap_or(path.as_path());
            let rel_str = rel.to_string_lossy();
            let cpp_rel = rel_str
                .strip_suffix(".cpp2rust")
                .unwrap_or(&rel_str)
                .to_string();
            project_root.join(cpp_rel)
        };

        // unit_name = 原始 cpp 文件的文件名（不含扩展名）
        let unit_name = original_cpp
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unit")
            .to_string();

        match ast_parser::parse_preprocessed(path) {
            Ok(ast) => {
                println!(
                    "  {} → {} class(es), {} fn(s), {} enum(s)",
                    path.display(),
                    ast.classes.len(),
                    ast.functions.len(),
                    ast.enums.len()
                );

                let (system_includes, project_header) =
                    extractor::read_source_includes(&original_cpp);
                let spec = extractor::extract(
                    &ast,
                    &unit_name,
                    &system_includes,
                    project_header.as_deref(),
                );
                let code = hicc_codegen::generate(&spec);
                project_generator::write_unit_rs(&lo.rust_dir, &unit_name, &code)?;
                unit_names.push(unit_name);
            }
            Err(err) => {
                parse_errors += 1;
                if args.skip_failed {
                    eprintln!("  Warning: parse failed for {}: {:#}", path.display(), err);
                } else {
                    return Err(err);
                }
            }
        }
    }

    if parse_errors > 0 {
        println!("\nWarning: {} file(s) failed to parse (--skip-failed active).", parse_errors);
    }

    // 生成 Cargo.toml 和 lib.rs
    project_generator::write_cargo_toml(&lo.rust_dir, feature)?;
    project_generator::write_lib_rs(&lo.rust_dir, &unit_names)?;

    println!("\n✓ cpp2rust-demo init completed.");
    println!("\nOutput structure:");
    println!("  .cpp2rust/{}/", feature);
    println!("    ├── c/          (captured .cpp2rust files)");
    println!("    ├── meta/       (build_cmd.txt, selected_files.json)");
    println!("    └── rust/       (generated Rust project: Cargo.toml, src/lib.rs, src/*.rs)");
    println!();
    println!("Generated {} unit file(s) in .cpp2rust/{}/rust/src/", unit_names.len(), feature);

    Ok(())
}

fn main() {
    let cli = Cli::parse();
    let result = match cli.command {
        Commands::Init(args) => run_init(args),
        Commands::Merge(args) => run_merge(args),
        Commands::Parse(args) => run_parse(args),
    };
    if let Err(e) = result {
        eprintln!("Error: {:#}", e);
        std::process::exit(1);
    }
}
