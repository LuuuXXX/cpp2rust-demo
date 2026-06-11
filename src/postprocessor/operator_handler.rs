//! 运算符重载处理器（Phase 4）
//!
//! 将 C++ 运算符重载对应的 MethodAccessor 转换为 const-ptr shim 函数，
//! 并将有类类型参数的方法从 `import_class!` 中移除。

use crate::ast_parser::{CppAst, FunctionInfo};
use crate::extractor::type_mapper::{clean_type, cpp_to_rust, to_snake_case};
use crate::ffi_model::{FfiSpec, FnBinding};

/// 支持的二元运算符名称及其 C++ 符号
const BINARY_OPS: &[(&str, &str)] = &[
    ("add", "+"),
    ("sub", "-"),
    ("mul", "*"),
    ("div", "/"),
    ("mod", "%"),
    ("shl", "<<"),
    ("shr", ">>"),
    ("bitand", "&"),
    ("bitor", "|"),
    ("bitxor", "^"),
];

/// 支持的比较运算符名称及其 C++ 符号
const COMPARE_OPS: &[(&str, &str)] = &[
    ("eq", "=="),
    ("ne", "!="),
    ("lt", "<"),
    ("gt", ">"),
    ("le", "<="),
    ("ge", ">="),
];

/// 支持的一元运算符名称
const UNARY_OPS: &[&str] = &["negate", "not", "bitnot", "pre_inc", "pre_dec"];

