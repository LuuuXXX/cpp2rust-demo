//! hicc 直出提取（去 shim 核心）
//!
//! 针对 **idiomatic 命名空间类**（真实 `class ns::T`，含公有构造函数与成员方法，
//! 无 `extern "C"` opaque 指针桥接）直接构建 hicc 三段式绑定规格：
//!
//! - `import_class!` 用 `#[cpp(class = "ns::T")]` 直接绑定真实命名空间类与方法
//!   （const 方法 → `&self`，非 const → `&mut self`）；
//! - 每个公有构造函数派生一条 `hicc::make_unique<T, Args...>` 工厂（在 `import_lib!`
//!   输出），并在 `import_class!` body 内生成关联函数 `pub fn new(...) -> Self { factory(...) }`；
//! - 析构交给 hicc 的 `Drop`，不再生成 `*_delete`/`destroy =` shim。
//!
//! 该路径替代旧的「extern "C" 不透明指针 + `*_new`/`*_delete` C ABI 桥接」策略
//! （见 `mod.rs` 的 `ShimKind` 分类）。仅当检测到 idiomatic 命名空间类模式时启用，
//! 现有 extern-C 示例仍走旧路径，互不影响。

use super::class_spec::build_method_binding;
use super::type_mapper::{cpp_to_rust, to_snake_case};
use crate::ast_parser::{ClassInfo, CppAst, MethodInfo};
use crate::ffi_model::{ClassSpec, CtorFactory};

/// 检测当前 AST 是否适用 hicc 直出（idiomatic 命名空间类）模式。
///
/// 判据：存在「带公有构造函数的命名空间类」，且**不存在任何 `extern "C"` 函数**
/// （后者表明是旧式 opaque 指针 + C ABI 桥接示例，仍走旧路径）。
pub(super) fn detect_idiomatic_mode(ast: &CppAst) -> bool {
    let has_ns_class_with_ctor = ast
        .classes
        .iter()
        .any(|c| c.is_in_namespace && has_public_ctor(c));
    if !has_ns_class_with_ctor {
        return false;
    }
    // 排除旧式 shim 示例：只要存在被分类为 Ctor/Dtor 桥接的函数（如 `*_new`/`*_delete`
    // 返回类指针/void*），即认为是 extern-C opaque 桥接示例，仍走旧路径。
    let class_names: Vec<&str> = ast.classes.iter().map(|c| c.name.as_str()).collect();
    let has_shim_ctor_dtor = ast.functions.iter().any(|f| {
        matches!(
            super::classify_fn(f, &class_names),
            super::ShimKind::Ctor | super::ShimKind::Dtor
        )
    });
    if has_shim_ctor_dtor {
        return false;
    }
    // 进一步排除「命名空间类 + 残留 extern "C" 桥接块」的半旧式示例：
    // 已转换为 idiomatic 的示例除约定的 `*_anchor` 锚点函数（libclang 会把全局
    // 普通 C++ 函数误标为 `extern "C"`）外，不应再有任何 `extern "C"` 函数。
    let has_extern_c_bridge = ast
        .functions
        .iter()
        .any(|f| f.is_extern_c && !f.name.ends_with("_anchor"));
    !has_extern_c_bridge
}

/// 是否存在至少一个公有、非拷贝/移动的构造函数。
fn has_public_ctor(ci: &ClassInfo) -> bool {
    ci.methods.iter().any(is_usable_public_ctor)
}

/// 公有、可作为工厂来源的构造函数（排除拷贝/移动构造）。
fn is_usable_public_ctor(m: &MethodInfo) -> bool {
    m.is_constructor && m.accessibility == "public" && !is_copy_or_move_ctor(m)
}

/// 判定拷贝构造（`T(const T&)`）/ 移动构造（`T(T&&)`）。
fn is_copy_or_move_ctor(m: &MethodInfo) -> bool {
    if !m.is_constructor || m.params.len() != 1 {
        return false;
    }
    let t = &m.params[0].type_name;
    // 形如 `const T &` / `T &&`（含命名空间限定时按尾段类名宽松匹配）
    t.contains('&')
}

/// 为 idiomatic 命名空间类构建 hicc 直出 `ClassSpec` 列表。
///
/// 仅处理 `is_in_namespace` 且含公有构造的类。方法/构造参数中含暂不可直出映射的类型
/// （如 `std::string`、未知类）时，会被保守跳过，留待手写示例补全（与黄金支架一致）。
pub(super) fn build_hicc_direct_specs(ast: &CppAst) -> Vec<ClassSpec> {
    let mut specs = Vec::new();
    // 已导出的简单类名集合，供方法类型映射合法性检查使用
    let exported: Vec<&str> = ast
        .classes
        .iter()
        .filter(|c| c.is_in_namespace && has_public_ctor(c))
        .map(|c| c.simple_name.as_str())
        .collect();

    for ci in ast.classes.iter() {
        if !ci.is_in_namespace || !has_public_ctor(ci) {
            continue;
        }
        if let Some(cs) = build_one(ci, &exported) {
            specs.push(cs);
        }
    }
    specs
}

