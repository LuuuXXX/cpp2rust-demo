//! hicc Rust 代码生成器（Phase 5）
//!
//! 将 `FfiSpec` 生成三段式 hicc Rust 代码：
//! `hicc::cpp!` + `hicc::import_class!`（有成员方法的类）+ `hicc::import_lib!`
//!
//! 所有有成员方法的类都在独立的 `import_class!` 块中生成。
//! 关联函数（ctor/dtor/factory）在 `import_lib!` 中作为顶层自由函数输出，
//! 使用完整的 Rust 函数名（如 `counter_new`），以匹配 `main()` 中的调用方式。

use crate::ffi_model::{FfiSpec, SelfKind};

/// 从 FfiSpec 生成三段式 hicc Rust FFI 代码字符串
pub fn generate(spec: &FfiSpec) -> String {
    let mut out = String::new();

    // ── hicc::cpp! ─────────────────────────────
    out.push_str("hicc::cpp! {\n");
    for line in &spec.cpp_block_lines {
        if line.is_empty() {
            out.push('\n');
        } else {
            out.push_str("    ");
            out.push_str(line);
            out.push('\n');
        }
    }
    out.push_str("}\n");

    // ── hicc::import_class! (所有有方法的类都生成独立块) ────
    for cs in &spec.class_specs {
        out.push('\n');
        out.push_str("hicc::import_class! {\n");
        out.push_str(&format!("    #[cpp(class = \"{}\")]\n", cs.name));
        out.push_str(&format!("    class {} {{\n", cs.name));
        for mb in &cs.methods {
            out.push_str(&format!(
                "        #[cpp(method = \"{}\")]\n",
                mb.cpp_sig
            ));
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
        out.push_str("}\n");
    }

    // ── hicc::import_lib! ─────────────────────
    // 当没有任何绑定内容时（void* opaque 模式等），跳过整个块
    let has_associated_fns = spec.class_specs.iter().any(|cs| !cs.associated_fns.is_empty());
    if spec.lib_spec.fn_bindings.is_empty()
        && spec.lib_spec.fwd_decls.is_empty()
        && !has_associated_fns
    {
        return out;
    }
    out.push('\n');
    out.push_str("hicc::import_lib! {\n");
    out.push_str(&format!("    #![link_name = \"{}\"]\n", spec.lib_spec.link_name));

    if !spec.lib_spec.fwd_decls.is_empty() {
        out.push('\n');
        for decl in &spec.lib_spec.fwd_decls {
            out.push_str(&format!("    {}\n", decl));
        }
    }

    // 关联函数（ctor/dtor/factory）作为顶层自由函数输出，保留完整 rust_name
    for cs in &spec.class_specs {
        for fb in &cs.associated_fns {
            out.push('\n');
            out.push_str(&format!("    #[cpp(func = \"{}\")]\n", fb.cpp_sig));
            let unsafe_kw = if fb.is_unsafe { "unsafe " } else { "" };
            let params_str = fb
                .params
                .iter()
                .map(|(n, t)| format!("{}: {}", n, t))
                .collect::<Vec<_>>()
                .join(", ");
            let ret_str = match &fb.ret_type {
                Some(t) => format!(" -> {}", t),
                None => String::new(),
            };
            out.push_str(&format!(
                "    {}fn {}({}){};\n",
                unsafe_kw, fb.rust_name, params_str, ret_str
            ));
        }
    }

    // 无关联函数归属的独立全局函数
    for fb in &spec.lib_spec.fn_bindings {
        out.push('\n');
        out.push_str(&format!("    #[cpp(func = \"{}\")]\n", fb.cpp_sig));

        let unsafe_kw = if fb.is_unsafe { "unsafe " } else { "" };
        let params_str = fb
            .params
            .iter()
            .map(|(n, t)| format!("{}: {}", n, t))
            .collect::<Vec<_>>()
            .join(", ");
        let ret_str = match &fb.ret_type {
            Some(t) => format!(" -> {}", t),
            None => String::new(),
        };
        out.push_str(&format!(
            "    {}fn {}({}){};\n",
            unsafe_kw, fb.rust_name, params_str, ret_str
        ));
    }

    out.push_str("}\n");

    out
}
