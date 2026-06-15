//! `import_lib!` 块构建（Phase 3）
//!
//! 从 `FunctionInfo` 生成 hicc 的 `import_lib! { ... }` 块对应的 `LibSpec`，
//! 并将 ctor/dtor/factory 函数分配到对应 `ClassSpec::associated_fns`。

use super::type_mapper::{clean_type, cpp_to_rust};
use super::{
    classify_functions, is_mappable_rust_type, normalize_ptr_spacing, ret_type_from_cpp,
    sanitize_fn_name, sanitize_param_name, strip_struct_class_keyword, ShimKind,
};
use crate::ast_parser::FunctionInfo;
use crate::ffi_model::{FnBinding, LibSpec};

pub(super) fn build_lib_spec(
    functions: &[&FunctionInfo],
    unit_name: &str,
    class_names: &[&str],
) -> LibSpec {
    let shims = classify_functions(functions, class_names);
    let mut fn_bindings: Vec<FnBinding> = shims
        .iter()
        .filter(|(_, k)| !matches!(k, ShimKind::MethodAccessor))
        .filter(|(fi, _)| !fi.is_variadic)
        .filter(|(fi, _)| !fi.name.starts_with("operator"))
        // C++ 成员函数指针（如 `int (Cls::*)() const`）无法映射为有效 Rust FFI 类型，跳过整个函数
        .filter(|(fi, _)| !fi.params.iter().any(|p| p.type_name.contains("::*)")))
        // 返回类型含 C++ 成员函数指针语法，同样无法映射为有效 Rust FFI 类型，跳过整个函数
        .filter(|(fi, _)| !fi.return_type.contains("::*)"))
        // 参数或返回类型经 cpp_to_rust 映射后仍是无法在 Rust FFI 中使用的类型
        // （如未声明的 C 类型 FILE、未知 C++ 类型 MessageMap、含命名空间的 std::string 等），
        // 跳过整个函数以避免生成无法编译的绑定代码
        .filter(|(fi, _)| {
            fi.params
                .iter()
                .all(|p| is_mappable_rust_type(&cpp_to_rust(&p.type_name), class_names))
                && is_mappable_rust_type(&cpp_to_rust(&fi.return_type), class_names)
        })
        .map(|(fi, _)| build_fn_binding(fi, class_names))
        .collect();

    // 先按 cpp_sig 去重：同一 C++ 签名可能因 "struct T*" 与 "T*" 的差异
    // 经 build_fn_binding 规范化后生成相同的 cpp_sig（如 friend 声明与 extern-C 声明）。
    // 保留首次出现的版本，避免后续误将其视为重载而追加数字后缀。
    {
        let mut seen_sigs: std::collections::HashSet<String> = std::collections::HashSet::new();
        fn_bindings.retain(|fb| seen_sigs.insert(fb.cpp_sig.clone()));
    }

    // 对生成结果中 rust_name 相同的函数（C++ 重载）添加数字后缀（_1, _2, ...）
    // 以确保生成的 Rust 绑定名称不重复
    {
        // 统计每个 rust_name 出现的次数
        let mut name_counts: std::collections::HashMap<String, usize> =
            std::collections::HashMap::new();
        for fb in &fn_bindings {
            *name_counts.entry(fb.rust_name.clone()).or_insert(0) += 1;
        }
        // 对出现多次的 rust_name，从第二次开始追加 _1、_2 … 后缀
        let mut seen_counts: std::collections::HashMap<String, usize> =
            std::collections::HashMap::new();
        for fb in &mut fn_bindings {
            if name_counts.get(&fb.rust_name).copied().unwrap_or(0) > 1 {
                let idx = seen_counts.entry(fb.rust_name.clone()).or_insert(0);
                if *idx > 0 {
                    fb.rust_name = format!("{}_{}", fb.rust_name, *idx);
                }
                *idx += 1;
            }
        }
    }

    // 前向声明：只包含在函数签名中实际引用的类（按原始顺序）
    let used_classes: std::collections::HashSet<&str> = fn_bindings
        .iter()
        .flat_map(|fb| {
            class_names.iter().filter(move |cn| {
                fb.cpp_sig.contains(*cn)
                    || fb.params.iter().any(|(_, t)| t.contains(*cn))
                    || fb
                        .ret_type
                        .as_ref()
                        .map(|r| r.contains(*cn))
                        .unwrap_or(false)
            })
        })
        .copied()
        .collect();
    let fwd_decls: Vec<String> = class_names
        .iter()
        .filter(|cn| used_classes.contains(**cn))
        .map(|n| format!("class {};", n))
        .collect();

    // link_name 只取路径末段（文件名），避免将模块路径（如 "unittest/documenttest"）
    // 直接用作链接库名导致 hicc-build 无法找到对应的编译产物。
    // 使用 std::path::Path::file_name() 而非手动拆分 '/'，跨平台更安全。
    let link_name = std::path::Path::new(unit_name)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or(unit_name)
        .to_string();

    LibSpec {
        link_name,
        fwd_decls,
        fn_bindings,
    }
}