fn build_one(ci: &ClassInfo, exported: &[&str]) -> Option<ClassSpec> {
    let qualified = ci.qualified_name();

    // ── 成员方法（复用通用 MethodBinding 构建）──
    let mut methods = Vec::new();
    for m in ci.methods.iter().filter(|m| {
        !m.is_constructor && !m.is_destructor && m.accessibility == "public" && !m.is_static
    }) {
        // 跳过 operator 重载（由 cpp! 命名包装处理）与 Rust 关键字方法名
        if m.name.starts_with("operator") {
            continue;
        }
        if let Some(mb) = build_method_binding(m) {
            // 仅保留参数/返回类型均可直出映射的方法（其余留待手写示例补全）
            if method_types_simple(&mb, exported) {
                methods.push(mb);
            }
        }
    }
    // 方法名去重（hicc import_class! 不支持同名方法）
    let mut seen = std::collections::HashSet::new();
    methods.retain(|mb| seen.insert(mb.rust_name.clone()));

    // ── 构造工厂（每个可用公有构造一条 make_unique）──
    let snake = to_snake_case(&ci.simple_name);
    let mut ctor_factories: Vec<CtorFactory> = Vec::new();
    let mut idx = 0usize;
    for m in ci.methods.iter().filter(|m| is_usable_public_ctor(m)) {
        // 仅当全部构造参数为可直出映射类型时纳入支架（其余留待手写示例补全）
        let rust_params: Vec<(String, String)> = m
            .params
            .iter()
            .enumerate()
            .map(|(i, p)| {
                (
                    super::sanitize_param_name(&p.name, i),
                    cpp_to_rust(&p.type_name),
                )
            })
            .collect();
        if !rust_params
            .iter()
            .all(|(_, t)| super::is_mappable_rust_type(t, exported))
        {
            continue;
        }
        let ctor_fn = if idx == 0 {
            "new".to_string()
        } else {
            format!("new_{}", idx + 1)
        };
        let factory_rust_name = if idx == 0 {
            format!("{}_new", snake)
        } else {
            format!("{}_new_{}", snake, idx + 1)
        };
        // make_unique C++ 签名：参数按值用 `T&&`，引用/指针原样保留
        let arg_types: Vec<String> = m
            .params
            .iter()
            .map(|p| make_unique_arg_type(&p.type_name))
            .collect();
        let targs = if arg_types.is_empty() {
            qualified.clone()
        } else {
            format!("{}, {}", qualified, arg_types.join(", "))
        };
        let make_unique_sig = format!(
            "std::unique_ptr<{q}> hicc::make_unique<{targs}>({sig})",
            q = qualified,
            targs = targs,
            sig = arg_types.join(", ")
        );
        ctor_factories.push(CtorFactory {
            ctor_fn,
            factory_rust_name,
            params: rust_params,
            make_unique_sig,
            ret_class: ci.simple_name.clone(),
            non_snake_case: false,
        });
        idx += 1;
    }

    if methods.is_empty() && ctor_factories.is_empty() {
        return None;
    }

    Some(ClassSpec {
        name: ci.simple_name.clone(),
        methods,
        associated_fns: Vec::new(),
        destroy_fn: None,
        is_interface: false,
        hicc_direct: true,
        cpp_class: Some(qualified),
        ctor_factories,
    })
}

/// make_unique 模板实参/调用实参的 C++ 类型：标量按值参用 `T&&`，引用/指针原样。
fn make_unique_arg_type(cpp_ty: &str) -> String {
    let t = super::normalize_ptr_spacing(super::strip_volatile(super::type_mapper::clean_type(
        cpp_ty,
    )));
    if t.contains('&') || t.contains('*') {
        t
    } else {
        format!("{}&&", t)
    }
}

/// 方法的参数与返回类型是否均为可直出映射的简单类型。
fn method_types_simple(mb: &crate::ffi_model::MethodBinding, exported: &[&str]) -> bool {
    mb.params
        .iter()
        .all(|(_, t)| super::is_mappable_rust_type(t, exported))
        && mb
            .ret_type
            .as_deref()
            .map(|t| super::is_mappable_rust_type(t, exported))
            .unwrap_or(true)
}
