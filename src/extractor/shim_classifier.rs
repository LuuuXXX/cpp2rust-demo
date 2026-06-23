//! Shim 分类器（Phase 3 辅助）
//!
//! 将 C 桥接函数（shim）分类为 Ctor/Dtor/MethodAccessor/StaticAccessor/Standalone，
//! 并负责将属于某个类的构造/析构/静态访问函数归属到对应 ClassSpec。

use super::type_mapper::to_snake_case;
use super::type_references_class;
use crate::ast_parser::FunctionInfo;
use crate::ffi_model::{ClassSpec, LibSpec};

/// Shim 函数分类。
#[derive(Debug, PartialEq)]
pub(crate) enum ShimKind {
    /// 构造函数（`foo_new` / `foo_new_variant` 等），返回类指针。
    Ctor,
    /// 析构函数（`foo_delete` / `foo_free` 等），第一个参数为类指针。
    Dtor,
    /// 方法访问器：第一个参数为类指针且参数名为 `self`/`this`/`thiz`。
    MethodAccessor,
    /// 独立函数（不属于任何类的 shim）。
    Standalone,
    /// 静态访问器：函数名以类名小写前缀开头，且无 self 类指针参数。
    StaticAccessor,
}

/// 批量分类 shim 函数列表。
pub(super) fn classify_functions<'a>(
    functions: &[&'a FunctionInfo],
    class_names: &[&str],
) -> Vec<(&'a FunctionInfo, ShimKind)> {
    functions
        .iter()
        .map(|fi| (*fi, classify_fn(fi, class_names)))
        .collect()
}

/// 对单个函数进行 shim 分类。
pub(super) fn classify_fn(fi: &FunctionInfo, class_names: &[&str]) -> ShimKind {
    let name_lower = fi.name.to_lowercase();

    let ret_is_class_ptr = class_names
        .iter()
        .any(|cn| type_references_class(&fi.return_type, cn));

    let first_param_is_class_ptr = fi
        .params
        .first()
        .map(|p| {
            class_names
                .iter()
                .any(|cn| type_references_class(&p.type_name, cn))
        })
        .unwrap_or(false);

    // 识别构造函数命名模式（使用原始大小写以正确处理驼峰变体）：
    //   foo_new          — ends_with("_new")
    //   foo_new_variant  — contains("_new_")
    //   foo_newCamelCase — _new 后紧跟大写字母（驼峰，如 foo_newWithSize）
    let name_has_new = fi.name == "new"
        || fi.name.ends_with("_new")
        || fi.name.contains("_new_")
        || fi.name.find("_new").is_some_and(|p| {
            fi.name
                .get(p + 4..)
                .is_some_and(|rest| rest.starts_with(|c: char| c.is_uppercase()))
        });

    if ret_is_class_ptr && name_has_new {
        return ShimKind::Ctor;
    }
    if first_param_is_class_ptr
        && (name_lower.ends_with("_delete")
            || name_lower.ends_with("_deleter")
            || name_lower == "delete"
            || name_lower.ends_with("_free")
            || name_lower == "free"
            || name_lower.ends_with("_destroy")
            || name_lower == "destroy"
            || name_lower.ends_with("_release")
            || name_lower == "release")
    {
        return ShimKind::Dtor;
    }
    // 只有当第一个参数是类指针且参数名为约定的 self/this/thiz（表示对象接收者）时，
    // 才归类为 MethodAccessor（会被跳过，不出现在 import_lib/import_class 中）。
    // 若第一个参数名是其他名称（如 other/src/input），则该参数只是普通的类指针参数，
    // 函数应归类为 Standalone，出现在 import_lib 中。
    let first_param_name_is_self = fi
        .params
        .first()
        .map(|p| matches!(p.name.as_str(), "self" | "this" | "thiz"))
        .unwrap_or(false);
    // volatile 限定的指针参数无法作为 hicc 类方法接收者，应归为 Standalone
    let first_param_is_volatile = fi
        .params
        .first()
        .map(|p| p.type_name.split_whitespace().any(|w| w == "volatile"))
        .unwrap_or(false);
    if first_param_is_class_ptr && first_param_name_is_self && !first_param_is_volatile {
        return ShimKind::MethodAccessor;
    }

    let is_static_accessor = class_names.iter().any(|cn| {
        let prefix = format!("{}_", cn.to_lowercase());
        name_lower.starts_with(&prefix)
    }) && !first_param_is_class_ptr;

    if is_static_accessor {
        ShimKind::StaticAccessor
    } else {
        ShimKind::Standalone
    }
}