pub(crate) fn build_fn_binding(fi: &FunctionInfo, class_names: &[&str]) -> FnBinding {
    let rust_name = sanitize_fn_name(&fi.name);
    let params: Vec<(String, String)> = fi
        .params
        .iter()
        .enumerate()
        .map(|(i, p)| (sanitize_param_name(&p.name, i), cpp_to_rust(&p.type_name)))
        .collect();

    let ret_type = ret_type_from_cpp(&fi.return_type);

    // unsafe: 参数中有裸指针（*mut T 或 *const i8），或返回值为裸 C 字符串
    // 例外：*mut ClassType 且返回值是原始类型（i8/u8/i16/u16/i32/u32/i64/u64/f32/f64/bool/isize/usize）
    //        且参数不含 volatile 限定 → 不标记 unsafe
    let primitive_ret = ret_type
        .as_deref()
        .map(|r| {
            matches!(
                r,
                "i8" | "u8"
                    | "i16"
                    | "u16"
                    | "i32"
                    | "u32"
                    | "i64"
                    | "u64"
                    | "f32"
                    | "f64"
                    | "bool"
                    | "isize"
                    | "usize"
            )
        })
        .unwrap_or(false);
    let has_volatile_param = fi
        .params
        .iter()
        .any(|p| p.type_name.split_whitespace().any(|w| w == "volatile"));
    let is_unsafe = params.iter().any(|(_, t)| {
        if t == "*const i8" {
            return true;
        }
        if t.starts_with("unsafe extern") {
            return true; // C 函数指针参数：需要 unsafe
        }
        if let Some(inner) = t.strip_prefix("*mut ") {
            let is_class = class_names.contains(&inner);
            // volatile 限定的类指针参数不能享受 primitive_ret 豁免：仍标记为 unsafe
            if is_class && primitive_ret && !has_volatile_param {
                return false;
            }
            return true;
        }
        false
    }) || ret_type
        .as_deref()
        .is_some_and(|r| r == "*const i8" || r == "*mut i8" || r.starts_with("unsafe extern"));

    // 检测参数或返回类型是否含 C 函数指针，用于生成 cpp2rust-todo[FP] 注释
    let has_fn_ptr_param =
        fi.params.iter().any(|p| p.type_name.contains("(*)")) || fi.return_type.contains("(*)");

    // 构造 C++ 函数签名：只有当参数类型为已知类的指针时才保留参数名，
    // 但 self/this/thiz 等接收者惯用名除外（这些参数在 C 签名中通常省略参数名）
    let param_parts: Vec<String> = fi
        .params
        .iter()
        .map(|p| {
            let ty_stripped = strip_struct_class_keyword(clean_type(&p.type_name));
            let ty = normalize_ptr_spacing(&ty_stripped);
            let is_class_ptr = class_names.iter().any(|cn| p.type_name.contains(cn));
            let is_self_name = matches!(p.name.as_str(), "self" | "this" | "thiz");
            // 函数指针类型（如 void (*)(T*)）无法在末尾追加参数名（非法 C++ 语法），跳过追名
            let is_fn_ptr = ty.contains("(*)");
            if is_class_ptr && !p.name.is_empty() && p.name != "_" && !is_self_name && !is_fn_ptr {
                format!("{} {}", ty, p.name)
            } else {
                ty
            }
        })
        .collect();

    let ret_clean = if fi.return_type.is_empty() || fi.return_type == "void" {
        "void".to_string()
    } else {
        let ret_stripped = strip_struct_class_keyword(clean_type(&fi.return_type));
        normalize_ptr_spacing(&ret_stripped)
    };

    // 无参数时统一输出空参数列表 "()"，与 C++ 风格一致。
    // 无论 extern_c 与否，hicc 对 C++ 签名均接受 "()" 写法。
    let params_str = if param_parts.is_empty() {
        String::new()
    } else {
        param_parts.join(", ")
    };

    let cpp_sig = format!("{} {}({})", ret_clean, fi.name, params_str);

    FnBinding {
        cpp_sig,
        rust_name,
        params,
        ret_type,
        is_unsafe,
        has_fn_ptr_param,
    }
}
