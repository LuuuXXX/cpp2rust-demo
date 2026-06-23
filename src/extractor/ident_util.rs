//! 标识符与参数名工具（Phase 3 辅助）
//!
//! Rust 关键字检查、参数名/函数名清理、C++ 参数列表格式化。
//! 这些工具在多个 extractor 子模块中共用，集中维护以避免重复。

use super::type_mapper::to_snake_case;
use crate::ast_parser::ParamInfo;

/// 判断是否为 Rust 关键字（Rust 2021 严格关键字 + 保留关键字）。
///
/// 用于参数名、函数名、方法名的消歧处理，防止生成的 Rust 代码出现关键字冲突。
pub(super) fn is_rust_keyword(s: &str) -> bool {
    matches!(
        s,
        // 严格关键字（Rust 2021）
        "as" | "async" | "await" | "break" | "const" | "continue" | "crate" | "dyn"
        | "else" | "enum" | "extern" | "false" | "fn" | "for" | "if" | "impl" | "in"
        | "let" | "loop" | "match" | "mod" | "move" | "mut" | "pub" | "ref" | "return"
        | "self" | "Self" | "static" | "struct" | "super" | "trait" | "true" | "type"
        | "union" | "unsafe" | "use" | "where" | "while"
        // 保留关键字
        | "abstract" | "become" | "box" | "do" | "final" | "gen" | "macro" | "override"
        | "priv" | "try" | "typeof" | "unsized" | "virtual" | "yield"
    )
}

/// 参数名称清理（避免 Rust 关键字）
pub(super) fn sanitize_param_name(name: &str, idx: usize) -> String {
    match name {
        "" | "_" => format!("arg{}", idx),
        _ if is_rust_keyword(name) => format!("{}_", name),
        _ => name.to_string(),
    }
}

/// 函数/方法名清理：先转 snake_case，再对关键字加 `_` 后缀。
///
/// 用于 `build_method_binding` 和 `build_fn_binding` 生成 `rust_name`，
/// 确保结果不与 Rust 关键字冲突。
pub(super) fn sanitize_fn_name(name: &str) -> String {
    let snake = to_snake_case(name);
    if is_rust_keyword(&snake) {
        format!("{}_", snake)
    } else {
        snake
    }
}

/// 格式化 C++ 参数列表字符串
pub(super) fn format_params_cpp(params: &[ParamInfo]) -> String {
    use super::type_mapper::clean_type;
    use super::normalize_ptr_spacing;
    params
        .iter()
        .map(|p| {
            let ty = normalize_ptr_spacing(clean_type(&p.type_name));
            if p.name.is_empty() || p.name == "_" {
                ty.to_string()
            } else {
                format!("{} {}", ty, p.name)
            }
        })
        .collect::<Vec<_>>()
        .join(", ")
}
