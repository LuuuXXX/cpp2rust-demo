//! hicc Rust 代码生成器（Phase 5）
//!
//! 将 `FfiSpec` 生成三段式 hicc Rust 代码：
//! `hicc::cpp!` + `hicc::import_class!`（有成员方法的类）+ `hicc::import_lib!`
//!
//! 所有有成员方法的类都在独立的 `import_class!` 块中生成。
//! 关联函数（ctor/factory）在 `import_lib!` 中作为顶层自由函数输出，
//! 使用完整的 Rust 函数名（如 `counter_new`），以匹配 `main()` 中的调用方式。

use crate::ffi_model::{FfiSpec, FnBinding, SelfKind};

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

    // ── hicc::cpp! ─────────────────────────────
    out.push_str(&emit_cpp_block(&spec.cpp_block_lines));

    // ── hicc::import_class! (所有有方法的类都生成独立块) ────
    for cs in &spec.class_specs {
        // P2-1：跳过空块（无方法、无关联函数、且无 destroy 属性）
        if cs.methods.is_empty() && cs.associated_fns.is_empty() && cs.destroy_fn.is_none() {
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
            out.push_str(&format!("    class {} {{}}\n", cs.name));
        } else {
            out.push_str(&format!("    class {} {{\n", cs.name));
            for mb in &cs.methods {
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
                out.push('\n');
            }
            // 去掉最后一个方法后多余的空行
            if out.ends_with("\n\n") {
                out.pop();
            }
            out.push_str("    }\n");
        }
        out.push_str("}\n");
    }

    // ── hicc::import_lib! ─────────────────────
    // 当没有任何绑定内容时（无可映射函数），跳过整个块
    let has_associated_fns = spec
        .class_specs
        .iter()
        .any(|cs| !cs.associated_fns.is_empty());
    if spec.lib_spec.fn_bindings.is_empty()
        && spec.lib_spec.fwd_decls.is_empty()
        && !has_associated_fns
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

    out.push_str("}\n");

    out
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