/// 对所有类应用运算符 shim 生成。
pub fn apply(spec: &mut FfiSpec, ast: &CppAst, functions: &[&FunctionInfo]) {
    let class_names: Vec<&str> = ast.classes.iter().map(|c| c.name.as_str()).collect();

    for ci in &ast.classes {
        let cn_lower = to_snake_case(&ci.name);
        let prefix = format!("{}_", cn_lower);

        // 收集该类的所有 MethodAccessors（第一个参数是该类的指针且参数名为 self/this/thiz）
        let mut accessors: Vec<&FunctionInfo> = functions
            .iter()
            .filter(|fi| is_class_accessor(&fi.name, &prefix, fi, &class_names))
            .copied()
            .collect();

        if accessors.is_empty() {
            continue;
        }

        // 检查是否有任何运算符相关 accessor（不含 Getter：getter-only 类不应触发 shim 生成）
        let has_ops = accessors.iter().any(|fi| {
            matches!(
                classify_accessor(fi, &prefix, &ci.name),
                AccessorKind::BinaryOp | AccessorKind::UnaryOp | AccessorKind::Compare
            )
        });
        if !has_ops {
            continue;
        }

        let mut cpp_shims: Vec<String> = Vec::new();
        let mut new_bindings: Vec<FnBinding> = Vec::new();

        // 按类别排序 accessors：Getter(0) → 二元运算符(1) → 一元运算符(2) → 比较方法(3)
        // 保证生成顺序与逻辑分类一致，不受头文件声明顺序影响
        accessors.sort_by_key(|fi| accessor_category(fi, &prefix, &ci.name));

        // 单次遍历 accessors，依次匹配 Getter / 二元运算符 / 一元运算符 / 比较方法
        for fi in &accessors {
            let stripped = match fi.name.strip_prefix(&prefix) {
                Some(s) => s,
                None => continue,
            };
            let shim_fn_name = to_snake_case(&fi.name);

            match classify_accessor(fi, &prefix, &ci.name) {
                AccessorKind::Getter => {
                    let ret_cpp = clean_type(&fi.return_type).to_string();
                    let ret_rust = cpp_to_rust(&fi.return_type);

                    cpp_shims.push(format!(
                        "{} {}(const {}* self) {{",
                        ret_cpp, shim_fn_name, ci.name
                    ));
                    cpp_shims.push(format!("    return self->{}();", stripped));
                    cpp_shims.push("}".to_string());
                    cpp_shims.push(String::new());

                    new_bindings.push(FnBinding {
                        cpp_sig: format!("{} {}(const {}*)", ret_cpp, shim_fn_name, ci.name),
                        rust_name: fi.name.clone(),
                        params: vec![("self_".to_string(), format!("*const {}", ci.name))],
                        ret_type: Some(ret_rust),
                        is_unsafe: false,
                        has_fn_ptr_param: false,
                    });
                }
                AccessorKind::BinaryOp => {
                    if let Some((_, op_sym)) = BINARY_OPS.iter().find(|(n, _)| *n == stripped) {
                        cpp_shims.push(format!(
                            "{}* {}(const {}* a, const {}* b) {{",
                            ci.name, shim_fn_name, ci.name, ci.name
                        ));
                        cpp_shims.push(format!("    return new {}(*a {} *b);", ci.name, op_sym));
                        cpp_shims.push("}".to_string());
                        cpp_shims.push(String::new());

                        new_bindings.push(FnBinding {
                            cpp_sig: format!(
                                "{}* {}(const {}*, const {}*)",
                                ci.name, shim_fn_name, ci.name, ci.name
                            ),
                            rust_name: fi.name.clone(),
                            params: vec![
                                ("a".to_string(), format!("*const {}", ci.name)),
                                ("b".to_string(), format!("*const {}", ci.name)),
                            ],
                            ret_type: Some(format!("*mut {}", ci.name)),
                            is_unsafe: false,
                            has_fn_ptr_param: false,
                        });
                    }
                }
                AccessorKind::UnaryOp => {
                    let body = match stripped {
                        "negate" => format!("return new {}(-*a);", ci.name),
                        "not" => format!("return new {}(!*a);", ci.name),
                        "bitnot" => format!("return new {}(~*a);", ci.name),
                        "pre_inc" => format!("auto tmp = *a; ++tmp; return new {}(tmp);", ci.name),
                        "pre_dec" => format!("auto tmp = *a; --tmp; return new {}(tmp);", ci.name),
                        _ => continue,
                    };

                    cpp_shims.push(format!(
                        "{}* {}(const {}* a) {{",
                        ci.name, shim_fn_name, ci.name
                    ));
                    cpp_shims.push(format!("    {}", body));
                    cpp_shims.push("}".to_string());
                    cpp_shims.push(String::new());

                    new_bindings.push(FnBinding {
                        cpp_sig: format!("{}* {}(const {}*)", ci.name, shim_fn_name, ci.name),
                        rust_name: fi.name.clone(),
                        params: vec![("a".to_string(), format!("*const {}", ci.name))],
                        ret_type: Some(format!("*mut {}", ci.name)),
                        is_unsafe: false,
                        has_fn_ptr_param: false,
                    });
                }
                AccessorKind::Compare => {
                    // 比较运算符生成 bool shim（使用 COMPARE_OPS 中的运算符符号）
                    if let Some((_, op_sym)) = COMPARE_OPS.iter().find(|(n, _)| *n == stripped) {
                        cpp_shims.push(format!(
                            "bool {}(const {}* a, const {}* b) {{",
                            shim_fn_name, ci.name, ci.name
                        ));
                        cpp_shims.push(format!("    return *a {} *b;", op_sym));
                        cpp_shims.push("}".to_string());
                        cpp_shims.push(String::new());

                        new_bindings.push(FnBinding {
                            cpp_sig: format!(
                                "bool {}(const {}*, const {}*)",
                                shim_fn_name, ci.name, ci.name
                            ),
                            rust_name: fi.name.clone(),
                            params: vec![
                                ("a".to_string(), format!("*const {}", ci.name)),
                                ("b".to_string(), format!("*const {}", ci.name)),
                            ],
                            ret_type: Some("bool".to_string()),
                            is_unsafe: false,
                            has_fn_ptr_param: false,
                        });
                    } else {
                        // 退回为原始比较方法（基于实际返回类型）
                        let ret_cpp = clean_type(&fi.return_type).to_string();
                        let ret_rust = cpp_to_rust(&fi.return_type);

                        cpp_shims.push(format!(
                            "{} {}(const {}* a, const {}* b) {{",
                            ret_cpp, shim_fn_name, ci.name, ci.name
                        ));
                        cpp_shims.push(format!("    return a->{}(*b);", stripped));
                        cpp_shims.push("}".to_string());
                        cpp_shims.push(String::new());

                        new_bindings.push(FnBinding {
                            cpp_sig: format!(
                                "{} {}(const {}*, const {}*)",
                                ret_cpp, shim_fn_name, ci.name, ci.name
                            ),
                            rust_name: fi.name.clone(),
                            params: vec![
                                ("a".to_string(), format!("*const {}", ci.name)),
                                ("b".to_string(), format!("*const {}", ci.name)),
                            ],
                            ret_type: Some(ret_rust),
                            is_unsafe: false,
                            has_fn_ptr_param: false,
                        });
                    }
                }
                AccessorKind::Other => {}
            }
        }

        if cpp_shims.is_empty() {
            continue;
        }

        // 去掉末尾多余空行
        while cpp_shims.last().map(|l| l.is_empty()).unwrap_or(false) {
            cpp_shims.pop();
        }

        // 追加到 cpp_block_lines 末尾（在 dtor 之后）
        spec.cpp_block_lines.push(String::new());
        spec.cpp_block_lines.extend(cpp_shims);

        // 追加新 binding 到 lib_spec 末尾
        spec.lib_spec.fn_bindings.extend(new_bindings);

        // 从 import_class! 中删除有类类型参数的方法（如 compare）
        if let Some(cs) = spec.class_specs.iter_mut().find(|cs| cs.name == ci.name) {
            cs.methods.retain(|mb| {
                !mb.params
                    .iter()
                    .any(|(_, t)| class_names.iter().any(|cn| t.contains(cn)))
            });
        }
    }
}

