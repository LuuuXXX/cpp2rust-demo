mod ast_parser;
mod capture;
mod extractor;
mod generator;
mod instantiation_tracker;
mod postprocessor;
mod types;

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use walkdir::WalkDir;

use crate::ast_parser::parse_preprocessed;

#[derive(Debug, Parser)]
#[command(name = "cpp2rust-ffi")]
#[command(about = "Generate Rust hicc FFI scaffolding from preprocessed C++ capture output")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Init {
        #[arg(short = 'i', long = "input")]
        input: PathBuf,
        #[arg(short = 'o', long = "output")]
        output: PathBuf,
    },
    Merge {
        #[arg(short = 'i', long = "input")]
        input: PathBuf,
    },
    Parse {
        file: PathBuf,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Init { input, output } => run_init(&input, &output),
        Commands::Merge { input } => generator::merge_project(&input),
        Commands::Parse { file } => {
            let ast = parse_preprocessed(&file)?;
            println!("{}", serde_json::to_string_pretty(&ast)?);
            Ok(())
        }
    }
}

fn run_init(input: &Path, output: &Path) -> Result<()> {
    let files = collect_c2rust_files(input)?;
    if files.is_empty() {
        bail!("no .c2rust files found under {}", input.display());
    }

    if output.exists() {
        fs::remove_dir_all(output)
            .with_context(|| format!("cleaning existing output directory {}", output.display()))?;
    }

    let asts = files
        .iter()
        .map(|path| parse_preprocessed(path))
        .collect::<Result<Vec<_>>>()?;

    generator::generate_project(&asts, output)
}

fn collect_c2rust_files(input: &Path) -> Result<Vec<PathBuf>> {
    if !input.exists() {
        bail!("input path does not exist: {}", input.display());
    }

    let mut files = if input.is_file() {
        vec![input.to_path_buf()]
    } else {
        WalkDir::new(input)
            .into_iter()
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.into_path())
            .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("c2rust"))
            .collect::<Vec<_>>()
    };
    files.sort();
    Ok(files)
}
