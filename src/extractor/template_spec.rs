//! 模板绑定规格构建（Phase B）
//!
//! 消费 [`crate::ast_parser::CppAst`] 中 Phase A 提取的 `template_classes` /
//! `template_functions` 字段，构建 [`TemplateClassSpec`] / [`TemplateFnSpec`]。
//!
//! 这些规格仅在 `CPP2RUST_GEN_TEMPLATES` 开启时由 `hicc_codegen` 消费生成泛型骨架；
//! 默认不消费，因此构建过程本身不改变任何默认生成产物。

use super::class_spec::build_method_binding;
use super::type_mapper::{clean_type, cpp_to_rust, to_snake_case};
use super::{is_rust_keyword, normalize_ptr_spacing, ret_type_from_cpp, sanitize_fn_name};
use crate::ast_parser::{TemplateClassInfo, TemplateFunctionInfo};
use crate::ffi_model::{TemplateClassSpec, TemplateFnSpec};

/// 从 `TemplateClassInfo` 构建 `TemplateClassSpec`（泛型 `import_class!` 骨架）。
///
/// 仅纳入 public、非 ctor/dtor、非 static 的成员方法，跳过 operator 重载与
/// Rust 关键字方法名（与 `build_class_spec` 的过滤口径一致）。无可生成成员方法时返回 `None`。
fn build_template_class_spec(tc: &TemplateClassInfo) -> Option<TemplateClassSpec> {
    let type_params: Vec<String> = tc.type_params.iter().map(|p| p.name.clone()).collect();

    let mut methods = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for m in &tc.methods {
        if m.is_constructor || m.is_destructor || m.accessibility != "public" || m.is_static {
            continue;
        }
        if m.name.starts_with("operator") || is_rust_keyword(&to_snake_case(&m.name)) {
            continue;
        }
        // 复用 import_class! 的方法绑定构建（含 volatile / 成员函数指针过滤）。
        // 注意：模板成员方法的类型可能含泛型形参 `T`，不在已导出类名集合内，
        // 因此此处不做 is_mappable_rust_type 过滤——泛型骨架本就以 `T` 表达。
        if let Some(mb) = build_method_binding(m) {
            // hicc::import_class! 不支持同名方法，去重保留首个。
            if seen.insert(mb.rust_name.clone()) {
                methods.push(mb);
            }
        }
    }

    if methods.is_empty() {
        return None;
    }

    Some(TemplateClassSpec {
        name: tc.name.clone(),
        type_params,
        methods,
    })
}

/// 从 `TemplateFunctionInfo` 构建 `TemplateFnSpec`（泛型 `import_lib!` 骨架）。
///
/// `cpp_sig` 保留模板形参（如 `void do_swap<T>(T *, T *)`），供 `#[cpp(func = "...")]` 使用。
fn build_template_fn_spec(tf: &TemplateFunctionInfo) -> TemplateFnSpec {
    let type_params: Vec<String> = tf.type_params.iter().map(|p| p.name.clone()).collect();

    let params: Vec<(String, String)> = tf
        .params
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let name = if p.name.is_empty() {
                format!("arg{}", i)
            } else {
                p.name.clone()
            };
            (name, cpp_to_rust(&p.type_name))
        })
        .collect();

    let ret_type = ret_type_from_cpp(&tf.return_type);

    // C++ 签名：`<ret> name<T,...>(types)`。参数仅保留类型（指针紧贴类型）。
    let param_types: Vec<String> = tf
        .params
        .iter()
        .map(|p| normalize_ptr_spacing(clean_type(&p.type_name)))
        .collect();
    let generics = if type_params.is_empty() {
        String::new()
    } else {
        format!("<{}>", type_params.join(", "))
    };
    let ret_clean = normalize_ptr_spacing(clean_type(&tf.return_type));
    let ret_part = if tf.return_type.is_empty() || tf.return_type == "void" {
        "void".to_string()
    } else {
        ret_clean
    };
    let cpp_sig = format!(
        "{} {}{}({})",
        ret_part,
        tf.name,
        generics,
        param_types.join(", ")
    );

    TemplateFnSpec {
        name: tf.name.clone(),
        type_params,
        cpp_sig,
        rust_name: sanitize_fn_name(&tf.name),
        params,
        ret_type,
    }
}