/// accessor 的功能类别（用于排序和逻辑分支）
#[derive(Debug, PartialEq, Eq)]
enum AccessorKind {
    Getter,    // 0 个额外参数，基础类型返回值
    BinaryOp,  // 1 个额外参数，返回类类型，名称在 BINARY_OPS 中
    UnaryOp,   // 0 个额外参数，返回类类型，名称在 UNARY_OPS 中
    Compare,   // 1 个额外类类型参数，返回基础类型
    Other,     // 不属于以上任何类别
}

/// 对给定 accessor 函数进行分类，消除 `apply` 和 `accessor_category` 的重复逻辑。
fn classify_accessor(fi: &FunctionInfo, prefix: &str, class_name: &str) -> AccessorKind {
    let stripped = match fi.name.strip_prefix(prefix) {
        Some(s) => s,
        None => return AccessorKind::Other,
    };
    let extra_params = &fi.params[1..];
    let ret_is_class = fi.return_type.contains(class_name);
    let ret_is_void = fi.return_type.is_empty() || fi.return_type == "void";

    if extra_params.is_empty() && !ret_is_class && !ret_is_void {
        return AccessorKind::Getter;
    }
    if BINARY_OPS.iter().any(|(n, _)| *n == stripped) && extra_params.len() == 1 && ret_is_class {
        return AccessorKind::BinaryOp;
    }
    if UNARY_OPS.contains(&stripped) && extra_params.is_empty() && ret_is_class {
        return AccessorKind::UnaryOp;
    }
    // 比较运算符：名称在 COMPARE_OPS 中，且有一个同类型参数，返回非 void 非类类型
    if COMPARE_OPS.iter().any(|(n, _)| *n == stripped)
        && extra_params.len() == 1
        && !ret_is_class
        && !ret_is_void
    {
        return AccessorKind::Compare;
    }
    // 旧式 compare accessor：有一个同类型参数，返回非 void 非类类型
    if extra_params.len() == 1 && !ret_is_class && !ret_is_void && is_compare_accessor(fi, class_name) {
        return AccessorKind::Compare;
    }
    AccessorKind::Other
}

/// 判断函数是否是指定类的 MethodAccessor
fn is_class_accessor(fn_name: &str, prefix: &str, fi: &FunctionInfo, class_names: &[&str]) -> bool {
    if !fn_name.starts_with(prefix) {
        return false;
    }
    let first = match fi.params.first() {
        Some(p) => p,
        None => return false,
    };
    let is_class_ptr = class_names.iter().any(|cn| {
        first.type_name.contains(&format!("{} *", cn))
            || first.type_name.contains(&format!("{}*", cn))
    });
    if !is_class_ptr {
        return false;
    }
    // 第一个参数名必须是 self/this/thiz（MethodAccessor 约定）
    matches!(first.name.as_str(), "self" | "this" | "thiz")
}

/// 判断额外参数是否包含该类的类型（用于识别 compare-style 方法）
fn is_compare_accessor(fi: &FunctionInfo, class_name: &str) -> bool {
    fi.params
        .iter()
        .skip(1)
        .any(|p| p.type_name.contains(class_name))
}

/// 返回 accessor 的类别优先级，用于排序：
/// 0 = Getter, 1 = 二元运算符, 2 = 一元运算符, 3 = 比较方法, 4 = 其他
fn accessor_category(fi: &FunctionInfo, prefix: &str, class_name: &str) -> u8 {
    match classify_accessor(fi, prefix, class_name) {
        AccessorKind::Getter => 0,
        AccessorKind::BinaryOp => 1,
        AccessorKind::UnaryOp => 2,
        AccessorKind::Compare => 3,
        AccessorKind::Other => 4,
    }
}

