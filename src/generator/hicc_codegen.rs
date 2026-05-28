//! hicc Rust 代码生成器（Phase 5）
//!
//! 将 `FfiSpec` 生成三段式 hicc Rust 代码：
//! `hicc::cpp!` + `hicc::import_class!`（无关联函数的类）+ `hicc::import_lib!`
//!
//! 对于有关联函数（ctor/dtor/factory）的类，使用 hicc v0.2.4 class body 语法，
//! 在 `import_lib!` 中生成 `class ClassName { methods... associated_fns... }` 格式，
//! 并省略该类的独立 `import_class!` 块。

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

    // ── hicc::import_class! (只为无关联函数的类生成) ────
    // 有关联函数的类使用 import_lib! class body 格式，不需要独立的 import_class! 块
    for cs in &spec.class_specs {
        if !cs.associated_fns.is_empty() {
            continue; // 这些类会合并进 import_lib! class body
        }
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
    let has_class_bodies = spec.class_specs.iter().any(|cs| !cs.associated_fns.is_empty());
    if spec.lib_spec.fn_bindings.is_empty()
        && spec.lib_spec.fwd_decls.is_empty()
        && !has_class_bodies
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

    // 有关联函数的类：生成 class body（含方法 + 关联函数）
    for cs in &spec.class_specs {
        if cs.associated_fns.is_empty() {
            continue;
        }
        out.push('\n');
        out.push_str(&format!("    #[cpp(class = \"{}\")]\n", cs.name));
        out.push_str(&format!("    class {} {{\n", cs.name));

        // 先输出 methods（有 self 的成员方法）
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

        // 再输出关联函数（无 self 的 ctor/dtor/factory）
        for fb in &cs.associated_fns {
            out.push_str(&format!("        #[cpp(func = \"{}\")]\n", fb.cpp_sig));
            let fn_name = strip_class_prefix(&fb.rust_name, &cs.name);
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
                "        {}fn {}({}){};\n",
                unsafe_kw, fn_name, params_str, ret_str
            ));
            out.push('\n');
        }

        // 去掉最后一个条目后多余的空行
        if out.ends_with("\n\n") {
            out.pop();
        }
        out.push_str("    }\n");
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

/// 从函数的 rust_name 中去掉类名前缀（snake_case 小写），使其成为类关联函数名。
///
/// 例：`counter_new` + class `Counter` → `new`
///     `point_new_xy` + class `Point` → `new_xy`
///     `buffer_new_copy` + class `Buffer` → `new_copy`
fn strip_class_prefix<'a>(rust_name: &'a str, class_name: &str) -> &'a str {
    let prefix = format!("{}_", class_name.to_lowercase());
    rust_name.strip_prefix(&prefix).unwrap_or(rust_name)
}
