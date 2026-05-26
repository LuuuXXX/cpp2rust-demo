use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};
use cpp2rust_ffi::{build_project, write_project, TodoSummary};

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
            print_todo_summary(&project.todo_summary);
        }
    }
    Ok(())
}

/// 将 `cpp2rust-todo` 注释摘要打印到 stderr（仅在有内容时输出）。
fn print_todo_summary(summary: &TodoSummary) {
    if summary.total() == 0 {
        return;
    }
    eprintln!("\ncpp2rust-todo summary ({} item(s) need attention):", summary.total());
    if summary.op_count > 0 {
        eprintln!(
            "  [OP] {}  operator shim(s) generated — consider implementing std::ops traits",
            summary.op_count
        );
    }
    if summary.fr_count > 0 {
        eprintln!(
            "  [FR] {}  friend function(s) — review Rust-side access control",
            summary.fr_count
        );
    }
    if summary.lm_count > 0 {
        eprintln!(
            "  [LM] {}  fn-pointer / lambda parameter(s) — consider typed Rust closure wrappers",
            summary.lm_count
        );
    }
    if summary.va_count > 0 {
        eprintln!(
            "  [VA] {}  variadic-template fixed-arity expansion(s) — consider a unified Rust API",
            summary.va_count
        );
    }
    eprintln!("Grep for `cpp2rust-todo` in generated src/main.rs for details.");
}
