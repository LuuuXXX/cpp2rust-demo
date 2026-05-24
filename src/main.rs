mod ast;
mod capture;
mod codegen;
mod error;
mod layout;
mod merge;
mod selector;

use anyhow::Context;
use ast::parse_translation_unit;
use capture::{build_hook, read_compiler_options, run_capture};
use clap::{Parser, Subcommand};
use codegen::generate_feature_project;
use error::Result;
use layout::{relative_display, FeatureLayout};
use merge::merge_feature;
use selector::select_files;
use std::fs;

#[derive(Debug, Parser)]
#[command(author, version, about)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Init {
        #[arg(long, default_value = "default")]
        feature: String,
        #[arg(last = true, required = true)]
        build_command: Vec<String>,
    },
    Merge {
        #[arg(long, default_value = "default")]
        feature: String,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Init { feature, build_command } => init(feature, build_command).context("init failed")?,
        Commands::Merge { feature } => merge(feature).context("merge failed")?,
    }
    Ok(())
}

fn init(feature: String, build_command: Vec<String>) -> Result<()> {
    let project_root = std::env::current_dir()?;
    let layout = FeatureLayout::new(&project_root, feature);
    layout.ensure_dirs()?;
    layout.write_build_command(&build_command)?;
    build_hook(&layout)?;
    run_capture(&layout, &build_command)?;

    let captures = layout.scan_ast_files()?;
    let selected = select_files(
        &captures
            .iter()
            .map(|capture| capture.source_rel.clone())
            .collect::<Vec<_>>(),
    )?;
    fs::write(layout.selected_files_path(), serde_json::to_string_pretty(&selected)?)?;

    let mut units = Vec::new();
    for selection in &selected {
        let capture = captures
            .iter()
            .find(|capture| capture.source_rel == selection.source_rel)
            .expect("selected capture must exist");
        let source_path = layout.project_root.join(&selection.source_rel);
        let _compiler_options = read_compiler_options(&capture.opts_path)?;
        let unit = parse_translation_unit(&capture.json_path, &source_path)?;
        units.push(unit);
    }
    generate_feature_project(&layout, &units)?;

    println!(
        "generated {} translation unit(s) under {}",
        units.len(),
        relative_display(&layout.feature_root)
    );
    Ok(())
}

fn merge(feature: String) -> Result<()> {
    let project_root = std::env::current_dir()?;
    let layout = FeatureLayout::new(&project_root, feature);
    let summary = merge_feature(&layout)?;
    println!(
        "merged {} module(s) under {}",
        summary.modules.len(),
        relative_display(&layout.rust_dir)
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    fn test_dir(name: &str) -> std::path::PathBuf {
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("target/test-work/main")
            .join(format!("{name}-{}", std::process::id()));
        if root.exists() {
            std::fs::remove_dir_all(&root).unwrap();
        }
        std::fs::create_dir_all(&root).unwrap();
        root
    }

    #[test]
    fn cli_has_subcommands() {
        let command = Cli::command();
        let names = command
            .get_subcommands()
            .map(|sub| sub.get_name().to_string())
            .collect::<Vec<_>>();
        assert!(names.contains(&"init".to_string()));
        assert!(names.contains(&"merge".to_string()));
    }

    #[test]
    fn merge_command_runs_on_generated_structure() {
        let dir = test_dir("merge");
        let layout = FeatureLayout::new(&dir, "default");
        layout.ensure_dirs().unwrap();
        fs::create_dir_all(layout.rust_src_dir.join("main")).unwrap();
        fs::write(layout.rust_src_dir.join("lib.rs"), "pub mod main;\n").unwrap();
        fs::write(layout.rust_src_dir.join("main/mod.rs"), "class Foo;\n").unwrap();
        fs::write(layout.rust_src_dir.join("main/types.rs"), "pub type Num = i32;\n").unwrap();
        let cwd = std::env::current_dir().unwrap();
        std::env::set_current_dir(&dir).unwrap();
        merge("default".into()).unwrap();
        std::env::set_current_dir(cwd).unwrap();
        assert!(layout.rust_dir.join("src.2/lib.rs").exists());
    }

    #[test]
    fn init_requires_build_command() {
        let parsed = Cli::try_parse_from(["cpp2rust-demo", "init"]);
        assert!(parsed.is_err());
    }
}
