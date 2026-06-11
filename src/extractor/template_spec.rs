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
use crate::ffi_model::{
    TemplateClassSpec, TemplateFactorySpec, TemplateFnSpec, TemplateInstanceSpec,
};

use super::class_spec::build_method_binding;
use super::type_mapper::{clean_type, cpp_to_rust, to_snake_case};
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
/// 实例化追踪策略：依据当前编译单元中「以具体类型实例化某个本文件声明的模板类」的
/// 使用点收集，覆盖以下来源：
///
/// 1. 类（含包装类）的**字段类型**，如 `Stack<int> impl;`；
/// 2. 类方法的**参数 / 返回类型**，如 `void use(Stack<int>& s)`（v6 Phase B 增强（续））；
/// 3. 全局函数的**参数 / 返回类型**（v6 Phase B 增强（续））；
/// 4. **显式实例化** `template class Foo<int>;`（v6 Phase B 增强（再续））——
///    libclang 将其表现为带模板实参的 `ClassDecl`，实参由 `ClassInfo::template_args` 携带。
///
/// 以 `(模板名, 具体类型实参)` 记录并去重。这覆盖了 v6 §3.2 中 025
/// （`Stack<int>`/`Stack<double>`）、027（`Matrix<int>`/`Matrix<double>` 显式实例化）等写法。
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

    // 来源 1：当前编译单元中类的字段类型（如 `Stack<int> impl;`）
    // 来源 2（v6 Phase B 增强（续））：方法的参数类型与返回类型（如 `void use(Stack<int>& s)`）
    for class in ast.classes.iter().filter(|c| c.is_from_current_file) {
        for field in &class.fields {
            collect_instance_from_type(&field.type_name, &template_names, &mut seen, &mut out);
        }
        for method in &class.methods {
            collect_instance_from_type(&method.return_type, &template_names, &mut seen, &mut out);
            for p in &method.params {
                collect_instance_from_type(&p.type_name, &template_names, &mut seen, &mut out);
            }
        }
    }

    // 来源 3（v6 Phase B 增强（续））：当前编译单元中全局函数的参数 / 返回类型
    for func in ast.functions.iter().filter(|f| f.is_from_current_file) {
        collect_instance_from_type(&func.return_type, &template_names, &mut seen, &mut out);
        for p in &func.params {
            collect_instance_from_type(&p.type_name, &template_names, &mut seen, &mut out);
        }
    }

    // 来源 4（v6 Phase B 增强（再续））：显式实例化 `template class Foo<int>;`。
    // libclang 将其表现为带模板实参（`template_args` 非空）的 `ClassDecl`，名称与模板类
    // 同名（如 `Stack`）。这里直接以已拆分好的具体类型实参记录实例化，无需再解析类型字符串。
    for class in ast.classes.iter().filter(|c| c.is_from_current_file) {
        if class.template_args.is_empty() || !template_names.contains(class.name.as_str()) {
            continue;
        }
        record_instance(&class.name, &class.template_args, &mut seen, &mut out);
    }

    out
}

/// 将一个 `(模板名, 具体类型实参列表)` 记录为实例化别名规格，按 `(名, 实参)` 在
/// **本次 `build_template_instances` 调用范围内**去重（通过调用方传入的 `seen` 集合，
/// 跨多个追踪来源共享；不跨编译单元或多次调用）。
fn record_instance(
    name: &str,
    args: &[String],
    seen: &mut std::collections::BTreeSet<(String, Vec<String>)>,
    out: &mut Vec<TemplateInstanceSpec>,
) {
    let key = (name.to_string(), args.to_vec());
    if !seen.insert(key) {
        return;
    }
    if let Some(spec) = build_instance_spec(name, args) {
        out.push(spec);
    }
}

/// 从一个类型字符串中识别「本文件模板类的实例化使用点」，去重后追加到 `out`。
///
/// 剥离指针 / 引用 / cv 限定后匹配 `Name<args>` 形式，且 `Name` 须属于
/// `template_names`（本文件声明的模板类）。相同 `(模板名, 实参列表)` 只记录一次。
fn collect_instance_from_type(
    type_name: &str,
    template_names: &std::collections::BTreeSet<&str>,
    seen: &mut std::collections::BTreeSet<(String, Vec<String>)>,
    out: &mut Vec<TemplateInstanceSpec>,
) {
    let core = strip_type_decorations(type_name);
    let Some((name, args)) = split_template_use(core) else {
        return;
    };
    if !template_names.contains(name.as_str()) {
        return;
    }
    record_instance(&name, &args, seen, out);
}

