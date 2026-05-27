use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

use crate::generator::hicc_codegen;
use crate::types::CppAst;

pub fn write_project(asts: &[CppAst], output_dir: &Path) -> Result<()> {
    fs::create_dir_all(output_dir.join("src"))
        .with_context(|| format!("creating output directory {}", output_dir.display()))?;

    fs::write(output_dir.join("Cargo.toml"), cargo_toml())
        .with_context(|| format!("writing {}", output_dir.join("Cargo.toml").display()))?;
    fs::write(output_dir.join("build.rs"), build_rs())
        .with_context(|| format!("writing {}", output_dir.join("build.rs").display()))?;
    fs::write(
        output_dir.join("src").join("main.rs"),
        hicc_codegen::generate_entrypoint(asts),
    )
    .with_context(|| format!("writing {}", output_dir.join("src/main.rs").display()))?;

    Ok(())
}

pub fn merge_project(output_dir: &Path) -> Result<()> {
    let src_dir = output_dir.join("src");
    fs::create_dir_all(&src_dir)
        .with_context(|| format!("creating source directory {}", src_dir.display()))?;

    let mut module_files = fs::read_dir(&src_dir)
        .with_context(|| format!("reading {}", src_dir.display()))?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("rs"))
        .filter(|path| path.file_name().and_then(|name| name.to_str()) != Some("lib.rs"))
        .collect::<Vec<_>>();
    module_files.sort();

    let merged = module_files
        .into_iter()
        .filter_map(|path| fs::read_to_string(&path).ok())
        .collect::<Vec<_>>()
        .join("\n\n");

    if !merged.is_empty() {
        fs::write(src_dir.join("lib.rs"), merged)
            .with_context(|| format!("writing {}", src_dir.join("lib.rs").display()))?;
    }

    Ok(())
}

fn cargo_toml() -> &'static str {
    r#"[package]
name = "cpp2rust-generated"
version = "0.1.0"
edition = "2021"

[dependencies]
hicc = { version = "0.2" }

[build-dependencies]
hicc-build = { version = "0.2" }
cc = "1"
"#
}

fn build_rs() -> &'static str {
    r#"fn main() {
    let mut build = hicc_build::Build::new();
    build.rust_file("src/main.rs").compile("cpp2rust_generated");
    println!("cargo::rerun-if-changed=src/main.rs");
}
"#
}
