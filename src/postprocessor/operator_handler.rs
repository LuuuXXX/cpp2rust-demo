//! 运算符重载处理器（Phase 2）
//!
//! 将 C++ 运算符重载对应的 MethodAccessor 转换为 const-ptr shim 函数，
//! 并将有类类型参数的方法从 `import_class!` 中移除。

use crate::ast_parser::{CppAst, FunctionInfo};
use crate::extractor::type_mapper::{clean_type, cpp_to_rust_ffi, to_snake_case};
use crate::ffi_model::{FfiSpec, FnBinding};

/// 支持的二元运算符名称及其 C++ 符号
const BINARY_OPS: &[(&str, &str)] = &[
    ("add", "+"),
    ("sub", "-"),
    ("mul", "*"),
    ("div", "/"),
];

/// 支持的一元运算符名称
const UNARY_OPS: &[&str] = &["negate"];

/// 对所有类应用运算符 shim 生成。
pub fn apply(spec: &mut FfiSpec, ast: &CppAst, functions: &[&FunctionInfo]) {
    let class_names: Vec<&str> = ast.classes.iter().map(|c| c.name.as_str()).collect();

    for ci in &ast.classes {
        let cn_lower = to_snake_case(&ci.name);
        let prefix = format!("{}_", cn_lower);

        // 收集该类的所有 MethodAccessors（第一个参数是该类的指针且参数名为 self/this/thiz）
        let accessors: Vec<&FunctionInfo> = functions
            .iter()
            .filter(|fi| is_class_accessor(&fi.name, &prefix, fi, &ci.name, &class_names))
            .copied()
            .collect();

        if accessors.is_empty() {
            continue;
        }

        // 检查是否有任何运算符相关 accessor（必须验证完整签名）
        let has_ops = accessors.iter().any(|fi| {
            let stripped = match fi.name.strip_prefix(&prefix) {
                Some(s) => s,
                None => return false,
            };
            let extra_params = &fi.params[1..];
            let ret_is_class = fi.return_type.contains(ci.name.as_str());
            let ret_is_void = fi.return_type.is_empty() || fi.return_type == "void";

            // 二元运算符：名称匹配 + 1 个额外参数 + 返回类类型
            let is_binary = BINARY_OPS.iter().any(|(n, _)| *n == stripped)
                && extra_params.len() == 1
                && ret_is_class;
            // 一元运算符：名称匹配 + 0 个额外参数 + 返回类类型
            let is_unary =
                UNARY_OPS.contains(&stripped) && extra_params.is_empty() && ret_is_class;
            // 比较方法：1 个额外类类型参数 + 返回基础类型
            let is_compare = !ret_is_class && !ret_is_void && is_compare_accessor(fi, &ci.name);

            is_binary || is_unary || is_compare
        });
        if !has_ops {
            continue;
        }

        let mut cpp_shims: Vec<String> = Vec::new();
        let mut new_bindings: Vec<FnBinding> = Vec::new();

        // 单次遍历 accessors，依次匹配 Getter / 二元运算符 / 一元运算符 / 比较方法
        for fi in &accessors {
            let stripped = match fi.name.strip_prefix(&prefix) {
                Some(s) => s,
                None => continue,
            };
            let extra_params = &fi.params[1..];
            let ret_is_class = fi.return_type.contains(&ci.name);
            let ret_is_void = fi.return_type.is_empty() || fi.return_type == "void";
            let shim_fn_name = to_snake_case(&fi.name);

            // 1. Getter（0 个额外参数，基础类型返回值）
            if extra_params.is_empty() && !ret_is_class && !ret_is_void {
                let ret_cpp = clean_type(&fi.return_type).to_string();
                let ret_rust = cpp_to_rust_ffi(&fi.return_type);

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
                });
                continue;
            }

            // 2. 二元运算符（1 个额外参数，返回该类指针）
            if extra_params.len() == 1 && ret_is_class {
                if let Some((_, op_sym)) = BINARY_OPS.iter().find(|(n, _)| *n == stripped) {
                    cpp_shims.push(format!(
                        "{}* {}(const {}* a, const {}* b) {{",
                        ci.name, shim_fn_name, ci.name, ci.name
                    ));
                    cpp_shims.push(format!(
                        "    return new {}(*a {} *b);",
                        ci.name, op_sym
                    ));
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
                    });
                }
                continue;
            }

            // 3. 一元运算符（0 个额外参数，返回该类指针，名称在 UNARY_OPS 中）
            if extra_params.is_empty() && ret_is_class && UNARY_OPS.contains(&stripped) {
                let body = match stripped {
                    "negate" => format!("return new {}(-*a);", ci.name),
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
                });
                continue;
            }

            // 4. 比较类方法（1 个额外类类型参数，返回基础类型）
            if extra_params.len() == 1
                && !ret_is_class
                && !ret_is_void
                && !BINARY_OPS.iter().any(|(n, _)| *n == stripped)
                && is_compare_accessor(fi, &ci.name)
            {
                let ret_cpp = clean_type(&fi.return_type).to_string();
                let ret_rust = cpp_to_rust_ffi(&fi.return_type);

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
                });
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

/// 判断函数是否是指定类的 MethodAccessor
fn is_class_accessor(
    fn_name: &str,
    prefix: &str,
    fi: &FunctionInfo,
    _class_name: &str,
    class_names: &[&str],
) -> bool {
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
