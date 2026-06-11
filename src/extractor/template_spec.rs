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
use crate::ffi_model::{TemplateClassSpec, TemplateFnSpec, TemplateInstantiation};

/// Rust 标量原始类型集合：实例化实参映射结果属于该集合时，才视为可用 `hicc::Pod<T>`
/// 包裹的 POD 类型；否则（指针/引用/复合类型）视为不可映射，跳过该实例化。
const RUST_SCALARS: &[&str] = &[
    "i8", "i16", "i32", "i64", "i128", "isize", "u8", "u16", "u32", "u64", "u128", "usize", "f32",
    "f64", "bool", "char",
];

/// 将 C++ 标识符片段转为 UpperCamelCase（用于拼接实例化别名）。
///
/// 以非字母数字字符切分单词，每个单词首字母大写。例如：
/// `"int"` → `"Int"`、`"unsigned int"` → `"UnsignedInt"`、`"Foo"` → `"Foo"`。
fn to_camel_case(s: &str) -> String {
    s.split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty())
        .map(|w| {
            let mut chars = w.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect()
}

/// 将单个 C++ 模板实参映射为 (Rust 实例化片段, 别名后缀)。
///
/// - 已导出的 C++ 类类型 → 直接用类名（hicc class 本身实现 `AbiType`）；
/// - POD 标量（`int`/`double`/...）→ `hicc::Pod<i32>` 等；
/// - 其余（含指针/引用/未导出复合类型）→ `None`，调用方跳过该实例化。
fn map_instantiation_arg(arg: &str, exported_class_names: &[&str]) -> Option<(String, String)> {
    let a = arg.trim();
    if a.is_empty() {
        return None;
    }
    if exported_class_names.contains(&a) {
        return Some((a.to_string(), to_camel_case(a)));
    }
    let rust = cpp_to_rust(a);
    if RUST_SCALARS.contains(&rust.as_str()) {
        return Some((format!("hicc::Pod<{}>", rust), to_camel_case(a)));
    }
    None
}

/// 在一段类型字符串中查找 `name<...>` 形式的实例化，返回每次出现的顶层实参列表。
///
/// 通过括号深度匹配最外层 `<...>`，并按顶层逗号切分实参（尊重嵌套泛型）。
/// `name` 需位于单词边界（前一字符非字母数字/下划线），避免 `MyStack` 误匹配 `Stack`。
fn find_instantiation_args(name: &str, usage: &str) -> Vec<Vec<String>> {
    let mut results = Vec::new();
    let bytes = usage.as_bytes();
    let nb = name.as_bytes();
    let mut i = 0usize;
    // 用 `<=` 以覆盖 name 恰好位于末尾的情形；切片上界可等于 len，访问安全。
    // 此后的 `<` 检查（j < bytes.len()）会自然排除末尾无后继 `<` 的非实例化。
    while i + nb.len() <= bytes.len() {
        if &bytes[i..i + nb.len()] == nb {
            // 单词边界检查：前一字符不能是标识符字符。
            let boundary_ok = i == 0 || {
                let prev = bytes[i - 1];
                !(prev.is_ascii_alphanumeric() || prev == b'_')
            };
            // name 之后（跳过空白）必须紧跟 '<'。
            let mut j = i + nb.len();
            while j < bytes.len() && bytes[j].is_ascii_whitespace() {
                j += 1;
            }
            if boundary_ok && j < bytes.len() && bytes[j] == b'<' {
                if let Some((args, end)) = split_top_level_args(usage, j) {
                    results.push(args);
                    i = end;
                    continue;
                }
            }
        }
        i += 1;
    }
    results
}

/// 从 `usage[open]`（指向 `<`）起匹配平衡的 `<...>`，按顶层逗号切分实参。
///
/// 返回 (实参列表, 结束位置)，结束位置为闭合 `>` 之后的字节索引；不平衡时返回 `None`。
fn split_top_level_args(usage: &str, open: usize) -> Option<(Vec<String>, usize)> {
    let bytes = usage.as_bytes();
    let mut depth = 0i32;
    let mut args = Vec::new();
    let mut start = open + 1;
    let mut k = open;
    while k < bytes.len() {
        match bytes[k] {
            b'<' => depth += 1,
            b'>' => {
                depth -= 1;
                if depth == 0 {
                    let seg = usage[start..k].trim();
                    if !seg.is_empty() {
                        args.push(seg.to_string());
                    }
                    return Some((args, k + 1));
                }
            }
            b',' if depth == 1 => {
                args.push(usage[start..k].trim().to_string());
                start = k + 1;
            }
            _ => {}
        }
        k += 1;
    }
    None
}

/// 从用户代码的类型用法中发现某模板类的具体实例化别名（去重、按别名排序）。
///
/// 仅纳入所有实参均可映射的实例化；任一实参不可映射（如含指针或未导出类型）则跳过。
fn collect_instantiations(
    class_name: &str,
    type_usages: &[String],
    exported_class_names: &[&str],
) -> Vec<TemplateInstantiation> {
    let mut seen = std::collections::HashSet::new();
    let mut insts = Vec::new();
    for usage in type_usages {
        for args in find_instantiation_args(class_name, usage) {
            if args.is_empty() {
                continue;
            }
            let mut rust_parts = Vec::new();
            let mut suffix_parts = Vec::new();
            let mut all_mapped = true;
            for a in &args {
                match map_instantiation_arg(a, exported_class_names) {
                    Some((rust, suffix)) => {
                        rust_parts.push(rust);
                        suffix_parts.push(suffix);
                    }
                    None => {
                        all_mapped = false;
                        break;
                    }
                }
            }
            if !all_mapped {
                continue;
            }
            let alias = format!("{}{}", class_name, suffix_parts.concat());
            if seen.insert(alias.clone()) {
                insts.push(TemplateInstantiation {
                    rust_target: format!("{}<{}>", class_name, rust_parts.join(", ")),
                    cpp_target: format!("{}<{}>", class_name, args.join(", ")),
                    alias,
                    cpp_args: args,
                });
            }
        }
    }
    insts.sort_by(|a, b| a.alias.cmp(&b.alias));
    insts
}

/// 从 `TemplateClassInfo` 构建 `TemplateClassSpec`（泛型 `import_class!` 骨架）。
///
/// 仅纳入 public、非 ctor/dtor、非 static 的成员方法，跳过 operator 重载与
/// Rust 关键字方法名（与 `build_class_spec` 的过滤口径一致）。无可生成成员方法时返回 `None`。
fn build_template_class_spec(
    tc: &TemplateClassInfo,
    type_usages: &[String],
    exported_class_names: &[&str],
) -> Option<TemplateClassSpec> {
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

    let instantiations = collect_instantiations(&tc.name, type_usages, exported_class_names);

    Some(TemplateClassSpec {
        name: tc.name.clone(),
        type_params,
        methods,
        instantiations,
        has_default_ctor: template_has_default_ctor(tc),
    })
}

/// 判断模板类是否具备可访问的默认构造函数。
///
/// 规则（与 C++ 语义对齐）：
/// - 抽象类无法实例化，直接返回 `false`；
/// - 若存在零参的 public 构造函数（含 `= default`）→ `true`；
/// - 若无任何用户声明的构造函数 → 编译器隐式生成默认构造函数 → `true`；
/// - 否则（仅有带参/非 public 构造函数）→ `false`。
fn template_has_default_ctor(tc: &TemplateClassInfo) -> bool {
    if tc.is_abstract {
        return false;
    }
    let ctors: Vec<&crate::ast_parser::MethodInfo> =
        tc.methods.iter().filter(|m| m.is_constructor).collect();
    if ctors.is_empty() {
        return true;
    }
    ctors
        .iter()
        .any(|c| c.accessibility == "public" && c.params.is_empty())
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
///
/// `type_usages` 是用户代码中出现的类型字符串集合（类字段、方法/函数签名等），
/// 用于发现模板类的具体实例化（如 `Stack<int>`）；`exported_class_names` 为已生成
/// `import_class!` 的具体类名集合，用于将实例化实参中的类类型映射为 hicc class。
pub(super) fn build_template_specs(
    template_classes: &[TemplateClassInfo],
    template_functions: &[TemplateFunctionInfo],
    type_usages: &[String],
    exported_class_names: &[&str],
) -> (Vec<TemplateClassSpec>, Vec<TemplateFnSpec>) {
    let classes = template_classes
        .iter()
        .filter_map(|tc| build_template_class_spec(tc, type_usages, exported_class_names))
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
        let spec = build_template_class_spec(&tc, &[], &[]).expect("应生成模板类规格");
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
        let spec = build_template_class_spec(&tc, &[], &[]).expect("应生成模板类规格");
        assert_eq!(spec.methods.len(), 1);
        assert_eq!(spec.methods[0].rust_name, "empty");
    }

    #[test]
    fn empty_template_class_returns_none() {
        let tc = template_class("Empty", vec!["T"], vec![]);
        assert!(build_template_class_spec(&tc, &[], &[]).is_none());
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
        let (classes, _) = build_template_specs(&[a, b], &[], &[], &[]);
        assert_eq!(classes.len(), 2);
        assert_eq!(classes[0].name, "A");
        assert_eq!(classes[1].name, "B");
    }

    #[test]
    fn to_camel_case_handles_multiword_and_class() {
        assert_eq!(to_camel_case("int"), "Int");
        assert_eq!(to_camel_case("double"), "Double");
        assert_eq!(to_camel_case("unsigned int"), "UnsignedInt");
        assert_eq!(to_camel_case("Foo"), "Foo");
    }

    #[test]
    fn map_arg_pod_and_class_and_reject() {
        // POD 标量 → hicc::Pod<...>
        assert_eq!(
            map_instantiation_arg("int", &[]),
            Some(("hicc::Pod<i32>".to_string(), "Int".to_string()))
        );
        assert_eq!(
            map_instantiation_arg("double", &[]),
            Some(("hicc::Pod<f64>".to_string(), "Double".to_string()))
        );
        // 已导出类类型 → 直接用类名
        assert_eq!(
            map_instantiation_arg("Widget", &["Widget"]),
            Some(("Widget".to_string(), "Widget".to_string()))
        );
        // 含指针 / 未导出复合类型 → 不可映射
        assert_eq!(map_instantiation_arg("int *", &[]), None);
        assert_eq!(map_instantiation_arg("std::string", &[]), None);
    }

    #[test]
    fn find_instantiation_args_balanced_and_boundary() {
        // 基础匹配
        assert_eq!(
            find_instantiation_args("Stack", "Stack<int>"),
            vec![vec!["int".to_string()]]
        );
        // 多实参（顶层逗号切分）
        assert_eq!(
            find_instantiation_args("Map", "Map<int, double>"),
            vec![vec!["int".to_string(), "double".to_string()]]
        );
        // 嵌套泛型不被内部逗号误切
        assert_eq!(
            find_instantiation_args("Box", "Box<Map<int, char>>"),
            vec![vec!["Map<int, char>".to_string()]]
        );
        // 单词边界：MyStack 不应匹配 Stack
        assert!(find_instantiation_args("Stack", "MyStack<int>").is_empty());
    }

    #[test]
    fn collect_instantiations_dedup_and_sorted() {
        let usages = vec![
            "Stack<int>".to_string(),
            "Stack<double>".to_string(),
            "Stack<int>".to_string(),   // 重复
            "Stack<int *>".to_string(), // 含指针 → 跳过
        ];
        let insts = collect_instantiations("Stack", &usages, &[]);
        assert_eq!(insts.len(), 2, "{:?}", insts);
        // 按别名排序：StackDouble < StackInt
        assert_eq!(insts[0].alias, "StackDouble");
        assert_eq!(insts[0].rust_target, "Stack<hicc::Pod<f64>>");
        assert_eq!(insts[1].alias, "StackInt");
        assert_eq!(insts[1].rust_target, "Stack<hicc::Pod<i32>>");
    }

    #[test]
    fn build_template_class_spec_collects_instantiations() {
        let tc = template_class(
            "Stack",
            vec!["T"],
            vec![method("size", "int", vec![], true)],
        );
        let usages = vec!["Stack<int>".to_string(), "Stack<double>".to_string()];
        let spec = build_template_class_spec(&tc, &usages, &[]).expect("应生成模板类规格");
        assert_eq!(spec.instantiations.len(), 2);
        assert_eq!(spec.instantiations[0].alias, "StackDouble");
        assert_eq!(spec.instantiations[1].alias, "StackInt");
    }

    #[test]
    fn collect_instantiations_fills_cpp_target() {
        let usages = vec!["Stack<int>".to_string()];
        let insts = collect_instantiations("Stack", &usages, &[]);
        assert_eq!(insts.len(), 1);
        // C++ 侧目标形式保留原始实参，用于 make_unique 的 #[cpp(func = "...")]。
        assert_eq!(insts[0].cpp_target, "Stack<int>");
        assert_eq!(insts[0].rust_target, "Stack<hicc::Pod<i32>>");
    }

    #[test]
    fn default_ctor_detected_for_explicit_zero_arg_ctor() {
        let mut ctor = method("Stack", "", vec![], false);
        ctor.is_constructor = true;
        let tc = template_class(
            "Stack",
            vec!["T"],
            vec![ctor, method("size", "int", vec![], true)],
        );
        assert!(template_has_default_ctor(&tc));
    }

    #[test]
    fn default_ctor_detected_when_no_user_ctor() {
        // 无任何用户声明构造函数 → 编译器隐式默认构造函数。
        let tc = template_class("Stack", vec!["T"], vec![method("size", "int", vec![], true)]);
        assert!(template_has_default_ctor(&tc));
    }

    #[test]
    fn default_ctor_absent_when_only_parameterized_ctor() {
        // 仅有带参构造函数 → 无默认构造函数。
        let mut ctor = method("Stack", "", vec![("n", "int")], false);
        ctor.is_constructor = true;
        let tc = template_class(
            "Stack",
            vec!["T"],
            vec![ctor, method("size", "int", vec![], true)],
        );
        assert!(!template_has_default_ctor(&tc));
    }

    #[test]
    fn default_ctor_absent_for_abstract_class() {
        let mut tc = template_class("Stack", vec!["T"], vec![method("size", "int", vec![], true)]);
        tc.is_abstract = true;
        assert!(!template_has_default_ctor(&tc));
    }

    #[test]
    fn default_ctor_absent_for_non_public_ctor() {
        let mut ctor = method("Stack", "", vec![], false);
        ctor.is_constructor = true;
        ctor.accessibility = "private".to_string();
        let tc = template_class(
            "Stack",
            vec!["T"],
            vec![ctor, method("size", "int", vec![], true)],
        );
        assert!(!template_has_default_ctor(&tc));
    }
}
