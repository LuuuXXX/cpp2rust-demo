use clap::{Args, Parser, Subcommand};
use cpp2rust_demo::commands;
use cpp2rust_demo::error::Result;
use std::path::PathBuf;

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
    /// 捕获 C++ 构建过程并准备 Rust 脚手架输入
    Init(InitArgs),
    /// 将每个符号生成的输出合并到模块级文件
    Merge(MergeArgs),
}

#[derive(Args)]
struct InitArgs {
    /// 特性名称（默认："default"）
    #[arg(long, default_value = "default")]
    feature: String,

    /// 要执行的构建命令（置于 '--' 之后）
    /// 示例：cpp2rust-demo init -- make -j4
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
    /// 要合并的特性名称（默认："default"；可多次指定以合并多个 feature）
    #[arg(long = "feature", value_name = "FEATURE")]
    features: Vec<String>,

    /// 将 Cargo 项目结构输出到指定目录（meta/ src/ build.rs Cargo.toml）；不指定则输出到标准位置
    #[arg(long, value_name = "DIR")]
    output_dir: Option<PathBuf>,
}

fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Commands::Init(args) => commands::init::run_init(&args.feature, &args.build_cmd),
        Commands::Merge(args) => commands::merge::run_merge(args.features, args.output_dir),
    }
}

fn main() {
    let cli = Cli::parse();
    if let Err(e) = run(cli) {
        eprintln!("错误：{:#}", e);
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn cli_help_does_not_panic() {
        Cli::command().debug_assert();
    }

    #[test]
    fn merge_default_feature() {
        let args = Cli::try_parse_from(["cpp2rust-demo", "merge"]).unwrap();
        let Commands::Merge(merge) = args.command else {
            panic!("expected Merge");
        };
        // 未提供 --feature 时，features 为空（代码内默认为 "default"）
        assert!(merge.features.is_empty());
    }

    #[test]
    fn merge_custom_feature() {
        let args =
            Cli::try_parse_from(["cpp2rust-demo", "merge", "--feature", "core_lib"]).unwrap();
        let Commands::Merge(merge) = args.command else {
            panic!("expected Merge");
        };
        assert_eq!(merge.features, vec!["core_lib"]);
    }

    #[test]
    fn merge_multi_feature() {
        let args = Cli::try_parse_from([
            "cpp2rust-demo",
            "merge",
            "--feature",
            "core",
            "--feature",
            "net",
        ])
        .unwrap();
        let Commands::Merge(merge) = args.command else {
            panic!("expected Merge");
        };
        assert_eq!(merge.features, vec!["core", "net"]);
    }

    #[test]
    fn init_requires_build_cmd() {
        // 不带 build_cmd 时应解析失败
        let result = Cli::try_parse_from(["cpp2rust-demo", "init"]);
        assert!(result.is_err());
    }
}
