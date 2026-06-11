//! hicc Rust 代码生成器（Phase 5）
//!
//! 将 `FfiSpec` 生成三段式 hicc Rust 代码：
//! `hicc::cpp!` + `hicc::import_class!`（有成员方法的类）+ `hicc::import_lib!`
//!
//! 所有有成员方法的类都在独立的 `import_class!` 块中生成。
//! 关联函数（ctor/factory）在 `import_lib!` 中作为顶层自由函数输出，
//! 使用完整的 Rust 函数名（如 `counter_new`），以匹配 `main()` 中的调用方式。

use crate::ffi_model::{
    DynamicCastSpec, FfiSpec, FnBinding, ProxyFactorySpec, SelfKind, TemplateClassSpec,
    TemplateFactorySpec, TemplateFnSpec, TemplateInstanceSpec,
};

/// `CPP2RUST_GEN_TEMPLATES` 环境变量名 — v6 Phase B 模板骨架生成开关。
pub const GEN_TEMPLATES_ENV: &str = "CPP2RUST_GEN_TEMPLATES";

/// `CPP2RUST_GEN_PROXY` 环境变量名 — v6 Phase C `@make_proxy` 代理工厂骨架生成开关。
pub const GEN_PROXY_ENV: &str = "CPP2RUST_GEN_PROXY";

/// `CPP2RUST_GEN_DYNAMIC_CAST` 环境变量名 — v6 Phase C（续）`@dynamic_cast` 下行转换骨架生成开关。
pub const GEN_DYNAMIC_CAST_ENV: &str = "CPP2RUST_GEN_DYNAMIC_CAST";

/// 是否启用模板类 / 模板函数泛型骨架生成。
///
/// **默认关闭**：仅当 `CPP2RUST_GEN_TEMPLATES` 取值为 `1` / `true` / `yes` / `on`
/// （忽略大小写）时启用。关闭时生成器不输出任何模板相关内容，默认产物逐字节不变，
/// 符合 v6 方案「不改变现有使用方法」的硬约束。
pub fn templates_enabled() -> bool {
    env_switch_enabled(GEN_TEMPLATES_ENV)
}

/// 是否启用 `@make_proxy` 代理工厂骨架生成（v6 Phase C）。
///
/// **默认关闭**：仅当 `CPP2RUST_GEN_PROXY` 取值为 `1` / `true` / `yes` / `on`
/// （忽略大小写）时启用。关闭时生成器不输出任何代理工厂内容，默认产物逐字节不变。
pub fn proxy_enabled() -> bool {
    env_switch_enabled(GEN_PROXY_ENV)
}

/// 是否启用 `@dynamic_cast` 下行转换骨架生成（v6 Phase C（续））。
///
/// **默认关闭**：仅当 `CPP2RUST_GEN_DYNAMIC_CAST` 取值为 `1` / `true` / `yes` / `on`
/// （忽略大小写）时启用。关闭时生成器不输出任何下行转换内容，默认产物逐字节不变。
pub fn dynamic_cast_enabled() -> bool {
    env_switch_enabled(GEN_DYNAMIC_CAST_ENV)
}

/// 读取布尔型环境变量开关：取值为 `1` / `true` / `yes` / `on`（忽略大小写）时为 `true`。
fn env_switch_enabled(var: &str) -> bool {
    std::env::var(var)
        .map(|v| {
            let v = v.trim().to_ascii_lowercase();
            matches!(v.as_str(), "1" | "true" | "yes" | "on")
        })
        .unwrap_or(false)
}

/// 将 cpp! 内容行写入 `hicc::cpp! { ... }` 块字符串（供 generator 和 merger 共用）。
pub fn emit_cpp_block(lines: &[String]) -> String {
    let mut out = String::new();
    out.push_str("hicc::cpp! {\n");
    for line in lines {
        if line.is_empty() {
            out.push('\n');
        } else {
            out.push_str("    ");
            out.push_str(line);
            out.push('\n');
        }
    }
    out.push_str("}\n");
    out
}

