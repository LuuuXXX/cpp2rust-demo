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
    // 真正的旧式 extern-C 桥接函数一定是**全局作用域**（namespace=None）且签名中
    // 引用某个类类型（不透明句柄指针，如 `Counter*`/`void*`）。libclang 把 .cpp 内
    // 的 C++ 命名空间函数（如 pugixml 的 `pugi::impl::default_allocate`）也误判为
    // extern-C，但它们带 namespace=Some(...)，可据此区分。
    let has_extern_c_bridge = ast.functions.iter().any(|f| {
        f.is_extern_c
            && f.namespace.is_none()  // 仅全局 extern-C 才算旧式桥接
            && !f.name.ends_with("_anchor")
            && fn_references_class(f, &class_names)
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
/// **跳过抽象类**：含纯虚方法的类（`is_abstract == true`）无法被 `make_unique` 实例化，
/// 一律跳过，避免生成的 `make_unique<AbstractClass>` 在 C++ 侧编译失败。
///
/// **拓扑排序**：若类 A 的方法签名引用类 B（参数或返回类型），B 必须先于 A 声明，
/// 否则 hicc 内部 `MethodsType<B>` 会在 A 的 `import_class!` 块内被先实例化，
/// 待 B 自身的 `import_class!` 块再特化时触发「specialization after instantiation」
/// 编译错误。本函数对入选类按引用关系做拓扑排序，被引用的类排在前。
pub(super) fn build_hicc_direct_specs(ast: &CppAst) -> Vec<ClassSpec> {
    let mut specs = Vec::new();
    // 候选类：命名空间类 + 含公有构造 + 非抽象 + **来自头文件**（is_from_current_file=false）
    //
    // is_from_current_file 过滤：pugixml 等单 .cpp 库会把内部类型（xml_allocator 等）
    // 定义在 .cpp 的匿名命名空间内。这些类型不属于公开 API，且其定义在编译时被
    // 限制在 .cpp 的编译单元内，外部无法链接。生成 import_class! 绑定会导致
    // hicc 在编译生成的 C++ 代码时报「type not declared in this scope」。
    //
    // detail 命名空间过滤：fmt / nlohmann-json / magic_enum 等库的「detail」子命名空间
    // 内含大量模板类（如 `fmt::v12::detail::iterator_buffer`）。这些是内部实现细节，
    // 不属于稳定 API，且多为模板声明（非具体实例化），生成 import_class! 会触发
    // hicc MethodsType 模板参数错误。一律跳过。
    let candidates: Vec<&ClassInfo> = ast
        .classes
        .iter()
        .filter(|c| {
            if !c.is_in_namespace || !has_public_ctor(c) || c.is_abstract || c.is_from_current_file
            {
                return false;
            }
            // 跳过库内部命名空间内的类（detail / impl / internal / customize / priv 等）
            if let Some(ns) = &c.namespace {
                if ns.split("::").any(|seg| {
                    matches!(
                        seg,
                        "detail" | "impl" | "internal" | "customize" | "priv" | "private"
                    )
                }) {
                    return false;
                }
            }
            true
        })
        .collect();
    // 已导出的简单类名集合，供方法类型映射合法性检查使用
    let exported: Vec<&str> = candidates.iter().map(|c| c.simple_name.as_str()).collect();
    // 简单名 → 命名空间限定名映射，供方法签名中的类引用补全限定
    let qual_map: Vec<(&str, String)> = candidates
        .iter()
        .map(|c| (c.simple_name.as_str(), c.qualified_name()))
        .collect();

    // 拓扑排序：若 A 的方法引用 B，则 B 应在 A 之前
    if std::env::var("CPP2RUST_DEBUG_TOPO").is_ok() {
        let names: Vec<&str> = candidates.iter().map(|c| c.simple_name.as_str()).collect();
        eprintln!("[hicc_direct] {} candidates", names.len());
        for n in &names {
            eprintln!("  - {}", n);
        }
    }
    let ordered_indices = topo_sort_by_reference(&candidates, &exported);

    // 破环过滤：对拓扑序中"靠前"的类，跳过引用"靠后"类的方法。
    // 背景：hicc 内部 MethodsType<T> 在首次见到 T 的类型引用时实例化主模板，
    // 待 T 自身的 import_class! 块再显式特化时若 T 已被实例化，触发
    // "specialization after instantiation" 编译错误。
    // 当 A、B 互相引用成环时（如 tinyxml2 中 XMLDocument::Print(XMLPrinter*) 与
    // XMLPrinter::VisitEnter(XMLDocument&)），拓扑序必有一方"靠前"——其引用
    // "靠后"类的方法必须跳过，让环的另一方向成为唯一的引用方向。
    // 代价：被跳过的方法不进入 Rust FFI（保守安全），用户可手写 import_lib! 补回。
    let position: std::collections::HashMap<usize, usize> = ordered_indices
        .iter()
        .enumerate()
        .map(|(pos, &idx)| (idx, pos))
        .collect();

    for &idx in &ordered_indices {
        let ci = candidates[idx];
        let my_pos = position[&idx];
        // 收集"位置比我靠后"的候选类简单名（这些类引用我会导致前向实例化）
        let later_classes: Vec<&str> = candidates
            .iter()
            .enumerate()
            .filter(|&(j, _)| position[&j] > my_pos)
            .map(|(_, c)| c.simple_name.as_str())
            .collect();
        if let Some(cs) = build_one_filtered(ci, &exported, &qual_map, &later_classes) {
            specs.push(cs);
        }
    }
    specs
}

/// 根据候选类之间的方法签名引用关系做拓扑排序：被引用的类排在引用者之前。
///
/// 节点：候选类索引（0..n）。边：A → B 表示「A 的方法签名引用了 B 的简单名」，
/// 拓扑序要求 B 在 A 之前输出。带环时退化为原始顺序（保守安全，不丢类）。
fn topo_sort_by_reference(candidates: &[&ClassInfo], exported: &[&str]) -> Vec<usize> {
    let n = candidates.len();
    // 邻接表：deps[i] = { j : candidates[i] 引用了 candidates[j].simple_name }
    let mut deps: Vec<std::collections::BTreeSet<usize>> = vec![Default::default(); n];
    for (i, ci) in candidates.iter().enumerate() {
        for m in ci.methods.iter() {
            // 检查方法签名（参数 + 返回类型）是否引用其它候选类
            let sig_checks: Vec<&str> = std::iter::once(m.return_type.as_str())
                .chain(m.params.iter().map(|p| p.type_name.as_str()))
                .collect();
            for sig in sig_checks {
                for (j, other) in candidates.iter().enumerate() {
                    if i == j {
                        continue;
                    }
                    if super::type_references_class(sig, &other.simple_name)
                        && exported.contains(&other.simple_name.as_str())
                    {
                        deps[i].insert(j);
                    }
                }
            }
        }
    }
    if std::env::var("CPP2RUST_DEBUG_TOPO").is_ok() {
        for (i, ci) in candidates.iter().enumerate() {
            let dep_names: Vec<&str> = deps[i]
                .iter()
                .map(|&d| candidates[d].simple_name.as_str())
                .collect();
            eprintln!("[topo] {} deps=[{}]", ci.simple_name, dep_names.join(", "));
        }
    }

    // Kahn 算法

    // Kahn 算法：每轮选出「所有依赖均已输出」的节点；同轮内按原顺序稳定输出
    let mut emitted = Vec::with_capacity(n);
    let mut done = vec![false; n];
    while emitted.len() < n {
        let mut progressed = false;
        let mut to_emit: Vec<usize> = Vec::new();
        for idx in 0..n {
            if done[idx] {
                continue;
            }
            if deps[idx].iter().all(|&dep| done[dep]) {
                to_emit.push(idx);
                progressed = true;
            }
        }
        for idx in to_emit {
            emitted.push(idx);
            done[idx] = true;
        }
        if !progressed {
            // 环：把剩余节点按原顺序追加（保守不丢）
            let remaining: Vec<usize> = (0..n).filter(|&i| !done[i]).collect();
            for idx in remaining {
                emitted.push(idx);
                done[idx] = true;
            }
            break;
        }
    }
    emitted
}

/// 把 C++ 方法签名字符串中「裸的已导出类简单名」补全为命名空间限定名。
///
/// 以标识符 token 为单位匹配（避免误伤含该名子串的其它标识符），且当 token
/// 紧跟在 `::` 之后（即已限定）时跳过，避免重复限定。
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
            let replacement = if already_qualified {
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

#[allow(dead_code)] // 保留作为单参数入口（与 build_one_filtered 等价，empty later_classes）
fn build_one(ci: &ClassInfo, exported: &[&str], qual_map: &[(&str, String)]) -> Option<ClassSpec> {
    build_one_filtered(ci, exported, qual_map, &[])
}

/// 与 `build_one` 相同，但额外跳过「方法签名引用 `later_classes` 中任一类」的方法。
///
/// `later_classes` 为「在拓扑序中位于本类之后」的候选类简单名集合。
/// 跳过这些方法可打破 hicc MethodsType 在环引用场景下的「specialization after
/// instantiation」编译错误（详见 `build_hicc_direct_specs` 的破环过滤注释）。
fn build_one_filtered(
    ci: &ClassInfo,
    exported: &[&str],
    qual_map: &[(&str, String)],
    later_classes: &[&str],
) -> Option<ClassSpec> {
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
        // 破环过滤：方法签名（返回类型 + 参数类型）引用了 later_classes 任一类则跳过
        let sig_tokens: Vec<&str> = std::iter::once(m.return_type.as_str())
            .chain(m.params.iter().map(|p| p.type_name.as_str()))
            .collect();
        let refs_later = sig_tokens.iter().any(|sig| {
            later_classes
                .iter()
                .any(|lc| super::type_references_class(sig, lc))
        });
        if refs_later {
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
        // 同样需要 qualify_class_types 对类引用做命名空间限定（与 call_types 一致）。
        let tmpl_types: Vec<String> = m
            .params
            .iter()
            .map(|p| {
                let raw = make_unique_template_type(&p.type_name);
                qualify_class_types(&raw, qual_map)
            })
            .collect();
        let targs = if tmpl_types.is_empty() {
            qualified.clone()
        } else {
            format!("{}, {}", qualified, tmpl_types.join(", "))
        };
        // 调用实参类型也需对类引用做命名空间限定（与方法的 qualify_class_types 对齐），
        // 否则 hicc 生成的 C++ 代码在全局作用域无法解析裸类名（如 `xml_node` 而非
        // `pugi::xml_node`）。
        let call_types: Vec<String> = m
            .params
            .iter()
            .map(|p| {
                let raw = make_unique_arg_type(&p.type_name);
                qualify_class_types(&raw, qual_map)
            })
            .collect();
        let make_unique_sig = format!(
            "std::unique_ptr<{q}> hicc::make_unique<{targs}>({sig})",
            q = qualified,
            targs = targs,
            sig = call_types.join(", ")
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

#[cfg(test)]
mod tests {
    use super::qualify_class_types;

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
        // `Vector` 含子串 `Vec`，但作为独立 token 不应被替换
        let got = qualify_class_types("int Vectorize(int Vec)", &map);
        assert_eq!(got, "int Vectorize(int ns::Vec)");
    }

    #[test]
    fn leaves_primitive_types_untouched() {
        let map = vec![("Buffer", "ns::Buffer".to_string())];
        let sig = "int get(int index) const";
        assert_eq!(qualify_class_types(sig, &map), sig);
    }
}
