use anyhow::anyhow;
use clap::{Args, Parser, Subcommand};
use cpp2rust_demo::ast_parser;
use cpp2rust_demo::capture;
use cpp2rust_demo::error::Result;
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
        println!("No files selected – skipping AST parsing.");
        return Ok(());
    }

    println!("\nRunning AST parser stub...");
    match ast_parser::parse_preprocessed(&selected[0]) {
        Ok(ast) => println!("Parsed AST for {}", ast.file.display()),
        Err(err) => println!("{}", err),
    }

    if args.skip_failed {
        println!("--skip-failed is not implemented yet (stub).");
    }

    println!("\n✓ cpp2rust-demo init completed for Phase T/0.");
    println!("\nOutput structure:");
    println!("  .cpp2rust/{}/", feature);
    println!("    ├── c/          (captured .cpp2rust files)");
    println!("    ├── meta/       (build_cmd.txt, selected_files.json)");
    println!("    └── rust/       (reserved for later phases)");

    Ok(())
}

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
