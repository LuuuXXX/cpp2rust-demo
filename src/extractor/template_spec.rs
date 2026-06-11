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
use crate::ffi_model::{TemplateClassSpec, TemplateFnSpec, TemplateInstanceSpec};

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
    // 跳过畸形模板：模板类必须同时具备名称与至少一个类型参数才能生成泛型骨架
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

/// 由 `CppAst` 构建模板实例化别名规格（v6 Phase B 增强）。
///
/// 实例化追踪策略：仅依据当前编译单元中「以具体类型实例化某个本文件声明的模板类」的
/// 使用点收集。当前实现扫描所有来自当前文件的类（含包装类）的**字段类型**，
/// 例如 `Stack<int> impl;`，并以 `(模板名, 具体类型实参)` 记录实例化。这覆盖了
/// v6 §3.2 中 025（`Stack<int>`/`Stack<double>`）与 027（`Matrix<int>`/`Matrix<double>`）
/// 等以包装类持有模板成员的典型写法。
///
/// 只为「本文件声明的模板类」生成别名，避免把三方库模板（如 `std::vector`）误纳入。
/// 复杂或非 POD 的类类型实参会被标记 `needs_class_type`，由生成器附 TODO 提示用户确认
/// 对应的 hicc 类型（符合 v6 §8 的降级策略）。
pub(super) fn build_template_instances(ast: &CppAst) -> Vec<TemplateInstanceSpec> {
    use std::collections::BTreeSet;

    // 本文件声明的模板类名集合（仅这些模板才生成实例化别名）
    let template_names: BTreeSet<&str> = ast
        .template_classes
        .iter()
        .filter(|tc| tc.is_from_current_file && !tc.name.is_empty())
        .map(|tc| tc.name.as_str())
        .collect();
    if template_names.is_empty() {
        return Vec::new();
    }

    let mut seen: BTreeSet<(String, Vec<String>)> = BTreeSet::new();
    let mut out: Vec<TemplateInstanceSpec> = Vec::new();

    for class in ast.classes.iter().filter(|c| c.is_from_current_file) {
        for field in &class.fields {
            let core = strip_type_decorations(&field.type_name);
            let Some((name, args)) = split_template_use(core) else {
                continue;
            };
            if !template_names.contains(name.as_str()) {
                continue;
            }
            // 去重：相同（模板名, 实参列表）只生成一个别名
            let key = (name.clone(), args.clone());
            if !seen.insert(key) {
                continue;
            }
            if let Some(spec) = build_instance_spec(&name, &args) {
                out.push(spec);
            }
        }
    }

    out
}

/// 去除类型字符串的指针/引用/cv 限定与首尾空白，返回核心类型名。
///
/// 例如 `Matrix<int> *` → `Matrix<int>`、`const Stack<double> &` → `Stack<double>`。
fn strip_type_decorations(ty: &str) -> &str {
    let mut s = clean_type(ty).trim();
    // 反复剥离首部的 cv 限定（const/volatile）与尾部的 `*` / `&` 及空白
    loop {
        let before = s.len();
        s = s.trim();
        if let Some(rest) = s.strip_prefix("const ") {
            s = rest.trim_start();
        }
        if let Some(rest) = s.strip_prefix("volatile ") {
            s = rest.trim_start();
        }
        s = s.trim_end_matches(['*', '&', ' ', '\t']);
        if s.len() == before {
            break;
        }
    }
    s.trim()
}

/// 将 `Name<arg1, arg2>` 拆分为 `(Name, [arg1, arg2])`；不含顶层尖括号时返回 `None`。
///
/// 正确处理嵌套尖括号（如 `Map<int, vector<int>>` → `("Map", ["int", "vector<int>"])`）。
fn split_template_use(core: &str) -> Option<(String, Vec<String>)> {
    let lt = core.find('<')?;
    if !core.ends_with('>') {
        return None;
    }
    let name = core[..lt].trim().to_string();
    if name.is_empty() {
        return None;
    }
    let inner = &core[lt + 1..core.len() - 1];
    let mut args = Vec::new();
    let mut depth = 0i32;
    let mut start = 0usize;
    for (i, ch) in inner.char_indices() {
        match ch {
            '<' => depth += 1,
            '>' => depth -= 1,
            ',' if depth == 0 => {
                let arg = inner[start..i].trim();
                if !arg.is_empty() {
                    args.push(arg.to_string());
                }
                start = i + 1;
            }
            _ => {}
        }
    }
    let last = inner[start..].trim();
    if !last.is_empty() {
        args.push(last.to_string());
    }
    if args.is_empty() {
        None
    } else {
        Some((name, args))
    }
}

/// 由模板名与具体类型实参列表构建一个实例化别名规格。
fn build_instance_spec(template_name: &str, args: &[String]) -> Option<TemplateInstanceSpec> {
    let mut hicc_args = Vec::with_capacity(args.len());
    let mut suffix = String::new();
    let mut needs_class_type = false;

    for arg in args {
        let (hicc, suf, is_class) = map_instance_arg(arg);
        hicc_args.push(hicc);
        suffix.push_str(&suf);
        needs_class_type |= is_class;
    }

    if hicc_args.is_empty() {
        return None;
    }

    Some(TemplateInstanceSpec {
        alias_name: format!("{}{}", template_name, suffix),
        template_name: template_name.to_string(),
        hicc_args,
        needs_class_type,
    })
}

