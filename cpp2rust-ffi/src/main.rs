mod ast_parser;
mod capture;
mod extractor;
mod generator;
mod instantiation_tracker;
mod postprocessor;
mod types;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use ast_parser::parse_cpp_file;
use generator::hicc_codegen::HiccCodegen;
use generator::project_generator::ProjectGenerator;
use types::CppAst;

/// cpp2rust-ffi - C++ to Rust Safe FFI automated scaffolding tool (v5)
#[derive(Parser, Debug)]
#[command(
    name = "cpp2rust-ffi",
    version = "0.1.0",
    about = "Automatically generate Rust FFI scaffolding from C++ source code"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Initialize: generate Rust FFI project from captured .c2rust files
    Init {
        /// Input directory containing .c2rust files (C2RUST_FEATURE_ROOT)
        #[arg(short, long, default_value = ".c2rust/v5")]
        input: PathBuf,

        /// Output directory for generated Rust project
        #[arg(short, long, default_value = "./rust_hicc")]
        output: PathBuf,

        /// C++ source directory (for build.rs generation)
        #[arg(short = 'c', long, default_value = "./cpp")]
        cpp_dir: PathBuf,

        /// Project name (defaults to basename of output dir)
        #[arg(short = 'n', long)]
        name: Option<String>,
    },

    /// Merge: merge multiple rust_hicc files into a single project
    Merge {
        /// Input Rust project directory
        #[arg(short, long, default_value = "./rust_hicc")]
        input: PathBuf,
    },

    /// Parse: show AST information from a C++ file (for debugging)
    Parse {
        /// C++ or .c2rust file to parse
        file: PathBuf,

        /// Output format: text or json
        #[arg(short, long, default_value = "text")]
        format: String,
    },

    /// Generate: generate Rust FFI directly from a C++ source file
    Generate {
        /// C++ source file to process
        #[arg(short, long)]
        input: PathBuf,

        /// Output Rust file (defaults to stdout)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// C++ source directory for build.rs
        #[arg(short = 'c', long)]
        cpp_dir: Option<PathBuf>,

        /// Project name
        #[arg(short = 'n', long)]
        name: Option<String>,

        /// Generate full project (Cargo.toml + build.rs + src/main.rs)
        #[arg(long)]
        project: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { input, output, cpp_dir, name } => {
            cmd_init(&input, &output, &cpp_dir, name.as_deref())
        }
        Commands::Merge { input } => cmd_merge(&input),
        Commands::Parse { file, format } => cmd_parse(&file, &format),
        Commands::Generate { input, output, cpp_dir, name, project } => {
            cmd_generate(&input, output.as_deref(), cpp_dir.as_deref(), name.as_deref(), project)
        }
    }
}

/// init 命令：从 .c2rust 文件生成 Rust FFI 项目
fn cmd_init(input: &Path, output: &Path, cpp_dir: &Path, name: Option<&str>) -> Result<()> {
    println!("cpp2rust-ffi init");
    println!("  input:   {}", input.display());
    println!("  output:  {}", output.display());
    println!("  cpp_dir: {}", cpp_dir.display());

    // 收集 .c2rust 或 .cpp 文件
    let files = collect_source_files(input)?;
    if files.is_empty() {
        // 如果没有 .c2rust 文件，尝试直接解析 cpp 目录
        eprintln!("Warning: No .c2rust files found in {}. Using cpp_dir directly.", input.display());
        let cpp_files = collect_source_files(cpp_dir)?;
        if cpp_files.is_empty() {
            anyhow::bail!("No source files found in {} or {}", input.display(), cpp_dir.display());
        }
        return process_files(&cpp_files, output, cpp_dir, name);
    }

    process_files(&files, output, cpp_dir, name)
}

