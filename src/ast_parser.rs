//! C++ AST 解析（Phase 1 实现，当前为 stub）

use anyhow::Result;
use std::path::Path;

/// Parsed C++ AST（当前为空 stub，Phase 1 会完善）
pub struct CppAst {
    pub file: std::path::PathBuf,
}

/// 解析 .cpp2rust 预处理文件（stub）
pub fn parse_preprocessed(_file: &Path) -> Result<CppAst> {
    anyhow::bail!("AST parsing not yet implemented (Phase 1)")
}