/// 由模板类构造函数与实例化别名派生构造工厂骨架（v6 Phase B 增强（续））。
///
/// 对每个 [`TemplateInstanceSpec`]，在其对应的本文件模板类中查找公有构造函数，
/// 将类型参数 `T` 替换为该实例化的具体 C++ 类型，生成 `import_lib!` 工厂骨架：
///
/// ```text
/// #[cpp(func = "Stack<int>* stack_i32_new(int initial)")]
/// pub unsafe fn stack_i32_new(initial: i32) -> StackI32;
/// ```
///
/// 工厂对应的 C++ 符号通常需用户在 C++ 侧显式实例化 / 包装后才存在，因此生成器会附
/// `cpp2rust-todo[TMPL]` 提示，需用户结合实际符号补全（符合 v6 方案 §8 降级策略）。
/// 实参个数与模板类型参数个数不一致的实例化会被跳过。
pub(super) fn build_template_factories(
    ast: &CppAst,
    instances: &[TemplateInstanceSpec],
) -> Vec<TemplateFactorySpec> {
    let mut out = Vec::new();

    for inst in instances {
        let Some(tc) = ast
            .template_classes
            .iter()
            .find(|tc| tc.is_from_current_file && tc.name == inst.template_name)
        else {
            continue;
        };
        // 实参与类型参数须一一对应，否则无法可靠替换
        if tc.type_params.len() != inst.cpp_args.len() {
            continue;
        }

        // 仅纳入公有构造函数（跳过析构 / 静态 / 拷贝赋值等）
        let ctors: Vec<&_> = tc
            .methods
            .iter()
            .filter(|m| m.is_constructor && m.accessibility == "public")
            .collect();
        let multi = ctors.len() > 1;

        for (idx, ctor) in ctors.into_iter().enumerate() {
            let base = format!("{}_new", to_snake_case(&inst.alias_name));
            let rust_name = if multi {
                format!("{}_{}", base, idx)
            } else {
                base
            };

            // C++ 工厂签名参数：将类型参数替换为具体实参后的类型（保留参数名）
            let cpp_param_types: Vec<String> = ctor
                .params
                .iter()
                .map(|p| {
                    let ty = substitute_type_params(
                        &normalize_ptr_spacing(clean_type(&p.type_name)),
                        &tc.type_params,
                        &inst.cpp_args,
                    );
                    if !p.name.is_empty() && p.name != "_" {
                        format!("{} {}", ty, p.name)
                    } else {
                        ty
                    }
                })
                .collect();

            // Rust 侧参数：替换类型参数后映射为 Rust 类型
            let params: Vec<(String, String)> = ctor
                .params
                .iter()
                .enumerate()
                .map(|(i, p)| {
                    let sub = substitute_type_params(&p.type_name, &tc.type_params, &inst.cpp_args);
                    (sanitize_param_name(&p.name, i), cpp_to_rust(&sub))
                })
                .collect();

            // 返回类型为实例化类型指针：如 `Stack<int>*`
            let cpp_ret = format!("{}<{}>*", inst.template_name, inst.cpp_args.join(", "));
            let cpp_sig = format!("{} {}({})", cpp_ret, rust_name, cpp_param_types.join(", "));

            out.push(TemplateFactorySpec {
                rust_name,
                alias_name: inst.alias_name.clone(),
                cpp_sig,
                params,
            });
        }
    }

    out
}

/// 将类型字符串中作为独立标识符出现的类型参数（如 `T`）替换为具体实参。
///
/// 例如 `const T&` + (`["T"]`, `["int"]`) → `const int&`；
/// 仅替换完整标识符，不会误伤 `Time`、`vector<T>` 中的子串（`Time` 不等于 `T`）。
fn substitute_type_params(ty: &str, type_params: &[String], cpp_args: &[String]) -> String {
    let mut result = ty.to_string();
    for (param, arg) in type_params.iter().zip(cpp_args.iter()) {
        result = replace_ident(&result, param, arg);
    }
    result
}

