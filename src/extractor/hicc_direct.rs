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
    // 进一步排除「命名空间类 + 残留 extern "C" opaque 指针桥接块」的半旧式示例。
    //
    // 注意：libclang 会把**所有**命名空间作用域的普通 C++ 自由函数（如
    // `int safe_add(int, int)`）误标为 `extern "C"`（get_language()==C），因此不能
    // 仅凭 `is_extern_c` 判定旧式桥接，否则会把含标量自由函数的 idiomatic 示例
    // （031/033/043/044/046/047/048 等）误判为旧路径而丢失类绑定。
    //
    // 真正的旧式 extern-C 桥接函数一定在签名（返回值或任一参数）中**引用某个类
    // 类型**（不透明句柄指针，如 `Counter*`/`void*`）。idiomatic 标量自由函数则不会。
    // 据此区分：仅当存在「引用类类型的 extern-C 非锚点函数」时才认为是旧式桥接。
    let has_extern_c_bridge = ast.functions.iter().any(|f| {
        f.is_extern_c && !f.name.ends_with("_anchor") && fn_references_class(f, &class_names)
    });
    !has_extern_c_bridge
}

/// 函数签名（返回类型或任一参数类型）是否引用了 `class_names` 中的某个类。
///
/// 用于区分「旧式 extern-C 不透明指针桥接函数」（引用类句柄）与「idiomatic 命名
/// 空间标量自由函数」（仅标量/`const char*` 等可直出类型）。
fn fn_references_class(f: &crate::ast_parser::FunctionInfo, class_names: &[&str]) -> bool {
    let in_ret = class_names
        .iter()
        .any(|cn| super::type_references_class(&f.return_type, cn));
    let in_params = f.params.iter().any(|p| {
        class_names
            .iter()
            .any(|cn| super::type_references_class(&p.type_name, cn))
    });
    in_ret || in_params
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
///
/// 采用两轮确认策略：
/// 1. 第一轮用「所有非抽象、有公有构造的命名空间类」作候选导出集，预判哪些类能生成
///    有效 ClassSpec（build_one 返回 Some）。
/// 2. 第二轮以「确认能生成规格的类」作最终导出集重新生成规格，保证 exported 中每个
///    类名都有对应的 Rust 类型定义，避免被其他类方法引用时出现「类型未定义」错误。
///
/// 抽象类（is_abstract = true，含纯虚函数）不可被 make_unique 实例化，必须排除。
///
/// 内联命名空间展开时同一个类可能在 `ast.classes` 中重复出现（例如 `toml::v3::table`
/// 同时出现在外层 `toml` 命名空间和内层 `v3` 命名空间遍历中），最终生成阶段按限定名
/// 去重，避免 hicc C++ 侧 `MethodsType<T>` 二次特化导致编译失败。
pub(super) fn build_hicc_direct_specs(ast: &CppAst) -> Vec<ClassSpec> {
    // 候选导出集：非抽象、有公有构造的命名空间类
    let candidate_exported: Vec<&str> = ast
        .classes
        .iter()
        .filter(|c| c.is_in_namespace && !c.is_abstract && has_public_ctor(c))
        .map(|c| c.simple_name.as_str())
        .collect();
    let candidate_qual: Vec<(&str, String)> = ast
        .classes
        .iter()
        .filter(|c| c.is_in_namespace && !c.is_abstract && has_public_ctor(c))
        .map(|c| (c.simple_name.as_str(), c.qualified_name()))
        .collect();

    // 确认集合：候选中能通过 build_one 生成非空 ClassSpec 的类
    let confirmed: std::collections::HashSet<&str> = ast
        .classes
        .iter()
        .filter(|ci| ci.is_in_namespace && !ci.is_abstract && has_public_ctor(ci))
        .filter_map(|ci| {
            if build_one(ci, &candidate_exported, &candidate_qual).is_some() {
                Some(ci.simple_name.as_str())
            } else {
                None
            }
        })
        .collect();

    // 最终导出集与 qual_map：仅含确认能生成规格的类
    let exported: Vec<&str> = candidate_exported
        .iter()
        .copied()
        .filter(|s| confirmed.contains(*s))
        .collect();
    let qual_map: Vec<(&str, String)> = candidate_qual
        .iter()
        .filter(|(simple, _)| confirmed.contains(*simple))
        .map(|(s, q)| (*s, q.clone()))
        .collect();

    // 以最终导出集生成规格；按限定名去重，避免内联命名空间展开导致同一类被注册两次
    let mut specs = Vec::new();
    let mut seen_qual: std::collections::HashSet<String> = std::collections::HashSet::new();
    for ci in ast.classes.iter() {
        if !ci.is_in_namespace || ci.is_abstract || !has_public_ctor(ci) {
            continue;
        }
        let qname = ci.qualified_name();
        if !seen_qual.insert(qname) {
            continue; // 跳过重复（内联命名空间二次暴露）
        }
        if let Some(cs) = build_one(ci, &exported, &qual_map) {
            specs.push(cs);
        }
    }
    specs
}

/// 把 C++ 方法签名字符串中「裸的已导出类简单名」补全为命名空间限定名。
///
/// 以标识符 token 为单位匹配（避免误伤含该名子串的其它标识符），且当 token
/// 紧跟在 `::` 之后（即已限定）时跳过，避免重复限定。
///
/// 额外跳过「参数名/函数名位置」的 token：若 token 之后（跳过空白）紧跟
/// `)` 或 `,`（参数名位置）或 `(`（函数名位置），则该 token 是标识符而非类型，
/// 不应被限定。例如 `void set(const key & key)` 中，类型位置的 `key` 被限定为
/// `toml::v3::key`，而参数名 `key` 在 `)` 之前则不被修改。
fn qualify_class_types(sig: &str, qual_map: &[(&str, String)]) -> String {
    if qual_map.is_empty() {
        return sig.to_string();
    }
    let bytes = sig.as_bytes();
    let is_ident = |b: u8| b.is_ascii_alphanumeric() || b == b'_';
    let mut out = String::with_capacity(sig.len());
    let mut i = 0usize;
    while i < bytes.len() {
        let b = bytes[i];
        if is_ident(b) {
            let start = i;
            while i < bytes.len() && is_ident(bytes[i]) {
                i += 1;
            }
            let token = &sig[start..i];
            // 已限定（前缀为 `::`）则不再替换
            let already_qualified = start >= 2 && &sig[start - 2..start] == "::";
            // 参数名/函数名位置：token 之后（跳过空白）紧跟 `)`、`,`、`(` 时，
            // 该 token 是名称而非类型，不替换。
            // `)` / `,` → 参数名（最后一个标识符之前），`(` → 函数名。
            let is_name_position = !already_qualified && {
                let rest = sig[i..].trim_start();
                rest.starts_with(')') || rest.starts_with(',') || rest.starts_with('(')
            };
            let replacement = if already_qualified || is_name_position {
                None
            } else {
                qual_map
                    .iter()
                    .find(|(simple, _)| *simple == token)
                    .map(|(_, qual)| qual.as_str())
            };
            match replacement {
                Some(q) => out.push_str(q),
                None => out.push_str(token),
            }
        } else {
            out.push(b as char);
            i += 1;
        }
    }
    out
}

fn build_one(ci: &ClassInfo, exported: &[&str], qual_map: &[(&str, String)]) -> Option<ClassSpec> {
    let qualified = ci.qualified_name();

    // ── 成员方法（复用通用 MethodBinding 构建）──
    let mut methods = Vec::new();
    for m in ci.methods.iter().filter(|m| {
        !m.is_constructor
            && !m.is_destructor
            && m.accessibility == "public"
            && !m.is_static
            // 跳过引用限定（`& `/`&&`）方法：其函数指针类型含引用限定符，
            // 与 hicc export_method 所需的普通成员函数指针不兼容。
            && !m.is_ref_qualified
    }) {
        // 跳过 operator 重载（由 cpp! 命名包装处理）与 Rust 关键字方法名
        if m.name.starts_with("operator") {
            continue;
        }
        if let Some(mb) = build_method_binding(m) {
            // 仅保留参数/返回类型均可直出映射的方法（其余留待手写示例补全）
            // 额外过滤：排除「参数或返回值为指向其他已导出类的裸指针」的方法，
            // 避免 hicc 在生成 C++ 文件时因前向引用而触发
            // "specialization after instantiation" 错误（循环依赖类如
            // tinyxml2::XMLDocument ↔ XMLPrinter）。自身类的裸指针（self-ref）
            // 不受此限，因为当前类的 MethodsType 特化先于方法注册生成。
            if method_types_simple(&mb, exported)
                && !has_cross_class_ptr(&mb, exported, &ci.name)
            {
                methods.push(mb);
            }
        }
    }
    // 方法名去重（hicc import_class! 不支持同名方法）：
    // 对同名重载，优先保留含非基本类型参数最多的版本（libclang 在某些平台（如
    // Windows LLVM 17）可能因模板参数推断失败把复杂类型参数误报为 `int`，需要
    // 选出质量更好的那个）；且保持原始方法出现顺序不变（不做整体排序）。
    let non_primitive_count = |mb: &crate::ffi_model::MethodBinding| -> usize {
        mb.params
            .iter()
            .filter(|(_, t)| !is_primitive_rust_type(t))
            .count()
    };
    // 第一遍：找出每个 rust_name 对应的最大非基本类型参数数
    let mut max_non_prim: std::collections::HashMap<String, usize> =
        std::collections::HashMap::new();
    for mb in &methods {
        let cnt = non_primitive_count(mb);
        let e = max_non_prim.entry(mb.rust_name.clone()).or_insert(0);
        if cnt > *e {
            *e = cnt;
        }
    }
    // 第二遍：按原顺序保留，每个名字只保留第一个满足「非基本类型数 == 最大值」的版本
    let mut seen_dedup: std::collections::HashSet<String> = std::collections::HashSet::new();
    methods.retain(|mb| {
        let max = *max_non_prim.get(&mb.rust_name).unwrap_or(&0);
        non_primitive_count(mb) >= max && seen_dedup.insert(mb.rust_name.clone())
    });

    // 方法 C++ 签名中对「其它已导出命名空间类」的裸引用需补全命名空间限定，
    // 否则 hicc 在 `cpp!` 展开处按全局作用域解析类型名会编译失败
    // （如 `void move_from(UniqueVector & src)` 应为 `class_move_ns::UniqueVector &`）。
    for mb in methods.iter_mut() {
        mb.cpp_sig = qualify_class_types(&mb.cpp_sig, qual_map);
    }

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
        // make_unique 模板实参用「衰减类型」（按值 T、引用/指针原样），
        // 调用实参用「转发类型」（按值 T&&、引用/指针原样），与 hicc 蓝本一致：
        // `make_unique<Widget, int>(int&&)`。
        let tmpl_types: Vec<String> = m
            .params
            .iter()
            .map(|p| make_unique_template_type(&p.type_name))
            .collect();
        let call_types: Vec<String> = m
            .params
            .iter()
            .map(|p| make_unique_arg_type(&p.type_name))
            .collect();
        let targs = if tmpl_types.is_empty() {
            qualified.clone()
        } else {
            format!("{}, {}", qualified, tmpl_types.join(", "))
        };
        let make_unique_sig = format!(
            "std::unique_ptr<{q}> hicc::make_unique<{targs}>({sig})",
            q = qualified,
            targs = targs,
            sig = call_types.join(", ")
        );
        // 构造工厂签名中的裸类型名同样需要命名空间限定
        let make_unique_sig = qualify_class_types(&make_unique_sig, qual_map);
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

/// make_unique 调用实参的 C++ 类型：左值/右值引用原样（引用折叠），其余（标量、
/// 指针）一律追加 `T&&`，以匹配 `make_unique(ArgTypes&&...)` 的转发引用形参。
fn make_unique_arg_type(cpp_ty: &str) -> String {
    let t = super::normalize_ptr_spacing(super::strip_volatile(super::type_mapper::clean_type(
        cpp_ty,
    )));
    if t.contains('&') {
        t
    } else {
        format!("{}&&", t)
    }
}

/// make_unique 模板实参的 C++ 类型：标量用衰减类型 `T`（不带 `&&`），引用/指针原样。
fn make_unique_template_type(cpp_ty: &str) -> String {
    super::normalize_ptr_spacing(super::strip_volatile(super::type_mapper::clean_type(
        cpp_ty,
    )))
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

/// 检测方法的参数或返回值是否包含指向「其他已导出类（非当前类）」的裸指针。
///
/// hicc 在生成 C++ 展开代码时，对每个 `import_class!` 先输出
/// `MethodsType<T>` 特化，再输出方法注册宏。若某方法的参数类型为
/// `*mut OtherClass`，hicc 会尝试访问 `MethodsType<OtherClass>` 的
/// 特化——若 OtherClass 尚未被特化（顺序靠后或循环依赖），编译器就会
/// 先实例化主模板，导致后续真正的特化报 "specialization after instantiation"
/// 错误。自身类的裸指针（`*mut SelfClass`）安全，因为特化在方法体前已生成。
fn has_cross_class_ptr(
    mb: &crate::ffi_model::MethodBinding,
    exported: &[&str],
    self_name: &str,
) -> bool {
    let is_cross_ptr = |t: &str| {
        let inner = t
            .strip_prefix("*mut ")
            .or_else(|| t.strip_prefix("*const "));
        if let Some(inner) = inner {
            // i8/u8 是 C 字符串，不是导出类
            if inner == "i8" || inner == "u8" {
                return false;
            }
            // 是其他导出类（非自身）的裸指针
            inner != self_name && exported.contains(&inner)
        } else {
            false
        }
    };
    mb.params.iter().any(|(_, t)| is_cross_ptr(t))
        || mb.ret_type.as_deref().map(|t| is_cross_ptr(t)).unwrap_or(false)
}

/// 判断 Rust 类型是否为内置基本类型（不含类/结构体引用）。
///
/// 用于方法去重前排序：含非基本类型参数的方法优先保留，避免 libclang 在模板参数
/// 推断失败时将复杂类型误报为 `i32` 所产生的低质量重载覆盖正确签名。
fn is_primitive_rust_type(t: &str) -> bool {
    matches!(
        t,
        "i8" | "i16"
            | "i32"
            | "i64"
            | "i128"
            | "u8"
            | "u16"
            | "u32"
            | "u64"
            | "u128"
            | "f32"
            | "f64"
            | "bool"
            | "isize"
            | "usize"
            | "()"
    )
}

#[cfg(test)]
mod tests {
    use super::{is_primitive_rust_type, qualify_class_types};

    #[test]
    fn qualifies_bare_exported_class_param() {
        let map = vec![("UniqueVector", "class_move_ns::UniqueVector".to_string())];
        let got = qualify_class_types("void move_from(UniqueVector & src)", &map);
        assert_eq!(got, "void move_from(class_move_ns::UniqueVector & src)");
    }

    #[test]
    fn does_not_double_qualify() {
        let map = vec![("UniqueVector", "class_move_ns::UniqueVector".to_string())];
        let sig = "void move_from(class_move_ns::UniqueVector & src)";
        assert_eq!(qualify_class_types(sig, &map), sig);
    }

    #[test]
    fn does_not_touch_substring_identifiers() {
        let map = vec![("Vec", "ns::Vec".to_string())];
        // `Vectorize` 含子串 `Vec`，但整体 token 不同，不应被替换；
        // `Vec` 作为独立类型 token（后跟参数名 `a`，非名称位置）应正确限定。
        let got = qualify_class_types("int Vectorize(Vec a)", &map);
        assert_eq!(got, "int Vectorize(ns::Vec a)");
    }

    #[test]
    fn leaves_primitive_types_untouched() {
        let map = vec![("Buffer", "ns::Buffer".to_string())];
        let sig = "int get(int index) const";
        assert_eq!(qualify_class_types(sig, &map), sig);
    }

    #[test]
    fn does_not_qualify_parameter_name_same_as_class() {
        // `const key & key`：类型位置的 key 应被限定，参数名位置的 key（`) `之前）不应被限定
        let map = vec![("key", "toml::v3::key".to_string())];
        let sig = "size_t count(const key & key) const";
        let got = qualify_class_types(sig, &map);
        assert_eq!(got, "size_t count(const toml::v3::key & key) const");
    }

    #[test]
    fn does_not_qualify_method_name() {
        // 方法名 `key` 出现在 `(` 之前，不应被限定
        let map = vec![("key", "toml::v3::key".to_string())];
        let sig = "key_type key() const";
        // `key` as method name before `(` should not be replaced
        let got = qualify_class_types(sig, &map);
        assert_eq!(got, "key_type key() const");
    }

    #[test]
    fn qualifies_return_type_named_same_as_class() {
        // 返回类型 `key` 应被限定
        let map = vec![("key", "toml::v3::key".to_string())];
        let sig = "const key & get_key() const";
        let got = qualify_class_types(sig, &map);
        assert_eq!(got, "const toml::v3::key & get_key() const");
    }

    #[test]
    fn primitive_rust_types_detected() {
        assert!(is_primitive_rust_type("i32"));
        assert!(is_primitive_rust_type("bool"));
        assert!(is_primitive_rust_type("u64"));
        assert!(!is_primitive_rust_type("&key"));
        assert!(!is_primitive_rust_type("*const i8"));
    }
}
