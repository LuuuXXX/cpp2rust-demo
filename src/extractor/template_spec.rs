//! 模板类 / 模板函数绑定提取器 — v6 Phase B
//!
//! 从 [`CppAst`] 的 `template_classes` / `template_functions` 字段构建
//! [`TemplateClassSpec`] / [`TemplateFnSpec`]，供生成器输出泛型
//! `import_class!` / `import_lib!` 骨架。
//!
//! 本模块产出的规格仅在生成器侧 `CPP2RUST_GEN_TEMPLATES` 开关开启时被输出，
//! 因此即便始终构建这些规格，默认产物也逐字节不变（见
//! `generator::hicc_codegen::templates_enabled`）。
//!
//! ## 设计取舍
//!
//! 模板成员方法签名可能含 `T`、`T&`、`T*` 等依赖泛型参数的类型。`cpp_to_rust`
//! 无法识别 `T`，会将其原样保留，因此生成的是「泛型骨架」：直接映射的成员函数
//! 用 `#[cpp(method = ...)]` 输出，参数/返回类型中的 `T` 保留为 Rust 侧泛型类型。
//! 复杂场景（`T::OutputRef` 等）由用户基于骨架补全，符合 v6 方案 §8 的降级策略。

use crate::ast_parser::{CppAst, TemplateClassInfo, TemplateFunctionInfo};
use crate::ffi_model::{TemplateClassSpec, TemplateFnSpec};

use super::class_spec::build_method_binding;
use super::type_mapper::{clean_type, cpp_to_rust};
use super::{normalize_ptr_spacing, sanitize_fn_name, sanitize_param_name};

/// 由 `CppAst` 构建模板类 / 模板函数绑定规格。
///
/// 仅纳入来自当前编译单元（`is_from_current_file`）的模板声明，与 v5 对普通
/// 类/函数的来源过滤策略一致，避免把被 `#include` 的三方库模板纳入绑定。
pub(super) fn build_template_specs(ast: &CppAst) -> (Vec<TemplateClassSpec>, Vec<TemplateFnSpec>) {
    let classes = ast
        .template_classes
        .iter()
        .filter(|tc| tc.is_from_current_file)
        .filter_map(build_template_class_spec)
        .collect();

    let functions = ast
        .template_functions
        .iter()
        .filter(|tf| tf.is_from_current_file)
        .filter_map(build_template_fn_spec)
        .collect();

    (classes, functions)
}

/// 构建单个模板类规格。无可映射的公有成员方法时返回 `None`（避免空骨架）。
fn build_template_class_spec(tc: &TemplateClassInfo) -> Option<TemplateClassSpec> {
    if tc.name.is_empty() || tc.type_params.is_empty() {
        return None;
    }

    // 仅纳入公有、非特殊（构造/析构）成员方法；构造函数按 hicc 约定应在
    // import_lib! 中声明，此处骨架先不生成，留待用户补全。
    let methods = tc
        .methods
        .iter()
        .filter(|m| m.accessibility == "public")
        .filter(|m| !m.is_constructor && !m.is_destructor && !m.is_static)
        .filter_map(build_method_binding)
        .collect::<Vec<_>>();

    if methods.is_empty() {
        return None;
    }

    Some(TemplateClassSpec {
        name: tc.name.clone(),
        type_params: tc.type_params.clone(),
        methods,
    })
}

/// 构建单个模板函数规格。
fn build_template_fn_spec(tf: &TemplateFunctionInfo) -> Option<TemplateFnSpec> {
    if tf.name.is_empty() || tf.type_params.is_empty() {
        return None;
    }

    // C++ 模板函数签名要求：`ret func<T,...>(arg_type, ...)`，参数列表只含类型、不含名字。
    let param_types: Vec<String> = tf
        .params
        .iter()
        .map(|p| normalize_ptr_spacing(clean_type(&p.type_name)))
        .collect();
    let ret_clean = if tf.return_type.is_empty() || tf.return_type == "void" {
        "void".to_string()
    } else {
        normalize_ptr_spacing(clean_type(&tf.return_type))
    };
    let type_args = tf.type_params.join(", ");
    let cpp_sig = format!(
        "{} {}<{}>({})",
        ret_clean,
        tf.name,
        type_args,
        param_types.join(", ")
    );

    let params: Vec<(String, String)> = tf
        .params
        .iter()
        .enumerate()
        .map(|(i, p)| (sanitize_param_name(&p.name, i), cpp_to_rust(&p.type_name)))
        .collect();
    let ret_type = if tf.return_type.is_empty() || tf.return_type == "void" {
        None
    } else {
        let rt = cpp_to_rust(&tf.return_type);
        if rt.is_empty() {
            None
        } else {
            Some(rt)
        }
    };

    Some(TemplateFnSpec {
        name: tf.name.clone(),
        type_params: tf.type_params.clone(),
        cpp_sig,
        rust_name: sanitize_fn_name(&tf.name),
        params,
        ret_type,
    })
}
