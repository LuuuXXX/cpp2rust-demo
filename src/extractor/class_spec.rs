//! import_class! ブロック構築（Phase 3）
//!
//! ClassInfo から hicc の `import_class! { ... }` ブロックの ClassSpec を生成する。

use super::type_mapper::{clean_type, cpp_to_rust, to_snake_case};
use super::{
    is_mappable_rust_type, is_rust_keyword, normalize_ptr_spacing, ret_type_from_cpp,
    sanitize_fn_name, sanitize_param_name, strip_volatile,
};
use crate::ast_parser::{ClassInfo, MethodInfo};
use crate::ffi_model::{ClassSpec, MethodBinding, SelfKind};

/// 检查 MethodBinding 的所有参数类型和返回类型是否均为合法 Rust FFI 类型。
fn is_method_types_mappable(mb: &MethodBinding, class_names: &[&str]) -> bool {
    mb.params
        .iter()
        .all(|(_, t)| is_mappable_rust_type(t, class_names))
        && mb
            .ret_type
            .as_deref()
            .map(|t| is_mappable_rust_type(t, class_names))
            .unwrap_or(true) // None（void 返回值）始终合法
}

/// `exported_class_names`：实际会生成 `import_class!` 块的类名列表（即 `used_classes`
/// 中的成员）。类型映射合法性检查只认可这些名称，避免将内部实现类（如
/// `xml_memory_page`、`xpath_context`）误判为合法 FFI 类型，从而生成引用未定义类型的代码。
pub(super) fn build_class_spec(
    ci: &ClassInfo,
    all_classes: &[ClassInfo],
    exported_class_names: &[&str],
) -> Option<ClassSpec> {
    // 收集本类的 public 非 ctor/dtor 方法（跳过 operator 重载和 Rust 关键字方法名）
    let own_methods: Vec<&MethodInfo> = ci
        .methods
        .iter()
        .filter(|m| {
            !m.is_constructor && !m.is_destructor && m.accessibility == "public" && !m.is_static
        })
        .filter(|m| !m.name.starts_with("operator") && !is_rust_keyword(&to_snake_case(&m.name)))
        .collect();

    // 收集所有基类的 public 方法（递归，保持顺序）
    let inherited = collect_inherited_methods(ci, all_classes);

    // 合并：继承方法 + 本类覆盖/新增方法
    // 规则：如果本类有同名方法（override），用本类的；否则用继承的
    let own_names: std::collections::HashSet<&str> =
        own_methods.iter().map(|m| m.name.as_str()).collect();

    // 用于类型映射合法性检查：只使用实际会导出的类名（外部传入）
    let class_names = exported_class_names;

    let mut methods: Vec<MethodBinding> = Vec::new();

    // 先放继承来的（本类未覆盖的，同样跳过 operator 和 Rust 关键字方法名）
    for im in &inherited {
        if !own_names.contains(im.name.as_str())
            && !im.name.starts_with("operator")
            && !is_rust_keyword(&to_snake_case(&im.name))
        {
            if let Some(mb) = build_method_binding(im) {
                // 过滤含无法映射为合法 Rust FFI 类型的参数/返回类型的方法，
                // 避免生成无法编译的绑定代码（与 build_lib_spec 的过滤逻辑一致）
                if is_method_types_mappable(&mb, class_names) {
                    methods.push(mb);
                }
            }
        }
    }

    // 再放本类的方法（按原始顺序：覆盖的和新增的）
    for m in &own_methods {
        if let Some(mb) = build_method_binding(m) {
            // 同上：过滤含无法映射类型的方法
            if is_method_types_mappable(&mb, class_names) {
                methods.push(mb);
            }
        }
    }

    // 去重：C++ 支持方法重载（同名不同参），但 hicc::import_class! 不支持同名方法。
    // 对 rust_name 相同的方法只保留第一个（参数最少/最简单的重载），其余重载跳过，
    // 避免生成 "field specified more than once" 编译错误。
    let mut seen_names: std::collections::HashSet<String> = std::collections::HashSet::new();
    let methods: Vec<MethodBinding> = methods
        .into_iter()
        .filter(|mb| seen_names.insert(mb.rust_name.clone()))
        .collect();

    if methods.is_empty() {
        return None;
    }

    // 检测纯虚接口类：所有 public 非 ctor/dtor 方法（含继承）均为纯虚
    let is_interface = !own_methods.is_empty()
        && own_methods.iter().all(|m| m.is_pure_virtual)
        && inherited.iter().all(|m| m.is_pure_virtual);

    Some(ClassSpec {
        name: ci.name.clone(),
        methods,
        associated_fns: Vec::new(),
        destroy_fn: None,
        is_interface,
    })
}

