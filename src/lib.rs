pub mod codegen;
pub mod ir;
pub mod parser;
pub mod typemap;

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use codegen::{generate_build_rs, generate_output_cargo_toml, generate_rust_source};
use ir::ParsedHeader;
use parser::parse_header_file;

/// 生成项目所需的全部文本文件。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedProject {
    pub cargo_toml: String,
    pub build_rs: String,
    pub main_rs: String,
    pub parsed_headers: Vec<ParsedHeader>,
}

/// 扫描输入目录并生成输出项目内容。
pub fn build_project(
    input_dir: &Path,
    output_dir: &Path,
    lib_name: &str,
) -> Result<GeneratedProject> {
    let headers = collect_files(input_dir, &["h", "hpp", "hh"])?;
    if headers.is_empty() {
        bail!("no header files found in {}", input_dir.display());
    }

    let cpp_files = collect_files(input_dir, &["cpp", "cc", "cxx"])?;
    let parsed_headers = headers
        .iter()
        .map(|path| parse_header_file(path))
        .collect::<Result<Vec<_>>>()?;

    let main_rs = generate_rust_source(&parsed_headers, lib_name)?;
    let cargo_toml = generate_output_cargo_toml(lib_name);
    let build_rs = generate_build_rs(input_dir, output_dir, lib_name, &headers, &cpp_files)?;

    Ok(GeneratedProject {
        cargo_toml,
        build_rs,
        main_rs,
        parsed_headers,
    })
}

/// 将输出项目落盘。
pub fn write_project(output_dir: &Path, project: &GeneratedProject) -> Result<()> {
    fs::create_dir_all(output_dir.join("src"))
        .with_context(|| format!("failed to create {}", output_dir.join("src").display()))?;
    fs::write(output_dir.join("Cargo.toml"), &project.cargo_toml).with_context(|| {
        format!(
            "failed to write {}",
            output_dir.join("Cargo.toml").display()
        )
    })?;
    fs::write(output_dir.join("build.rs"), &project.build_rs)
        .with_context(|| format!("failed to write {}", output_dir.join("build.rs").display()))?;
    fs::write(output_dir.join("src/main.rs"), &project.main_rs).with_context(|| {
        format!(
            "failed to write {}",
            output_dir.join("src/main.rs").display()
        )
    })?;
    Ok(())
}

fn collect_files(dir: &Path, extensions: &[&str]) -> Result<Vec<PathBuf>> {
    let mut files = fs::read_dir(dir)
        .with_context(|| format!("failed to read directory {}", dir.display()))?
        .filter_map(|entry| entry.ok().map(|item| item.path()))
        .filter(|path| {
            path.is_file()
                && path
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .map(|ext| {
                        extensions
                            .iter()
                            .any(|candidate| ext.eq_ignore_ascii_case(candidate))
                    })
                    .unwrap_or(false)
        })
        .collect::<Vec<_>>();
    files.sort();
    Ok(files)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collect_files_returns_sorted_matches() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let files = collect_files(&root.join("examples/001_hello_world/cpp"), &["h"]).unwrap();
        assert_eq!(files.len(), 1);
        assert!(files[0].ends_with("hello_world.h"));
    }
}