/// 向 `out` 写出单个函数绑定行（`#[cpp(func = "...")]` 属性 + fn 签名）。
///
/// `ret_override`：若为 `Some(s)`，用 `s` 替换 `fb.ret_type` 作为返回类型（不含 ` -> ` 前缀）。
fn emit_fn_binding(out: &mut String, fb: &FnBinding, ret_override: Option<&str>) {
    out.push('\n');
    if fb.has_fn_ptr_param {
        out.push_str(
            "    // cpp2rust-todo[FP]: 含函数指针参数，需确保回调符合 extern \"C\" 调用约定\n",
        );
    }
    out.push_str(&format!("    #[cpp(func = \"{}\")]\n", fb.cpp_sig));
    let unsafe_kw = if fb.is_unsafe { "unsafe " } else { "" };
    let params_str = fb
        .params
        .iter()
        .map(|(n, t)| format!("{}: {}", n, t))
        .collect::<Vec<_>>()
        .join(", ");
    let ret_str = match ret_override {
        Some(t) => format!(" -> {}", t),
        None => match &fb.ret_type {
            Some(t) => format!(" -> {}", t),
            None => String::new(),
        },
    };
    out.push_str(&format!(
        "    {}fn {}({}){};\n",
        unsafe_kw, fb.rust_name, params_str, ret_str
    ));
}

/// 从 FfiSpec 生成三段式 hicc Rust FFI 代码字符串
pub fn generate(spec: &FfiSpec) -> String {
    let mut out = String::new();

    // 跨模块类型可见性：各 unit 文件通过 lib.rs 的 `pub use self::xxx::*` 重新导出，
    // 再经此 glob import 访问兄弟模块中定义的 hicc 类型（如 RapidJsonDocumentHandle）。
    out.push_str("#[allow(unused_imports)]\n");
    out.push_str("use crate::*;\n\n");

    // ── hicc::cpp! ─────────────────────────────
    out.push_str(&emit_cpp_block(&spec.cpp_block_lines));

    // ── hicc::import_class! (所有有方法的类都生成独立块) ────
    for cs in &spec.class_specs {
        // P2-1：跳过空块（无方法、无关联函数、且无 destroy 属性）
        if cs.is_empty() {
            continue;
        }
        out.push('\n');
        out.push_str("hicc::import_class! {\n");

        // P2-2：有析构函数优先用 #[cpp(class = "...", destroy = "...")]，
        // 无析构的纯虚接口用 #[interface]，其余用 #[cpp(class = "...")]
        if let Some(dtor) = &cs.destroy_fn {
            // P1-1：有析构函数时生成 destroy = "..."（即便是接口类也需要析构）
            out.push_str(&format!(
                "    #[cpp(class = \"{}\", destroy = \"{}\")]\n",
                cs.name, dtor
            ));
        } else if cs.is_interface {
            out.push_str("    #[interface]\n");
        } else {
            out.push_str(&format!("    #[cpp(class = \"{}\")]\n", cs.name));
        }

        if cs.methods.is_empty() {
            out.push_str(&format!("    pub class {} {{}}\n", cs.name));
        } else {
            out.push_str(&format!("    pub class {} {{\n", cs.name));
            let methods = &cs.methods;
            for (i, mb) in methods.iter().enumerate() {
                if mb.has_fn_ptr_param {
                    out.push_str("        // cpp2rust-todo[FP]: 含函数指针参数，需确保回调符合 extern \"C\" 调用约定\n");
                }
                out.push_str(&format!("        #[cpp(method = \"{}\")]\n", mb.cpp_sig));
                let self_ref = match mb.self_kind {
                    SelfKind::Ref => "&self",
                    SelfKind::RefMut => "&mut self",
                };
                let params_str = if mb.params.is_empty() {
                    String::new()
                } else {
                    let ps: Vec<String> = mb
                        .params
                        .iter()
                        .map(|(n, t)| format!("{}: {}", n, t))
                        .collect();
                    format!(", {}", ps.join(", "))
                };
                let ret_str = match &mb.ret_type {
                    Some(t) => format!(" -> {}", t),
                    None => String::new(),
                };
                out.push_str(&format!(
                    "        fn {}({}{}){};",
                    mb.rust_name, self_ref, params_str, ret_str
                ));
                out.push('\n');
                // 方法间插入空行，最后一个方法后不加
                if i + 1 < methods.len() {
                    out.push('\n');
                }
            }
            out.push_str("    }\n");
        }
        out.push_str("}\n");
    }

    // ── 模板类 import_class!（v6 Phase B，受开关控制）──
    let gen_templates = templates_enabled();
    if gen_templates {
        for tcs in &spec.template_classes {
            emit_template_class(&mut out, tcs);
        }
        // 模板实例化别名（v6 Phase B 增强）：紧跟泛型骨架之后输出，便于对照
        emit_template_instances(&mut out, &spec.template_instances);
    }

    // ── hicc::import_lib! ─────────────────────
    // 当没有任何绑定内容时（无可映射函数），跳过整个块
    let has_associated_fns = spec
        .class_specs
        .iter()
        .any(|cs| !cs.associated_fns.is_empty());
    let has_template_fns = gen_templates && !spec.template_functions.is_empty();
    let has_template_factories = gen_templates && !spec.template_factories.is_empty();
    let gen_proxy = proxy_enabled();
    let has_proxy_factories = gen_proxy && !spec.proxy_factories.is_empty();
    let gen_dynamic_cast = dynamic_cast_enabled();
    let has_dynamic_casts = gen_dynamic_cast && !spec.dynamic_casts.is_empty();
    if spec.lib_spec.fn_bindings.is_empty()
        && spec.lib_spec.fwd_decls.is_empty()
        && !has_associated_fns
        && !has_template_fns
        && !has_template_factories
        && !has_proxy_factories
        && !has_dynamic_casts
    {
        return out;
    }
    out.push('\n');
    out.push_str("hicc::import_lib! {\n");
    out.push_str(&format!(
        "    #![link_name = \"{}\"]\n",
        spec.lib_spec.link_name
    ));

    if !spec.lib_spec.fwd_decls.is_empty() {
        out.push('\n');
        for decl in &spec.lib_spec.fwd_decls {
            out.push_str(&format!("    {}\n", decl));
        }
    }

    // 关联函数（ctor/factory）作为顶层自由函数输出，保留完整 rust_name
    for cs in &spec.class_specs {
        for fb in &cs.associated_fns {
            // P1-2：ctor 若对应类有 destroy_fn，返回类型由 *mut Foo 改为 owned Foo
            let owned_ret = if cs.destroy_fn.is_some() {
                fb.ret_type.as_deref().map(|t| strip_mut_ptr(t, &cs.name))
            } else {
                None
            };
            emit_fn_binding(&mut out, fb, owned_ret.as_deref());
        }
    }

    // 无关联函数归属的独立全局函数
    for fb in &spec.lib_spec.fn_bindings {
        emit_fn_binding(&mut out, fb, None);
    }

    // 模板函数骨架（v6 Phase B，受开关控制）
    if has_template_fns {
        for tfs in &spec.template_functions {
            emit_template_fn(&mut out, tfs);
        }
    }

    // 模板实例化构造工厂骨架（v6 Phase B 增强（续），受开关控制）
    if has_template_factories {
        for tf in &spec.template_factories {
            emit_template_factory(&mut out, tf);
        }
    }

    // @make_proxy 代理工厂骨架（v6 Phase C，受 CPP2RUST_GEN_PROXY 开关控制）
    if has_proxy_factories {
        for pf in &spec.proxy_factories {
            emit_proxy_factory(&mut out, pf);
        }
    }

    // @dynamic_cast 下行转换骨架（v6 Phase C（续），受 CPP2RUST_GEN_DYNAMIC_CAST 开关控制）
    if has_dynamic_casts {
        for dc in &spec.dynamic_casts {
            emit_dynamic_cast(&mut out, dc);
        }
    }

    out.push_str("}\n");

    out
}