/// 递归收集所有基类的 public 非 ctor/dtor 方法（不含静态方法）
fn collect_inherited_methods<'a>(
    ci: &ClassInfo,
    all_classes: &'a [ClassInfo],
) -> Vec<&'a MethodInfo> {
    let mut result: Vec<&'a MethodInfo> = Vec::new();
    let mut seen: std::collections::HashSet<&str> = std::collections::HashSet::new();

    for base in &ci.bases {
        let base_name = clean_type(&base.name).to_string();
        if let Some(base_ci) = all_classes.iter().find(|c| c.name == base_name) {
            // 先递归收集基类的基类
            let grand_inherited = collect_inherited_methods(base_ci, all_classes);
            for m in grand_inherited {
                if seen.insert(m.name.as_str()) {
                    result.push(m);
                }
            }
            // 再收集本基类的方法
            for m in base_ci.methods.iter().filter(|m| {
                !m.is_constructor && !m.is_destructor && m.accessibility == "public" && !m.is_static
            }) {
                if seen.insert(m.name.as_str()) {
                    result.push(m);
                }
            }
        }
    }
    result
}

pub(super) fn build_method_binding(m: &MethodInfo) -> Option<MethodBinding> {
    // hicc 不支持 volatile this 限定的方法（方法指针类型不匹配），跳过
    if m.is_volatile {
        return None;
    }
    // C++ 成员函数指针无法映射为有效 Rust FFI 类型，跳过
    if m.params.iter().any(|p| p.type_name.contains("::*)")) || m.return_type.contains("::*)") {
        return None;
    }
    let rust_name = sanitize_fn_name(&m.name);
    let self_kind = if m.is_const {
        SelfKind::Ref
    } else {
        SelfKind::RefMut
    };

    let params: Vec<(String, String)> = m
        .params
        .iter()
        .enumerate()
        .map(|(i, p)| (sanitize_param_name(&p.name, i), cpp_to_rust(&p.type_name)))
        .collect();

    let ret_type = ret_type_from_cpp(&m.return_type);

    // 检测参数或返回类型是否含 C 函数指针，用于生成 cpp2rust-todo[FP] 注释
    let has_fn_ptr_param =
        m.params.iter().any(|p| p.type_name.contains("(*)")) || m.return_type.contains("(*)");

    // C++ 方法签名：含参数名（若 AST 有）、剥除参数 volatile、指针紧贴类型
    // 返回类型 volatile 和方法 this-volatile 均需保留，供 hicc 编译时方法指针类型检查
    let param_types: Vec<String> = m
        .params
        .iter()
        .map(|p| {
            let ty = normalize_ptr_spacing(strip_volatile(clean_type(&p.type_name)));
            // C++ 签名中：有名字则 "type name"，无名则仅 "type"
            if !p.name.is_empty() && p.name != "_" {
                format!("{} {}", ty, p.name)
            } else {
                ty.to_string()
            }
        })
        .collect();
    let ret_clean = normalize_ptr_spacing(strip_volatile(clean_type(&m.return_type)));
    let cv_suffix = match (m.is_const, m.is_volatile) {
        (true, true) => " const volatile",
        (true, false) => " const",
        (false, true) => " volatile",
        (false, false) => "",
    };
    let cpp_sig = if m.return_type.is_empty() || m.return_type == "void" {
        format!("void {}({}){}", m.name, param_types.join(", "), cv_suffix)
    } else {
        format!(
            "{} {}({}){}",
            ret_clean,
            m.name,
            param_types.join(", "),
            cv_suffix
        )
    };

    Some(MethodBinding {
        cpp_sig,
        rust_name,
        self_kind,
        params,
        ret_type,
        has_fn_ptr_param,
    })
}
