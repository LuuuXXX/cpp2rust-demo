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
    /// 生成代码中各类 `cpp2rust-todo` 注释的数量汇总。
    pub todo_summary: TodoSummary,
}

/// `cpp2rust-todo[TAG]` 注释计数，供 CLI 输出摘要使用。
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TodoSummary {
    /// `[OP]` 运算符重载 shim，建议实现 std::ops traits。
    pub op_count: usize,
    /// `[FR]` 友元函数，建议在 Rust 侧明确访问控制。
    pub fr_count: usize,
    /// `[LM]` 函数指针/lambda 参数，建议封装为类型化 Rust 闭包。
    pub lm_count: usize,
    /// `[RTTI]` 整数类型枚举判别器模式（getType/getTypeName），新增子类时需同步维护枚举。
    pub rtti_count: usize,
    /// `[VA]` 可变参数模板固定元数展开，建议在 Rust 侧统一 API。
    pub va_count: usize,
}

impl TodoSummary {
    fn from_source(source: &str) -> Self {
        Self {
            op_count: source.matches("cpp2rust-todo[OP]").count(),
            fr_count: source.matches("cpp2rust-todo[FR]").count(),
            lm_count: source.matches("cpp2rust-todo[LM]").count(),
            rtti_count: source.matches("cpp2rust-todo[RTTI]").count(),
            va_count: source.matches("cpp2rust-todo[VA]").count(),
        }
    }

    /// 所有标签计数之和。
    pub fn total(&self) -> usize {
        self.op_count + self.fr_count + self.lm_count + self.rtti_count + self.va_count
    }
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
    let todo_summary = TodoSummary::from_source(&main_rs);
    let cargo_toml = generate_output_cargo_toml(lib_name);
    let build_rs = generate_build_rs(input_dir, output_dir, lib_name, &headers, &cpp_files)?;

    Ok(GeneratedProject {
        cargo_toml,
        build_rs,
        main_rs,
        parsed_headers,
        todo_summary,
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