/// 输出单个模板类的泛型 `import_class!` 块（v6 Phase B 骨架）。
///
/// 形如：
/// ```text
/// hicc::import_class! {
///     // cpp2rust-todo[TMPL]: ...
///     #[cpp(class = "template<class T> Stack<T>")]
///     pub class Stack<T> {
///         #[cpp(method = "void push(T)")]
///         pub fn push(&mut self, value: T);
///     }
/// }
/// ```
fn emit_template_class(out: &mut String, tcs: &TemplateClassSpec) {
    // 无可映射的公有成员方法时不输出空骨架（与 import_class! 跳过空块的策略一致）
    if tcs.methods.is_empty() {
        return;
    }
    let params = tcs.type_params.join(", ");
    // C++ 模板类声明形式：template<class T, ...> Name<T, ...>
    let cpp_params = tcs
        .type_params
        .iter()
        .map(|p| format!("class {}", p))
        .collect::<Vec<_>>()
        .join(", ");
    let cpp_class = format!("template<{}> {}<{}>", cpp_params, tcs.name, params);

    out.push('\n');
    out.push_str("hicc::import_class! {\n");
    out.push_str(
        "    // cpp2rust-todo[TMPL]: 模板类泛型骨架，请按实际实例化类型校验签名与 AbiType 约束；\n",
    );
    out.push_str(
        "    // 构造函数/静态方法需在 import_lib! 中声明，复杂依赖类型（如 T::OutputRef）请手动补全。\n",
    );
    out.push_str(&format!("    #[cpp(class = \"{}\")]\n", cpp_class));
    out.push_str(&format!("    pub class {}<{}> {{\n", tcs.name, params));
    for (i, mb) in tcs.methods.iter().enumerate() {
        out.push_str(&format!("        #[cpp(method = \"{}\")]\n", mb.cpp_sig));
        let self_ref = match mb.self_kind {
            SelfKind::Ref => "&self",
            SelfKind::RefMut => "&mut self",
        };
        let params_str = if mb.params.is_empty() {
            String::new()
        } else {
            let ps: Vec<String> = mb
                .params
                .iter()
                .map(|(n, t)| format!("{}: {}", n, t))
                .collect();
            format!(", {}", ps.join(", "))
        };
        let ret_str = match &mb.ret_type {
            Some(t) => format!(" -> {}", t),
            None => String::new(),
        };
        out.push_str(&format!(
            "        pub fn {}({}{}){};\n",
            mb.rust_name, self_ref, params_str, ret_str
        ));
        if i + 1 < tcs.methods.len() {
            out.push('\n');
        }
    }
    out.push_str("    }\n");
    out.push_str("}\n");
}