/// 以「完整标识符」为单位，将 `from` 替换为 `to`（标识符前后不能是字母/数字/下划线）。
fn replace_ident(haystack: &str, from: &str, to: &str) -> String {
    if from.is_empty() {
        return haystack.to_string();
    }
    let bytes = haystack.as_bytes();
    let mut out = String::with_capacity(haystack.len());
    let mut i = 0usize;
    while i < haystack.len() {
        if haystack[i..].starts_with(from) {
            let before_ok = i == 0 || !is_ident_byte(bytes[i - 1]);
            let after_idx = i + from.len();
            let after_ok = after_idx >= bytes.len() || !is_ident_byte(bytes[after_idx]);
            if before_ok && after_ok {
                out.push_str(to);
                i = after_idx;
                continue;
            }
        }
        // 推进一个 UTF-8 字符
        let ch = haystack[i..].chars().next().unwrap();
        out.push(ch);
        i += ch.len_utf8();
    }
    out
}

fn is_ident_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
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
    let mut cpp_args = Vec::with_capacity(args.len());
    let mut suffix = String::new();
    let mut needs_class_type = false;

    for arg in args {
        let (hicc, suf, is_class) = map_instance_arg(arg);
        hicc_args.push(hicc);
        // 原始 C++ 实参（剥离修饰），用于派生构造工厂的 C++ 签名（如 `Stack<int>`）
        cpp_args.push(strip_type_decorations(arg).to_string());
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
        cpp_args,
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
        assert_eq!(spec.cpp_args, vec!["int".to_string()]);
        assert!(!spec.needs_class_type);
    }

    #[test]
    fn record_instance_dedups_by_name_and_args() {
        use std::collections::BTreeSet;
        let mut seen: BTreeSet<(String, Vec<String>)> = BTreeSet::new();
        let mut out = Vec::new();
        // 显式实例化路径以已拆分的实参直接记录（如 `template class Stack<long>;`）
        record_instance("Stack", &["long".to_string()], &mut seen, &mut out);
        // 同一 (模板名, 实参) 再次出现（例如同时来自字段类型）应被去重
        record_instance("Stack", &["long".to_string()], &mut seen, &mut out);
        assert_eq!(out.len(), 1);
        // `long` 的位宽随平台而异：LP64（Linux/macOS）映射为 i64，
        // LLP64（Windows）映射为 i32，故别名后缀须与平台保持一致。
        #[cfg(not(target_os = "windows"))]
        let expected_alias = "StackI64";
        #[cfg(target_os = "windows")]
        let expected_alias = "StackI32";
        assert_eq!(out[0].alias_name, expected_alias);
        assert_eq!(out[0].cpp_args, vec!["long".to_string()]);
    }

    #[test]
    fn pascal_case_ident_capitalizes_segments() {
        assert_eq!(pascal_case_ident("i32"), "I32");
        assert_eq!(pascal_case_ident("std::string"), "StdString");
        assert_eq!(pascal_case_ident("unsigned int"), "UnsignedInt");
    }

    #[test]
    fn replace_ident_only_matches_whole_identifiers() {
        // 完整标识符被替换
        assert_eq!(replace_ident("T", "T", "int"), "int");
        assert_eq!(replace_ident("const T&", "T", "int"), "const int&");
        assert_eq!(replace_ident("T*", "T", "double"), "double*");
        // 子串不被误替换（Time / TT 不是 T）
        assert_eq!(replace_ident("Time", "T", "int"), "Time");
        assert_eq!(replace_ident("TT", "T", "int"), "TT");
        assert_eq!(replace_ident("vector_T", "T", "int"), "vector_T");
    }

    #[test]
    fn substitute_type_params_replaces_each_param() {
        let params = vec!["T".to_string(), "U".to_string()];
        let args = vec!["int".to_string(), "double".to_string()];
        assert_eq!(
            substitute_type_params("const T&", &params, &args),
            "const int&"
        );
        assert_eq!(substitute_type_params("U*", &params, &args), "double*");
        // 未出现的标识符保持不变
        assert_eq!(substitute_type_params("char", &params, &args), "char");
    }
}
