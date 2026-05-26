use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};
use cpp2rust_ffi::{build_project, write_project};

/// cpp2rust-ffi 命令行入口。
#[derive(Debug, Parser)]
#[command(name = "cpp2rust-ffi")]
#[command(about = "基于正则的 C++ 头文件到 Rust hicc FFI 生成器")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// 初始化一个 hicc 输出项目。
    Init {
        #[arg(long)]
        input: PathBuf,
        #[arg(long)]
        output: PathBuf,
        #[arg(long)]
        lib_name: Option<String>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Init {
            input,
            output,
            lib_name,
        } => {
            let inferred = lib_name.unwrap_or_else(|| {
                input
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("generated_hicc")
                    .to_string()
            });
            let project = build_project(&input, &output, &inferred)?;
            write_project(&output, &project)?;
            println!(
                "generated hicc project at {} from {}",
                output.display(),
                input.display()
            );
        }
    }
    Ok(())
}