/// 在 `import_lib!` 块内输出单个模板函数骨架（v6 Phase B）。
fn emit_template_fn(out: &mut String, tfs: &TemplateFnSpec) {
    out.push('\n');
    out.push_str(
        "    // cpp2rust-todo[TMPL]: 模板函数需按实例化类型声明（如 do_swap<int>(int*, int*)）；\n",
    );
    out.push_str("    // 下方 <T> 为泛型占位，请替换为实际实例化类型并确认安全性。\n");
    out.push_str(&format!("    #[cpp(func = \"{}\")]\n", tfs.cpp_sig));
    let params_str = tfs
        .params
        .iter()
        .map(|(n, t)| format!("{}: {}", n, t))
        .collect::<Vec<_>>()
        .join(", ");
    let ret_str = match &tfs.ret_type {
        Some(t) => format!(" -> {}", t),
        None => String::new(),
    };
    out.push_str(&format!(
        "    pub unsafe fn {}({}){};\n",
        tfs.rust_name, params_str, ret_str
    ));
}

/// 在 `import_lib!` 块内输出单个模板实例化构造工厂骨架（v6 Phase B 增强（续））。
///
/// 形如：
/// ```text
/// // cpp2rust-todo[TMPL]: StackI32 构造工厂骨架 —— 需在 C++ 侧提供对应符号并校验签名
/// #[cpp(func = "Stack<int>* stack_i32_new(int value)")]
/// pub unsafe fn stack_i32_new(value: i32) -> StackI32;
/// ```
fn emit_template_factory(out: &mut String, tf: &TemplateFactorySpec) {
    out.push('\n');
    out.push_str(&format!(
        "    // cpp2rust-todo[TMPL]: {} 构造工厂骨架 —— 需在 C++ 侧提供对应符号（如显式实例化/包装）并校验签名\n",
        tf.alias_name
    ));
    out.push_str(&format!("    #[cpp(func = \"{}\")]\n", tf.cpp_sig));
    let params_str = tf
        .params
        .iter()
        .map(|(n, t)| format!("{}: {}", n, t))
        .collect::<Vec<_>>()
        .join(", ");
    out.push_str(&format!(
        "    pub unsafe fn {}({}) -> {};\n",
        tf.rust_name, params_str, tf.alias_name
    ));
}