/// 将单个具体类型实参映射为 `(hicc 实参, 别名后缀, 是否为类类型)`。
///
/// - POD 标量（`int`/`double`/`bool`...）→ `hicc::Pod<i32>`，后缀为 Rust 类型的 PascalCase（如 `I32`）；
/// - 其他（类类型）→ 保留清理后的 C++ 类型名，后缀为其标识符片段，并标记需要用户确认 hicc 类型。
fn map_instance_arg(arg: &str) -> (String, String, bool) {
    let core = strip_type_decorations(arg);
    let rust = cpp_to_rust(core);
    if is_pod_scalar(&rust) {
        (
            format!("hicc::Pod<{}>", rust),
            pascal_case_ident(&rust),
            false,
        )
    } else {
        // 非 POD：保留 C++ 类型名，提示用户在 hicc 侧确认对应类型
        (core.to_string(), pascal_case_ident(core), true)
    }
}

/// 判断 Rust 类型是否为可直接用 `hicc::Pod<...>` 包装的基础标量类型。
fn is_pod_scalar(rust: &str) -> bool {
    matches!(
        rust,
        "i8" | "i16"
            | "i32"
            | "i64"
            | "i128"
            | "isize"
            | "u8"
            | "u16"
            | "u32"
            | "u64"
            | "u128"
            | "usize"
            | "f32"
            | "f64"
            | "bool"
            | "char"
    )
}

/// 将标识符片段转为 PascalCase 别名后缀（仅保留字母数字，首字母大写）。
///
/// 例如 `i32` → `I32`、`f64` → `F64`、`std::string` → `StdString`。
fn pascal_case_ident(s: &str) -> String {
    let mut out = String::new();
    let mut upper_next = true;
    for ch in s.chars() {
        if ch.is_alphanumeric() {
            if upper_next {
                out.extend(ch.to_uppercase());
                upper_next = false;
            } else {
                out.push(ch);
            }
        } else {
            // 分隔符（`:`、空格、`_` 等）触发下一个字母大写
            upper_next = true;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_type_decorations_removes_ptr_ref_cv() {
        assert_eq!(strip_type_decorations("Matrix<int> *"), "Matrix<int>");
        assert_eq!(
            strip_type_decorations("const Stack<double> &"),
            "Stack<double>"
        );
        assert_eq!(strip_type_decorations("Stack<int>"), "Stack<int>");
    }

    #[test]
    fn split_template_use_parses_name_and_args() {
        assert_eq!(
            split_template_use("Stack<int>"),
            Some(("Stack".to_string(), vec!["int".to_string()]))
        );
        assert_eq!(
            split_template_use("Matrix<double>"),
            Some(("Matrix".to_string(), vec!["double".to_string()]))
        );
        // 非模板使用返回 None
        assert_eq!(split_template_use("int"), None);
        assert_eq!(split_template_use("Foo"), None);
    }

    #[test]
    fn split_template_use_handles_nested_angles() {
        assert_eq!(
            split_template_use("Map<int, vector<int>>"),
            Some((
                "Map".to_string(),
                vec!["int".to_string(), "vector<int>".to_string()]
            ))
        );
    }

    #[test]
    fn map_instance_arg_pod_scalar_uses_pod_wrapper() {
        let (hicc, suffix, is_class) = map_instance_arg("int");
        assert_eq!(hicc, "hicc::Pod<i32>");
        assert_eq!(suffix, "I32");
        assert!(!is_class);

        let (hicc, suffix, is_class) = map_instance_arg("double");
        assert_eq!(hicc, "hicc::Pod<f64>");
        assert_eq!(suffix, "F64");
        assert!(!is_class);
    }

    #[test]
    fn map_instance_arg_class_type_kept_and_flagged() {
        let (hicc, suffix, is_class) = map_instance_arg("std::string");
        assert_eq!(hicc, "std::string");
        assert_eq!(suffix, "StdString");
        assert!(is_class);
    }

    #[test]
    fn build_instance_spec_produces_alias() {
        let spec = build_instance_spec("Stack", &["int".to_string()]).unwrap();
        assert_eq!(spec.alias_name, "StackI32");
        assert_eq!(spec.template_name, "Stack");
        assert_eq!(spec.hicc_args, vec!["hicc::Pod<i32>".to_string()]);
        assert!(!spec.needs_class_type);
    }

    #[test]
    fn pascal_case_ident_capitalizes_segments() {
        assert_eq!(pascal_case_ident("i32"), "I32");
        assert_eq!(pascal_case_ident("std::string"), "StdString");
        assert_eq!(pascal_case_ident("unsigned int"), "UnsignedInt");
    }
}
