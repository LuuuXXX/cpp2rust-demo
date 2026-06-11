//! hicc Rust 代码生成器（Phase 5）
//!
//! 将 `FfiSpec` 生成三段式 hicc Rust 代码：
//! `hicc::cpp!` + `hicc::import_class!`（有成员方法的类）+ `hicc::import_lib!`
//!
//! 所有有成员方法的类都在独立的 `import_class!` 块中生成。
//! 关联函数（ctor/factory）在 `import_lib!` 中作为顶层自由函数输出，
//! 使用完整的 Rust 函数名（如 `counter_new`），以匹配 `main()` 中的调用方式。

use crate::ffi_model::{FfiSpec, FnBinding, SelfKind, TemplateClassSpec, TemplateFnSpec};

/// 控制是否生成模板（泛型）绑定骨架的环境变量。
///
/// 默认**关闭**：未设置或设为 `0`/`false`/`no`/`off`（忽略大小写）时不生成模板块，
/// 从而保证默认生成产物逐字节不变（不触动既有 L1 黄金 / L2 编译基线）。
/// 仅当显式设为其他真值（如 `1`/`true`/`on`）时，才追加泛型 `import_class!` /
/// `import_lib!` 骨架。后续阶段可在示例中开启以演示模板原生映射。
pub const GEN_TEMPLATES_ENV: &str = "CPP2RUST_GEN_TEMPLATES";

/// 依 [`GEN_TEMPLATES_ENV`] 判断是否生成模板绑定骨架（默认关闭）。
pub fn templates_enabled() -> bool {
    match std::env::var(GEN_TEMPLATES_ENV) {
        Ok(v) => matches!(
            v.trim().to_ascii_lowercase().as_str(),
            "1" | "true" | "yes" | "on"
        ),
        Err(_) => false,
    }
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
        out.push_str(&emit_template_blocks(spec));
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

    out.push_str(&emit_template_blocks(spec));

    out
}

/// 生成模板（泛型）绑定骨架字符串。
///
/// 仅当 [`templates_enabled`] 为真且存在模板规格时返回非空内容；否则返回空串，
/// 保证默认产物不受影响。生成内容包含：
/// - 每个模板类一个泛型 `hicc::import_class!` 块（`pub class Name<T> { ... }`）；
/// - 一个集中的 `hicc::import_lib!` 块声明模板函数的泛型签名。
///
/// 由于具体实例化类型（如 `Stack<hicc::Pod<i32>>`）尚需后续阶段补充，每个块均带
/// `cpp2rust-todo[TPL]` 提示，符合既有降级标记约定。
fn emit_template_blocks(spec: &FfiSpec) -> String {
    if !templates_enabled() {
        return String::new();
    }
    let classes: Vec<&TemplateClassSpec> = spec
        .template_classes
        .iter()
        .filter(|c| !c.is_empty())
        .collect();
    if classes.is_empty() && spec.template_fns.is_empty() {
        return String::new();
    }

    let mut out = String::new();

    for cs in classes {
        let generics = format!("<{}>", cs.type_params.join(", "));
        out.push('\n');
        out.push_str("hicc::import_class! {\n");
        out.push_str(
            "    // cpp2rust-todo[TPL]: 泛型骨架；请在 import_lib! 中以具体实例化类型（如 hicc::Pod<i32>）声明别名与工厂\n",
        );
        out.push_str(&format!("    #[cpp(class = \"{}\")]\n", cs.name));
        out.push_str(&format!("    pub class {}{} {{\n", cs.name, generics));
        for (i, mb) in cs.methods.iter().enumerate() {
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
                "        fn {}({}{}){};\n",
                mb.rust_name, self_ref, params_str, ret_str
            ));
            if i + 1 < cs.methods.len() {
                out.push('\n');
            }
        }
        out.push_str("    }\n");
        out.push_str("}\n");
    }

    if !spec.template_fns.is_empty() {
        out.push('\n');
        out.push_str("hicc::import_lib! {\n");
        out.push_str(&format!(
            "    #![link_name = \"{}\"]\n",
            spec.lib_spec.link_name
        ));
        for tf in &spec.template_fns {
            emit_template_fn(&mut out, tf);
        }
        out.push_str("}\n");
    }

    out
}

/// 写出单个模板函数的泛型签名绑定（带 `cpp2rust-todo[TPL]` 提示）。
fn emit_template_fn(out: &mut String, tf: &TemplateFnSpec) {
    out.push('\n');
    out.push_str(
        "    // cpp2rust-todo[TPL]: 模板函数泛型签名；请替换为具体实例化类型（如 do_swap<int>）\n",
    );
    out.push_str(&format!("    #[cpp(func = \"{}\")]\n", tf.cpp_sig));
    let params_str = tf
        .params
        .iter()
        .map(|(n, t)| format!("{}: {}", n, t))
        .collect::<Vec<_>>()
        .join(", ");
    let ret_str = match &tf.ret_type {
        Some(t) => format!(" -> {}", t),
        None => String::new(),
    };
    out.push_str(&format!(
        "    fn {}({}){};\n",
        tf.rust_name, params_str, ret_str
    ));
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