/// 在 `import_lib!` 块内输出单个 `@make_proxy` 代理工厂骨架（v6 Phase C）。
///
/// 形如：
/// ```text
/// // cpp2rust-todo[PROXY]: @make_proxy 工厂骨架 —— 使 Rust 侧可实现 C++ 接口 Bar；
/// // 需确认构造函数参数类型列表与 @make_proxy 一致，Rust 实现类经 hicc::Interface<Baz> 传入。
/// #[cpp(func = "Baz @make_proxy<Baz>()")]
/// #[interface(name = "Bar")]
/// fn new_rust_baz(intf: hicc::Interface<Baz>) -> Baz;
/// ```
fn emit_proxy_factory(out: &mut String, pf: &ProxyFactorySpec) {
    out.push('\n');
    out.push_str(&format!(
        "    // cpp2rust-todo[PROXY]: @make_proxy 工厂骨架 —— 使 Rust 侧可实现 C++ 接口 {}；\n",
        pf.interface_name
    ));
    out.push_str(&format!(
        "    // 需确认构造函数参数类型列表与 @make_proxy 一致，Rust 实现类经 hicc::Interface<{}> 传入。\n",
        pf.concrete_class
    ));
    out.push_str(&format!("    #[cpp(func = \"{}\")]\n", pf.cpp_sig));
    out.push_str(&format!(
        "    #[interface(name = \"{}\")]\n",
        pf.interface_name
    ));
    // 第一个参数固定为 Rust 实现类（hicc::Interface<具体类>），其后为构造函数参数
    let mut all_params: Vec<String> = vec![format!("intf: hicc::Interface<{}>", pf.concrete_class)];
    all_params.extend(pf.params.iter().map(|(n, t)| format!("{}: {}", n, t)));
    out.push_str(&format!(
        "    fn {}({}) -> {};\n",
        pf.rust_name,
        all_params.join(", "),
        pf.concrete_class
    ));
}

/// 在 `import_lib!` 块内输出单个 `@dynamic_cast` 下行转换骨架（v6 Phase C（续））。
///
/// 形如：
/// ```text
/// // cpp2rust-todo[DCAST]: @dynamic_cast 下行转换骨架 —— 多态基类 Foo 向下转换为派生类 Bar；
/// // 转换失败返回空指针，调用方需判空（is_null）。RTTI 要求源类型为多态类型（含虚函数）。
/// #[cpp(func = "const Bar* @dynamic_cast<const Bar*>(const Foo*)")]
/// pub unsafe fn dynamic_cast_foo_to_bar(src: *const Foo) -> *const Bar;
/// ```
fn emit_dynamic_cast(out: &mut String, dc: &DynamicCastSpec) {
    out.push('\n');
    out.push_str(&format!(
        "    // cpp2rust-todo[DCAST]: @dynamic_cast 下行转换骨架 —— 多态基类 {} 向下转换为派生类 {}；\n",
        dc.src_class, dc.dst_class
    ));
    out.push_str(
        "    // 转换失败返回空指针，调用方需判空（is_null）。RTTI 要求源类型为多态类型（含虚函数）。\n",
    );
    out.push_str(&format!("    #[cpp(func = \"{}\")]\n", dc.cpp_sig));
    out.push_str(&format!(
        "    pub unsafe fn {}(src: *const {}) -> *const {};\n",
        dc.rust_name, dc.src_class, dc.dst_class
    ));
}

/// 输出模板实例化别名骨架（v6 Phase B 增强）。
///
/// 形如：
/// ```text
/// // cpp2rust-todo[TMPL]: 模板实例化别名 —— 请确认实参类型与 AbiType 约束
/// pub type StackI32 = Stack<hicc::Pod<i32>>;
/// pub type StackF64 = Stack<hicc::Pod<f64>>;
/// ```
///
/// 别名是普通 Rust 类型别名，需与对应的泛型模板类骨架（`emit_template_class`）配合使用；
/// 因此仅在 `CPP2RUST_GEN_TEMPLATES` 开启时输出，默认产物逐字节不变。
fn emit_template_instances(out: &mut String, instances: &[TemplateInstanceSpec]) {
    if instances.is_empty() {
        return;
    }
    out.push('\n');
    out.push_str(
        "// cpp2rust-todo[TMPL]: 以下为模板实例化别名骨架，请确认实参类型与 AbiType 约束；\n",
    );
    out.push_str(
        "// POD 标量已用 hicc::Pod 包装，类类型实参需替换为对应的 hicc 类（如 hicc_std::string）。\n",
    );
    for inst in instances {
        if inst.needs_class_type {
            out.push_str(&format!(
                "// cpp2rust-todo[TMPL]: {} 含类类型实参，请将其替换为对应的 hicc 类型\n",
                inst.alias_name
            ));
        }
        out.push_str(&format!(
            "pub type {} = {}<{}>;\n",
            inst.alias_name,
            inst.template_name,
            inst.hicc_args.join(", ")
        ));
    }
}