fn process_files(files: &[PathBuf], output: &Path, cpp_dir: &Path, name: Option<&str>) -> Result<()> {
    let mut merged_ast = CppAst::default();

    for file in files {
        println!("  Parsing: {}", file.display());
        match parse_cpp_file(file) {
            Ok(ast) => {
                // 合并 AST
                merged_ast.classes.extend(ast.classes);
                merged_ast.functions.extend(ast.functions);
                merged_ast.enums.extend(ast.enums);
                merged_ast.consts.extend(ast.consts);
                for inc in ast.includes {
                    if !merged_ast.includes.contains(&inc) {
                        merged_ast.includes.push(inc);
                    }
                }
                if merged_ast.source_name.is_empty() {
                    merged_ast.source_name = ast.source_name;
                }
            }
            Err(e) => {
                eprintln!("Warning: Failed to parse {}: {}", file.display(), e);
            }
        }
    }

    let project_name = name
        .map(|s| s.to_string())
        .or_else(|| {
            output.file_name()
                .and_then(|n| n.to_str())
                .map(|s| s.to_string())
        })
        .unwrap_or_else(|| merged_ast.source_name.clone());

    if project_name.is_empty() {
        merged_ast.source_name = "ffi_project".to_string();
    }

    let generator = ProjectGenerator::new(output, &project_name);
    generator.generate(&merged_ast, cpp_dir)?;

    println!("Generated Rust project in: {}", output.display());
    Ok(())
}

/// merge 命令：合并多个 rust_hicc 文件
fn cmd_merge(input: &Path) -> Result<()> {
    println!("cpp2rust-ffi merge");
    println!("  input: {}", input.display());

    let main_rs = input.join("src").join("main.rs");
    if !main_rs.exists() {
        anyhow::bail!("No src/main.rs found in {}", input.display());
    }

    println!("Merge complete (single-file project, no merge needed)");
    Ok(())
}

/// parse 命令：显示 C++ AST
fn cmd_parse(file: &Path, format: &str) -> Result<()> {
    let ast = parse_cpp_file(file)
        .with_context(|| format!("Failed to parse {}", file.display()))?;

    match format {
        "json" => {
            let json = serde_json::to_string_pretty(&ast)?;
            println!("{}", json);
        }
        _ => {
            println!("=== AST for {} ===", file.display());
            println!("Classes: {}", ast.classes.len());
            for class in &ast.classes {
                println!("  - CXXRecordDecl: {}", class.name);
                for method in &class.methods {
                    println!("    - CXXMethodDecl: {}", method.name);
                }
            }
            println!("Functions: {}", ast.functions.len());
            for func in &ast.functions {
                println!("  - FunctionDecl: {}", func.name);
            }
            println!("Enums: {}", ast.enums.len());
            for enum_ in &ast.enums {
                println!("  - EnumDecl: {}", enum_.name);
            }
            println!("Consts: {}", ast.consts.len());
        }
    }

    Ok(())
}

/// generate 命令：从 C++ 文件直接生成 Rust FFI
fn cmd_generate(
    input: &Path,
    output: Option<&Path>,
    cpp_dir: Option<&Path>,
    name: Option<&str>,
    project: bool,
) -> Result<()> {
    let ast = parse_cpp_file(input)
        .with_context(|| format!("Failed to parse {}", input.display()))?;

    if project {
        // 生成完整项目
        let out_dir = output.unwrap_or_else(|| Path::new("./rust_hicc"));
        let default_cpp = input.parent().unwrap_or(Path::new("."));
        let cpp_dir = cpp_dir.unwrap_or(default_cpp);
        let project_name = name
            .map(|s| s.to_string())
            .or_else(|| {
                out_dir.file_name()
                    .and_then(|n| n.to_str())
                    .map(|s| s.to_string())
            })
            .unwrap_or_else(|| ast.source_name.clone());

        let generator = ProjectGenerator::new(out_dir, &project_name);
        generator.generate(&ast, cpp_dir)?;
        println!("Generated Rust project in: {}", out_dir.display());
    } else {
        // 生成单个 main.rs 文件
        let codegen = HiccCodegen::new();
        let content = codegen.generate(&ast);

        if let Some(out_path) = output {
            std::fs::write(out_path, &content)
                .with_context(|| format!("Failed to write to {}", out_path.display()))?;
            println!("Written to: {}", out_path.display());
        } else {
            print!("{}", content);
        }
    }

    Ok(())
}

/// 收集目录中的 C++ 或 .c2rust 源文件
fn collect_source_files(dir: &Path) -> Result<Vec<PathBuf>> {
    if !dir.exists() {
        return Ok(Vec::new());
    }

    if dir.is_file() {
        return Ok(vec![dir.to_path_buf()]);
    }

    let mut files = Vec::new();
    for entry in WalkDir::new(dir).follow_links(true) {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            if let Some(ext) = path.extension() {
                let ext = ext.to_string_lossy().to_lowercase();
                if matches!(ext.as_str(), "c2rust" | "cpp" | "cc" | "cxx") {
                    files.push(path.to_path_buf());
                }
            }
        }
    }
    files.sort();
    Ok(files)
}
