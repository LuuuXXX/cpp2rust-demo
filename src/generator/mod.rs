pub mod hicc_codegen;
pub mod project_generator;

use std::path::Path;

use anyhow::Result;

use crate::types::CppAst;

pub fn generate_project(asts: &[CppAst], output_dir: &Path) -> Result<()> {
    project_generator::write_project(asts, output_dir)
}

pub fn merge_project(output_dir: &Path) -> Result<()> {
    project_generator::merge_project(output_dir)
}