// ─────────────────────────────────────────────
//  T3: operator_handler 核心逻辑单元测试
// ─────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast_parser::{ClassInfo, CppAst, FunctionInfo, ParamInfo};
    use crate::ffi_model::{FfiSpec, LibSpec, MethodBinding, SelfKind};

    fn make_class(name: &str) -> ClassInfo {
        ClassInfo {
            name: name.to_string(),
            is_struct: false,
            is_abstract: false,
            template_args: vec![],
            bases: vec![],
            methods: vec![],
            fields: vec![],
            is_in_namespace: false,
            is_from_current_file: true,
        }
    }

    fn make_fi(name: &str, return_type: &str, params: &[(&str, &str)]) -> FunctionInfo {
        FunctionInfo {
            name: name.to_string(),
            return_type: return_type.to_string(),
            params: params
                .iter()
                .map(|(n, t)| ParamInfo {
                    name: n.to_string(),
                    type_name: t.to_string(),
                    has_default: false,
                })
                .collect(),
            is_inline: false,
            is_variadic: false,
            is_extern_c: true,
            friend_of: None,
            body_offset: None,
            is_from_current_file: true,
        }
    }

    fn make_spec_with_class(class_name: &str) -> FfiSpec {
        FfiSpec {
            unit_name: "test".to_string(),
            cpp_block_lines: vec![],
            class_specs: vec![crate::ffi_model::ClassSpec {
                name: class_name.to_string(),
                methods: vec![MethodBinding {
                    cpp_sig: format!("int {}_value(const {}* self) const", class_name.to_lowercase(), class_name),
                    rust_name: "value".to_string(),
                    self_kind: SelfKind::Ref,
                    params: vec![],
                    ret_type: Some("i32".to_string()),
                    has_fn_ptr_param: false,
                }],
                associated_fns: vec![],
                destroy_fn: None,
                is_interface: false,
            }],
            lib_spec: LibSpec::default(),
        }
    }

    fn make_ast_with_class(class_name: &str) -> CppAst {
        CppAst {
            file: std::path::PathBuf::from("test.cpp"),
            classes: vec![make_class(class_name)],
            functions: vec![],
            enums: vec![],
            typedefs: vec![],
            template_class_ranges: vec![],
        }
    }

    /// (a) operator_handler 为 binary add 生成正确的命名 shim
    #[test]
    fn apply_generates_binary_add_shim() {
        let cn = "Vec2";
        let prefix = "vec2_";
        // 函数：vec2_add(self: Vec2*, other: Vec2*) -> Vec2*
        let fi = make_fi(
            &format!("{}add", prefix),
            &format!("{} *", cn),
            &[
                ("self", &format!("{} *", cn)),
                ("other", &format!("{} *", cn)),
            ],
        );
        let mut spec = make_spec_with_class(cn);
        let ast = make_ast_with_class(cn);
        let fns = vec![&fi];
        apply(&mut spec, &ast, &fns);

        // 应在 fn_bindings 中找到 add shim
        let found = spec.lib_spec.fn_bindings.iter().any(|fb| fb.rust_name.contains("add"));
        assert!(found, "apply 应生成 vec2_add shim，但 fn_bindings 中没有");

        // 应在 cpp_block_lines 中找到 add 的 C++ shim 代码
        let cpp_has_add = spec
            .cpp_block_lines
            .iter()
            .any(|l| l.contains("vec2_add"));
        assert!(cpp_has_add, "apply 应在 cpp_block_lines 中生成 C++ shim");
    }

    /// (b) operator_handler 为比较运算符 eq 生成 bool shim
    #[test]
    fn apply_generates_compare_eq_shim() {
        let cn = "Num";
        let prefix = "num_";
        // 函数：num_eq(self: Num*, other: Num*) -> bool
        let fi = make_fi(
            &format!("{}eq", prefix),
            "bool",
            &[("self", &format!("{} *", cn)), ("other", &format!("{} *", cn))],
        );
        let mut spec = make_spec_with_class(cn);
        let ast = make_ast_with_class(cn);
        let fns = vec![&fi];
        apply(&mut spec, &ast, &fns);

        let eq_binding = spec.lib_spec.fn_bindings.iter().find(|fb| fb.rust_name.contains("eq"));
        assert!(eq_binding.is_some(), "apply 应生成 num_eq shim");
        // eq shim 的返回类型应为 bool
        let ret = eq_binding.unwrap().ret_type.as_deref().unwrap_or("");
        assert_eq!(ret, "bool", "eq shim 的返回类型应为 bool");
    }

    /// (c) operator_handler 为新增一元运算符 not 生成正确 shim
    #[test]
    fn apply_generates_unary_not_shim() {
        let cn = "Flag";
        let prefix = "flag_";
        // 函数：flag_not(self: Flag*) -> Flag*
        let fi = make_fi(
            &format!("{}not", prefix),
            &format!("{} *", cn),
            &[("self", &format!("{} *", cn))],
        );
        let mut spec = make_spec_with_class(cn);
        let ast = make_ast_with_class(cn);
        let fns = vec![&fi];
        apply(&mut spec, &ast, &fns);

        let not_binding = spec
            .lib_spec
            .fn_bindings
            .iter()
            .find(|fb| fb.rust_name.contains("not"));
        assert!(not_binding.is_some(), "apply 应生成 flag_not shim");
        // 一元 shim 的参数应只有 self（1 个参数）
        assert_eq!(not_binding.unwrap().params.len(), 1, "一元 shim 应只有 1 个参数");
    }
}