/// P1-2 辅助：若返回类型是 `*mut ClassName`，去掉指针返回 `ClassName`（owned）
fn strip_mut_ptr(ret_type: &str, class_name: &str) -> String {
    let expected = format!("*mut {}", class_name);
    if ret_type.trim() == expected.trim() {
        class_name.to_string()
    } else {
        ret_type.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ffi_model::{ClassSpec, FfiSpec, FnBinding, LibSpec, MethodBinding, SelfKind};

    fn make_fn_binding(name: &str, has_fn_ptr_param: bool) -> FnBinding {
        FnBinding {
            cpp_sig: format!("void {}()", name),
            rust_name: name.to_string(),
            params: vec![],
            ret_type: None,
            is_unsafe: has_fn_ptr_param,
            has_fn_ptr_param,
        }
    }

    fn make_method_binding(name: &str, has_fn_ptr_param: bool) -> MethodBinding {
        MethodBinding {
            cpp_sig: format!("void {}()", name),
            rust_name: name.to_string(),
            self_kind: SelfKind::RefMut,
            params: vec![],
            ret_type: None,
            has_fn_ptr_param,
        }
    }

    fn make_spec_with_fn(fb: FnBinding) -> FfiSpec {
        FfiSpec {
            unit_name: "test".to_string(),
            cpp_block_lines: vec!["#include <test.h>".to_string()],
            class_specs: vec![],
            lib_spec: LibSpec {
                link_name: "test".to_string(),
                fwd_decls: vec![],
                fn_bindings: vec![fb],
            },
            ..Default::default()
        }
    }

    /// 含函数指针参数的函数绑定应生成 cpp2rust-todo[FP] 注释
    #[test]
    fn generate_fn_binding_with_fp_emits_todo_comment() {
        let fb = make_fn_binding("apply_op", true);
        let spec = make_spec_with_fn(fb);
        let code = generate(&spec);
        assert!(
            code.contains("// cpp2rust-todo[FP]:"),
            "含函数指针参数的函数绑定应生成 cpp2rust-todo[FP] 注释，实际输出：\n{}",
            code
        );
    }

    /// 不含函数指针的函数绑定不应生成 cpp2rust-todo[FP] 注释
    #[test]
    fn generate_without_fp_no_todo_comment() {
        let fb = make_fn_binding("get_value", false);
        let spec = make_spec_with_fn(fb);
        let code = generate(&spec);
        assert!(
            !code.contains("// cpp2rust-todo[FP]:"),
            "不含函数指针的函数绑定不应生成 cpp2rust-todo[FP] 注释，实际输出：\n{}",
            code
        );
    }

    /// 含函数指针参数的方法绑定应生成 cpp2rust-todo[FP] 注释
    #[test]
    fn generate_method_with_fp_emits_todo_comment() {
        let mb = make_method_binding("set_handler", true);
        let spec = FfiSpec {
            unit_name: "test".to_string(),
            cpp_block_lines: vec!["#include <test.h>".to_string()],
            class_specs: vec![ClassSpec {
                name: "MyClass".to_string(),
                methods: vec![mb],
                associated_fns: vec![],
                destroy_fn: None,
                is_interface: false,
            }],
            lib_spec: LibSpec {
                link_name: "test".to_string(),
                fwd_decls: vec![],
                fn_bindings: vec![],
            },
            ..Default::default()
        };
        let code = generate(&spec);
        assert!(
            code.contains("// cpp2rust-todo[FP]:"),
            "含函数指针参数的方法绑定应生成 cpp2rust-todo[FP] 注释，实际输出：\n{}",
            code
        );
    }

    /// 生成的 unit 文件应以 `use crate::*;` 开头，使跨模块类型可见
    #[test]
    fn generate_includes_crate_glob_import() {
        let fb = make_fn_binding("foo", false);
        let spec = make_spec_with_fn(fb);
        let code = generate(&spec);
        assert!(
            code.contains("use crate::*;"),
            "生成代码应包含 `use crate::*;` 以允许跨模块类型引用，实际输出：\n{}",
            code
        );
        assert!(
            code.contains("#[allow(unused_imports)]"),
            "生成代码应包含 `#[allow(unused_imports)]`，实际输出：\n{}",
            code
        );
    }
}
