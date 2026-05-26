use anyhow::Result;
use std::path::{Path, PathBuf};
use std::fs;

use crate::types::CppAst;
use crate::generator::hicc_codegen::HiccCodegen;

/// 生成完整的 Rust 项目结构
pub struct ProjectGenerator {
    /// 输出目录
    pub output_dir: PathBuf,
    /// 项目名称
    pub project_name: String,
}

impl ProjectGenerator {
    pub fn new(output_dir: impl AsRef<Path>, project_name: &str) -> Self {
        Self {
            output_dir: output_dir.as_ref().to_path_buf(),
            project_name: project_name.to_string(),
        }
    }

    /// 从 CppAst 生成完整项目
    pub fn generate(&self, ast: &CppAst, cpp_dir: &Path) -> Result<()> {
        let src_dir = self.output_dir.join("src");
        fs::create_dir_all(&src_dir)?;

        // 生成 Cargo.toml
        self.write_cargo_toml()?;

        // 生成 build.rs
        self.write_build_rs(ast, cpp_dir)?;

        // 生成 src/main.rs
        let codegen = HiccCodegen::new();
        let main_content = codegen.generate(ast);
        fs::write(src_dir.join("main.rs"), main_content)?;

        Ok(())
    }

    fn write_cargo_toml(&self) -> Result<()> {
        let content = format!(
            r#"[package]
name = "{name}"
version = "0.1.0"
edition = "2021"

[dependencies]
hicc = {{ version = "0.2" }}

[build-dependencies]
hicc-build = {{ version = "0.2" }}
cc = "1.0"
"#,
            name = self.project_name
        );
        fs::write(self.output_dir.join("Cargo.toml"), content)?;
        Ok(())
    }

    fn write_build_rs(&self, ast: &CppAst, cpp_dir: &Path) -> Result<()> {
        let cpp_files: Vec<String> = collect_cpp_files(cpp_dir)?;
        let cpp_files_str = cpp_files.iter()
            .map(|f| format!("    cc_build.file(cpp_dir.join(\"{}\"));", f))
            .collect::<Vec<_>>()
            .join("\n");

        let rerun_files: Vec<String> = cpp_files.iter()
            .map(|f| format!("    println!(\"cargo::rerun-if-changed=../cpp/{}\");", f))
            .collect();
        let rerun_str = rerun_files.join("\n");

        let content = format!(
            r#"fn main() {{
    let cpp_dir = std::path::PathBuf::from("../cpp");

    let mut build = hicc_build::Build::new();
    use std::ops::DerefMut;
    let cc_build: &mut cc::Build = build.deref_mut();
    cc_build.include(&cpp_dir);
    cc_build.include(".");
    cc_build.cpp(true);
{cpp_files}

    build.rust_file("src/main.rs").compile("{name}");

    println!("cargo::rustc-link-lib={name}");
    #[cfg(not(all(target_os = "windows", target_env = "msvc")))]
    println!("cargo::rustc-link-lib=stdc++");
    println!("cargo::rerun-if-changed=src/main.rs");
{rerun}
}}
"#,
            name = self.project_name,
            cpp_files = cpp_files_str,
            rerun = rerun_str,
        );
        fs::write(self.output_dir.join("build.rs"), content)?;
        Ok(())
    }
}

/// 收集目录中的 C++ 源文件名（不含路径）
fn collect_cpp_files(dir: &Path) -> Result<Vec<String>> {
    let mut files = Vec::new();
    if !dir.exists() {
        return Ok(files);
    }
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if let Some(ext) = path.extension() {
            let ext = ext.to_string_lossy().to_lowercase();
            if matches!(ext.as_str(), "cpp" | "cc" | "cxx" | "c++") {
                if let Some(name) = path.file_name() {
                    files.push(name.to_string_lossy().to_string());
                }
            }
        }
    }
    files.sort();
    Ok(files)
}