/// 从 AST 的模板字段构建模板绑定规格。
///
/// AST 层（`parse_preprocessed`）已将 `template_classes` / `template_functions`
/// 限定为**用户代码**（含用户头文件，排除系统头），因此此处不再按文件来源过滤，
/// 直接映射全部条目。
pub(super) fn build_template_specs(
    template_classes: &[TemplateClassInfo],
    template_functions: &[TemplateFunctionInfo],
) -> (Vec<TemplateClassSpec>, Vec<TemplateFnSpec>) {
    let classes = template_classes
        .iter()
        .filter_map(build_template_class_spec)
        .collect();

    let fns = template_functions
        .iter()
        .map(build_template_fn_spec)
        .collect();

    (classes, fns)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast_parser::{MethodInfo, ParamInfo, TemplateParamInfo};

    fn type_param(name: &str) -> TemplateParamInfo {
        TemplateParamInfo {
            name: name.to_string(),
            kind: "type".to_string(),
        }
    }

    fn method(name: &str, ret: &str, params: Vec<(&str, &str)>, is_const: bool) -> MethodInfo {
        MethodInfo {
            name: name.to_string(),
            return_type: ret.to_string(),
            params: params
                .into_iter()
                .map(|(n, t)| ParamInfo {
                    name: n.to_string(),
                    type_name: t.to_string(),
                    has_default: false,
                })
                .collect(),
            is_const,
            is_volatile: false,
            is_virtual: false,
            is_pure_virtual: false,
            is_static: false,
            is_constructor: false,
            is_destructor: false,
            is_inline: false,
            accessibility: "public".to_string(),
            body_offset: None,
            is_override: false,
            is_default: false,
        }
    }

    fn template_class(
        name: &str,
        params: Vec<&str>,
        methods: Vec<MethodInfo>,
    ) -> TemplateClassInfo {
        TemplateClassInfo {
            name: name.to_string(),
            is_struct: false,
            is_abstract: false,
            type_params: params.into_iter().map(type_param).collect(),
            bases: vec![],
            methods,
            fields: vec![],
            is_in_namespace: false,
            is_from_current_file: true,
        }
    }

    #[test]
    fn builds_generic_class_spec_with_type_param() {
        let tc = template_class(
            "Stack",
            vec!["T"],
            vec![
                method("size", "int", vec![], true),
                method("push", "void", vec![("value", "T")], false),
            ],
        );
        let spec = build_template_class_spec(&tc).expect("应生成模板类规格");
        assert_eq!(spec.name, "Stack");
        assert_eq!(spec.type_params, vec!["T".to_string()]);
        assert_eq!(spec.methods.len(), 2);
        // push 的参数类型应保留泛型形参 T
        let push = spec.methods.iter().find(|m| m.rust_name == "push").unwrap();
        assert!(push.cpp_sig.contains("push(T value)"), "{}", push.cpp_sig);
    }

    #[test]
    fn skips_ctor_dtor_and_static() {
        let mut ctor = method("Stack", "", vec![], false);
        ctor.is_constructor = true;
        let mut stat = method("make", "int", vec![], false);
        stat.is_static = true;
        let tc = template_class(
            "Stack",
            vec!["T"],
            vec![ctor, stat, method("empty", "bool", vec![], true)],
        );
        let spec = build_template_class_spec(&tc).expect("应生成模板类规格");
        assert_eq!(spec.methods.len(), 1);
        assert_eq!(spec.methods[0].rust_name, "empty");
    }

    #[test]
    fn empty_template_class_returns_none() {
        let tc = template_class("Empty", vec!["T"], vec![]);
        assert!(build_template_class_spec(&tc).is_none());
    }

    #[test]
    fn builds_template_fn_spec_with_generics_in_sig() {
        let tf = TemplateFunctionInfo {
            name: "do_swap".to_string(),
            return_type: "void".to_string(),
            params: vec![
                ParamInfo {
                    name: "a".to_string(),
                    type_name: "T *".to_string(),
                    has_default: false,
                },
                ParamInfo {
                    name: "b".to_string(),
                    type_name: "T *".to_string(),
                    has_default: false,
                },
            ],
            type_params: vec![type_param("T")],
            is_variadic: false,
            is_from_current_file: true,
        };
        let spec = build_template_fn_spec(&tf);
        assert_eq!(spec.name, "do_swap");
        assert_eq!(spec.cpp_sig, "void do_swap<T>(T*, T*)");
        assert_eq!(spec.rust_name, "do_swap");
        assert!(spec.ret_type.is_none());
        assert_eq!(spec.params.len(), 2);
    }

    #[test]
    fn build_template_specs_maps_all_user_templates() {
        // AST 层已限定为用户代码，故 build_template_specs 直接映射全部条目。
        let a = template_class("A", vec!["T"], vec![method("get", "int", vec![], true)]);
        let b = template_class("B", vec!["T"], vec![method("get", "int", vec![], true)]);
        let (classes, _) = build_template_specs(&[a, b], &[]);
        assert_eq!(classes.len(), 2);
        assert_eq!(classes[0].name, "A");
        assert_eq!(classes[1].name, "B");
    }
}