/// 将 LibSpec::fn_bindings 中属于某个类的 ctor/dtor/StaticAccessor 函数
/// 移至对应 ClassSpec::associated_fns，使代码生成器可输出 class body 格式。
///
/// 匹配规则：函数名前缀与类名匹配（如 `counter_new` → 归属 `Counter`）；
/// 仅处理 `ShimKind::Ctor`、`ShimKind::Dtor`、`ShimKind::StaticAccessor`。
/// 不属于任何已知类（或类无对应 ClassSpec）的函数保留在 fn_bindings 中。
pub(super) fn assign_associated_fns(
    class_specs: &mut [ClassSpec],
    lib_spec: &mut LibSpec,
    functions: &[&FunctionInfo],
    class_names: &[&str],
) {
    // 预先分类所有 shim 函数，同时建立两张表（单次遍历）
    let shims = classify_functions(functions, class_names);

    // 同时构建 rust_name → ShimKind 与 rust_name → FunctionInfo 两张映射
    let mut kind_map: std::collections::HashMap<String, &ShimKind> =
        std::collections::HashMap::new();
    let mut fn_by_rust_name: std::collections::HashMap<String, &FunctionInfo> =
        std::collections::HashMap::new();
    for (fi, kind) in &shims {
        let rust_name = to_snake_case(&fi.name);
        kind_map.entry(rust_name.clone()).or_insert(kind);
        fn_by_rust_name.entry(rust_name).or_insert(fi);
    }

    let mut remaining = Vec::new();
    for fb in lib_spec.fn_bindings.drain(..) {
        let kind = kind_map.get(&fb.rust_name).copied();
        let should_move = matches!(
            kind,
            Some(ShimKind::Ctor | ShimKind::Dtor | ShimKind::StaticAccessor)
        );

        if should_move {
            // 通过函数签名中的类型（返回类型 / 第一个参数类型）确定归属类。
            // 这比名称前缀匹配更可靠，可正确处理 RapidJsonBigIntegerHandle 这类
            // 类名与函数名前缀不一致的情况。
            let matching_function = fn_by_rust_name.get(&fb.rust_name).copied();
            let owning: Option<&str> = matching_function.and_then(|fi| {
                if matches!(kind, Some(ShimKind::Ctor)) {
                    // Ctor：返回类型中含类名（优先最长匹配，避免子串误匹配）
                    class_names
                        .iter()
                        .filter(|cn| fi.return_type.contains(*cn))
                        .max_by_key(|cn| cn.len())
                        .copied()
                } else if matches!(kind, Some(ShimKind::Dtor)) {
                    // Dtor：第一个参数类型含类名（优先最长匹配，避免子串误匹配）
                    fi.params.first().and_then(|p| {
                        class_names
                            .iter()
                            .filter(|cn| p.type_name.contains(*cn))
                            .max_by_key(|cn| cn.len())
                            .copied()
                    })
                } else {
                    // StaticAccessor：退回名称前缀匹配
                    class_names
                        .iter()
                        .filter(|cn| {
                            let prefix = format!("{}_", cn.to_lowercase());
                            fb.rust_name.starts_with(&prefix)
                        })
                        .max_by_key(|cn| cn.len())
                        .copied()
                }
            });

            if let Some(cn) = owning {
                if let Some(cs) = class_specs.iter_mut().find(|c| c.name == cn) {
                    // Dtor：记录 destroy_fn 名称（不放入 associated_fns，dtor 不在 Rust 端显式调用）
                    if matches!(kind, Some(ShimKind::Dtor)) {
                        cs.destroy_fn = Some(fb.rust_name.clone());
                    } else {
                        cs.associated_fns.push(fb);
                    }
                    continue;
                }
            }
        }
        remaining.push(fb);
    }
    lib_spec.fn_bindings = remaining;

    // 确保有 associated_fns 的类在 fwd_decls 中有前向声明
    for cs in class_specs.iter() {
        if !cs.associated_fns.is_empty() {
            let decl = format!("class {};", cs.name);
            if !lib_spec.fwd_decls.contains(&decl) {
                lib_spec.fwd_decls.push(decl);
            }
        }
    }
}
