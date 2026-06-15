//! 冒烟测试生成器（Phase D）
//!
//! 由 `init` 在 `.cpp2rust/<feature>/rust/tests/smoke.rs` 生成冒烟测试，
//! 用于"生成即验证"：确认生成的 Rust FFI 绑定能够**编译**、**链接**并**可调用**。
//!
//! ## 设计来源
//!
//! 参考 hicc 自身的验证方式（`hicc-std/src/std_test/*`）——测试与 FFI 绑定位于同一
//! crate，通过 `cargo test` 链接 C++ 静态库并真实使用绑定，验证 ABI 与基本可用性。
//!
//! ## 生成策略
//!
//! 对每个 FfiSpec 中的可测试项生成独立 `#[test]` 函数：
//!
//! 1. **工厂函数**（associated_fns，如 `counter_new()`）：零参数时生成调用测试
//! 2. **类方法**（methods）：所在类有零参构造函数时，构造实例并调用方法
//! 3. **全局函数**（lib_spec.fn_bindings）：零参数时生成调用测试
//! 4. **编译期类型断言**：所有 `pub class` 类型
//! 5. 无法自动生成测试的函数以 `cpp2rust-todo[SMOKE]` 注释列出

use crate::ffi_model::{FfiSpec, FnBinding};

/// 冒烟测试文件相对生成项目根目录的路径（用于生成与用户提示保持一致）。
pub const SMOKE_TEST_PATH: &str = "tests/smoke.rs";

// ─────────────────────────────────────────────────────────────────
//  辅助函数
// ─────────────────────────────────────────────────────────────────

/// 收集所有会经 `lib.rs` 重导出的 `pub class` 类型名。
fn collect_pub_class_names(specs: &[&FfiSpec]) -> Vec<String> {
    let mut names: Vec<String> = Vec::new();
    for spec in specs {
        for cs in &spec.class_specs {
            if cs.is_empty() {
                continue;
            }
            if !names.contains(&cs.name) {
                names.push(cs.name.clone());
            }
        }
    }
    names
}

/// 收集所有 FFI 函数名（工厂函数、方法、全局函数），用于 SMOKE 占位说明。
fn collect_all_fn_names(specs: &[&FfiSpec]) -> Vec<String> {
    let mut names: Vec<String> = Vec::new();
    for spec in specs {
        for cs in &spec.class_specs {
            for fb in &cs.associated_fns {
                if !names.contains(&fb.rust_name) {
                    names.push(fb.rust_name.clone());
                }
            }
            for mb in &cs.methods {
                if !names.contains(&mb.rust_name) {
                    names.push(mb.rust_name.clone());
                }
            }
        }
        for fb in &spec.lib_spec.fn_bindings {
            if !names.contains(&fb.rust_name) {
                names.push(fb.rust_name.clone());
            }
        }
    }
    names
}

/// 判断函数是否为零参数。
fn is_zero_param(fb: &FnBinding) -> bool {
    fb.params.is_empty()
}

/// 查找某类的零参构造/工厂函数。
fn find_zero_param_factory<'a>(specs: &[&'a FfiSpec], class_name: &str) -> Option<&'a FnBinding> {
    for spec in specs {
        for cs in &spec.class_specs {
            if cs.name == class_name {
                for fb in &cs.associated_fns {
                    if is_zero_param(fb) {
                        return Some(fb);
                    }
                }
            }
        }
    }
    None
}

/// 为 Rust 原始类型返回默认值字面量（用于 smoke 测试的 assert_eq! 断言）。
///
/// 仅返回**零值/默认值**可确定的原始类型；其他类型（类、字符串、指针、自定义类型）
/// 返回 `None`，由调用方决定如何处理（通常保留 `let _result = ...`）。
///
/// 参考 hicc-usages/examples/006_class_basic/rust_hicc/tests/smoke.rs 中的
/// `assert_eq!(c.count(), 0)` 风格断言。
fn default_value_literal(rust_type: &str) -> Option<&'static str> {
    match rust_type.trim() {
        "i8" | "i16" | "i32" | "i64" | "isize" | "u8" | "u16" | "u32" | "u64" | "usize" => {
            Some("0")
        }
        "f32" | "f64" => Some("0.0"),
        "bool" => Some("false"),
        _ => None,
    }
}

// ─────────────────────────────────────────────────────────────────
//  生成主函数
// ─────────────────────────────────────────────────────────────────

/// 生成 `tests/smoke.rs` 的完整内容。
///
/// `lib_name` 为生成 crate 的库名（`feature` 中的 `-` 已替换为 `_`）。
/// `specs` 为各编译单元的 FFI 规格。
pub fn generate_smoke_test(lib_name: &str, specs: &[&FfiSpec]) -> String {
    let class_names = collect_pub_class_names(specs);
    let all_fn_names = collect_all_fn_names(specs);

    let mut out = String::new();
    out.push_str("//! 由 cpp2rust-demo 自动生成的冒烟测试（init 阶段）。\n");
    out.push_str("//!\n");
    out.push_str("//! 目的：验证生成的 Rust FFI 绑定可编译、可链接、可调用。\n");
    out.push_str("//! 运行：在本目录执行 `cargo test`。\n");
    out.push_str("\n#![allow(unused_imports, unused_variables, unused_mut)]\n\n");
    out.push_str(&format!("use {}::*;\n\n", lib_name));

    // ── A. 编译期类型断言 ──────────────────────────────────────
    if !class_names.is_empty() {
        out.push_str("// ── 编译期类型断言：所有 FFI pub class 类型可用 ──\n\n");
        out.push_str("/// 编译期断言：所有生成的 FFI 类型均可用。\n");
        out.push_str("#[test]\n");
        out.push_str("fn smoke_ffi_types_available() {\n");
        out.push_str("    fn assert_type_available<T>() {}\n");
        for name in &class_names {
            out.push_str(&format!("    assert_type_available::<{}>();\n", name));
        }
        out.push_str("}\n\n");
    }

    // ── B. 工厂函数测试（零参构造函数）──────────────────────────
    let mut tested_fns: Vec<String> = Vec::new();
    let mut factory_count: usize = 0;

    for spec in specs {
        for cs in &spec.class_specs {
            if cs.is_empty() {
                continue;
            }
            for fb in &cs.associated_fns {
                if tested_fns.contains(&fb.rust_name) {
                    continue;
                }
                tested_fns.push(fb.rust_name.clone());

                if is_zero_param(fb) {
                    out.push_str(&format!("#[test]\nfn smoke_{}() {{\n", fb.rust_name));
                    if fb.is_unsafe {
                        out.push_str("    unsafe {\n");
                        out.push_str(&format!("        let _obj = {}();\n", fb.rust_name));
                        out.push_str("    }\n");
                    } else {
                        out.push_str(&format!("    let _obj = {}();\n", fb.rust_name));
                    }
                    out.push_str("}\n\n");
                    factory_count += 1;
                }
            }
        }
    }

    // ── C. 类方法测试（有零参构造函数的类）──────────────────────
    // 增强（参考 hicc-usages/006_class_basic/rust_hicc/tests/smoke.rs）：
    // 当方法返回原始类型（i32/u32/bool/f64 等）时，对返回值生成 `assert_eq!(result, <default>)` 断言，
    // 验证新构造实例的默认状态（如 `assert_eq!(c.count(), 0)`）。
    // 其他返回类型（类、指针、字符串）无法预知默认值，保留 `let _result = ...` 形式。

    for spec in specs {
        for cs in &spec.class_specs {
            if cs.methods.is_empty() {
                continue;
            }
            let factory = find_zero_param_factory(specs, &cs.name);
            let factory_name = match factory {
                Some(fb) => fb.rust_name.as_str(),
                None => continue, // 无构造函数则跳过该类方法
            };
            let factory_is_unsafe = factory.map(|f| f.is_unsafe).unwrap_or(false);

            for mb in &cs.methods {
                if tested_fns.contains(&mb.rust_name) {
                    continue;
                }
                // 有额外参数的方法无法自动测试，跳过
                if !mb.params.is_empty() {
                    continue;
                }
                tested_fns.push(mb.rust_name.clone());

                out.push_str(&format!("#[test]\nfn smoke_{}() {{\n", mb.rust_name));

                // 构造实例
                if factory_is_unsafe {
                    out.push_str("    let obj = unsafe { ");
                    out.push_str(&format!("{}() }};\n", factory_name));
                } else {
                    out.push_str(&format!("    let obj = {}();\n", factory_name));
                }

                // 可变性
                let is_mut = mb.self_kind == crate::ffi_model::SelfKind::RefMut;
                if is_mut {
                    out.push_str("    let mut obj = obj;\n");
                }

                // 调用方法，对原始类型返回值生成 assert 断言
                let method_call = format!("obj.{}()", mb.rust_name);
                match &mb.ret_type {
                    Some(rt) => {
                        if let Some(literal) = default_value_literal(rt) {
                            // 原始类型：assert_eq!(result, <default>)
                            out.push_str(&format!("    let result = {};\n", method_call));
                            out.push_str(&format!("    assert_eq!(result, {});\n", literal));
                        } else {
                            // 类/指针/字符串等：保留 _result 形式
                            out.push_str(&format!("    let _result = {};\n", method_call));
                        }
                    }
                    None => {
                        out.push_str(&format!("    {};\n", method_call));
                    }
                }
                out.push_str("}\n\n");
            }
        }
    }

    // ── D. 全局函数测试（零参函数）──────────────────────────────
    let mut global_count: usize = 0;

    for spec in specs {
        for fb in &spec.lib_spec.fn_bindings {
            if tested_fns.contains(&fb.rust_name) {
                continue;
            }

            if !is_zero_param(fb) {
                continue;
            }
            tested_fns.push(fb.rust_name.clone());

            out.push_str(&format!("#[test]\nfn smoke_{}() {{\n", fb.rust_name));
            if fb.is_unsafe {
                out.push_str("    unsafe {\n");
                match &fb.ret_type {
                    Some(_) => {
                        out.push_str(&format!("        let _result = {}();\n", fb.rust_name))
                    }
                    None => out.push_str(&format!("        {}();\n", fb.rust_name)),
                }
                out.push_str("    }\n");
            } else {
                match &fb.ret_type {
                    Some(_) => out.push_str(&format!("    let _result = {}();\n", fb.rust_name)),
                    None => out.push_str(&format!("    {}();\n", fb.rust_name)),
                }
            }
            out.push_str("}\n\n");
            global_count += 1;
        }
    }

    // ── E. 纯链接测试（兜底）───────────────────────────────────
    if class_names.is_empty() && factory_count == 0 && global_count == 0 {
        out.push_str("/// 最小冒烟测试：确保生成的 crate 能编译并链接进测试二进制。\n");
        out.push_str("#[test]\n");
        out.push_str("fn smoke_crate_links() {\n");
        out.push_str("    assert!(true);\n");
        out.push_str("}\n");
    }

    // ── F. SMOKE 占位说明（无法自动测试的函数）──────────────────
    // 收集尚未测试的函数名
    let untested: Vec<String> = all_fn_names
        .iter()
        .filter(|n| !tested_fns.contains(n))
        .cloned()
        .collect();

    if !untested.is_empty() {
        out.push('\n');
        out.push_str("// cpp2rust-todo[SMOKE]: 以下函数含非平凡参数，无法自动生成调用测试。\n");
        out.push_str(
            "// 如需补充行为断言，请在对应函数声明中将参数替换为可构造的默认值后手动测试。\n",
        );
        out.push_str("// 待补充行为断言的函数：\n");
        for name in &untested {
            out.push_str(&format!("//   - {}\n", name));
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ffi_model::{ClassSpec, FfiSpec, FnBinding, LibSpec, MethodBinding, SelfKind};

    fn class_with_method(name: &str) -> ClassSpec {
        ClassSpec {
            name: name.to_string(),
            methods: vec![MethodBinding {
                cpp_sig: "int get() const".to_string(),
                rust_name: "get".to_string(),
                self_kind: SelfKind::Ref,
                params: vec![],
                ret_type: Some("i32".to_string()),
                has_fn_ptr_param: false,
            }],
            associated_fns: vec![],
            destroy_fn: None,
            is_interface: false,
        }
    }

    fn class_with_factory(name: &str) -> ClassSpec {
        ClassSpec {
            name: name.to_string(),
            methods: vec![MethodBinding {
                cpp_sig: "int value() const".to_string(),
                rust_name: "value".to_string(),
                self_kind: SelfKind::Ref,
                params: vec![],
                ret_type: Some("i32".to_string()),
                has_fn_ptr_param: false,
            }],
            associated_fns: vec![FnBinding {
                cpp_sig: format!("{}* {}_new()", name, name.to_lowercase()),
                rust_name: format!("{}_new", name.to_lowercase()),
                params: vec![],
                ret_type: Some(name.to_string()),
                is_unsafe: false,
                has_fn_ptr_param: false,
            }],
            destroy_fn: None,
            is_interface: false,
        }
    }

    fn empty_class(name: &str) -> ClassSpec {
        ClassSpec {
            name: name.to_string(),
            methods: vec![],
            associated_fns: vec![],
            destroy_fn: None,
            is_interface: false,
        }
    }

    fn spec_with(classes: Vec<ClassSpec>, fns: Vec<FnBinding>) -> FfiSpec {
        FfiSpec {
            unit_name: "unit".to_string(),
            cpp_block_lines: vec![],
            class_specs: classes,
            lib_spec: LibSpec {
                link_name: "unit".to_string(),
                fwd_decls: vec![],
                fn_bindings: fns,
            },
            ..Default::default()
        }
    }

    fn fn_binding(name: &str) -> FnBinding {
        FnBinding {
            cpp_sig: format!("Foo* {}()", name),
            rust_name: name.to_string(),
            params: vec![],
            ret_type: Some("Foo".to_string()),
            is_unsafe: false,
            has_fn_ptr_param: false,
        }
    }

    fn unsafe_fn_binding(name: &str) -> FnBinding {
        FnBinding {
            cpp_sig: format!("void {}()", name),
            rust_name: name.to_string(),
            params: vec![],
            ret_type: None,
            is_unsafe: true,
            has_fn_ptr_param: false,
        }
    }

    #[test]
    fn generate_includes_pub_class_type_assertions() {
        let spec = spec_with(vec![class_with_method("Counter")], vec![]);
        let code = generate_smoke_test("my_lib", &[&spec]);
        assert!(
            code.contains("use my_lib::*;"),
            "应包含 crate 导入\n{}",
            code
        );
        assert!(
            code.contains("assert_type_available::<Counter>();"),
            "应为 pub class 生成类型可用性断言\n{}",
            code
        );
    }

    #[test]
    fn generate_skips_empty_class() {
        let spec = spec_with(vec![empty_class("Opaque")], vec![]);
        let code = generate_smoke_test("my_lib", &[&spec]);
        assert!(
            !code.contains("assert_type_available::<Opaque>();"),
            "空 ClassSpec 不应生成类型断言\n{}",
            code
        );
    }

    #[test]
    fn generate_factory_test_for_zero_param() {
        let spec = spec_with(vec![class_with_factory("Widget")], vec![]);
        let code = generate_smoke_test("my_lib", &[&spec]);
        assert!(
            code.contains("fn smoke_widget_new()"),
            "零参工厂函数应生成独立测试\n{}",
            code
        );
        assert!(
            code.contains("let _obj = widget_new();"),
            "工厂测试应调用构造函数\n{}",
            code
        );
    }

    #[test]
    fn generate_method_test_with_factory() {
        let spec = spec_with(vec![class_with_factory("Widget")], vec![]);
        let code = generate_smoke_test("my_lib", &[&spec]);
        assert!(
            code.contains("fn smoke_value()"),
            "有构造函数的类方法应生成测试\n{}",
            code
        );
        // 增强（P1.4）：返回 i32 的零参方法应生成 assert_eq!(result, 0)
        assert!(
            code.contains("let result = obj.value();"),
            "方法测试应构造实例并调用方法（结果存入 result）\n{}",
            code
        );
        assert!(
            code.contains("assert_eq!(result, 0);"),
            "i32 返回值的零参方法应生成默认值 assert 断言\n{}",
            code
        );
    }

    /// P1.4 新增：返回非原始类型（如类、字符串）的零参方法应保留 `let _result = ...` 形式
    #[test]
    fn generate_method_test_non_primitive_ret_keeps_underscore() {
        // 返回 Foo（类类型）的方法不应生成 assert（无默认值）
        let spec = spec_with(
            vec![ClassSpec {
                name: "Widget".to_string(),
                methods: vec![MethodBinding {
                    cpp_sig: "Foo get_foo() const".to_string(),
                    rust_name: "get_foo".to_string(),
                    self_kind: SelfKind::Ref,
                    params: vec![],
                    ret_type: Some("Foo".to_string()),
                    has_fn_ptr_param: false,
                }],
                associated_fns: vec![FnBinding {
                    cpp_sig: "Widget* widget_new()".to_string(),
                    rust_name: "widget_new".to_string(),
                    params: vec![],
                    ret_type: Some("Widget".to_string()),
                    is_unsafe: false,
                    has_fn_ptr_param: false,
                }],
                destroy_fn: None,
                is_interface: false,
            }],
            vec![],
        );
        let code = generate_smoke_test("my_lib", &[&spec]);
        assert!(
            code.contains("let _result = obj.get_foo();"),
            "非原始类型返回值的零参方法应保留 _result 形式（不生成 assert），实际：\n{}",
            code
        );
        assert!(
            !code.contains("assert_eq!(_result"),
            "非原始类型不应生成 assert_eq!，实际：\n{}",
            code
        );
    }

    /// P1.4 新增：bool 返回值的零参方法应生成 `assert_eq!(result, false)`
    #[test]
    fn generate_method_test_bool_ret_uses_false_default() {
        let spec = spec_with(
            vec![ClassSpec {
                name: "Flag".to_string(),
                methods: vec![MethodBinding {
                    cpp_sig: "bool is_on() const".to_string(),
                    rust_name: "is_on".to_string(),
                    self_kind: SelfKind::Ref,
                    params: vec![],
                    ret_type: Some("bool".to_string()),
                    has_fn_ptr_param: false,
                }],
                associated_fns: vec![FnBinding {
                    cpp_sig: "Flag* flag_new()".to_string(),
                    rust_name: "flag_new".to_string(),
                    params: vec![],
                    ret_type: Some("Flag".to_string()),
                    is_unsafe: false,
                    has_fn_ptr_param: false,
                }],
                destroy_fn: None,
                is_interface: false,
            }],
            vec![],
        );
        let code = generate_smoke_test("my_lib", &[&spec]);
        assert!(
            code.contains("assert_eq!(result, false);"),
            "bool 返回值应生成 `assert_eq!(result, false)` 默认值断言，实际：\n{}",
            code
        );
    }

    /// P1.4 单元测试：default_value_literal 覆盖所有原始类型
    #[test]
    fn default_value_literal_covers_primitive_types() {
        // 整数类型 → 0
        for t in &[
            "i8", "i16", "i32", "i64", "isize", "u8", "u16", "u32", "u64", "usize",
        ] {
            assert_eq!(default_value_literal(t), Some("0"), "type {}", t);
        }
        // 浮点 → 0.0
        assert_eq!(default_value_literal("f32"), Some("0.0"));
        assert_eq!(default_value_literal("f64"), Some("0.0"));
        // 布尔 → false
        assert_eq!(default_value_literal("bool"), Some("false"));
        // 非原始类型 → None
        assert_eq!(default_value_literal("Foo"), None);
        assert_eq!(default_value_literal("*const i8"), None);
        assert_eq!(default_value_literal("hicc_std::string"), None);
    }

    #[test]
    fn generate_global_fn_test_for_zero_param() {
        let spec = spec_with(vec![], vec![fn_binding("create_thing")]);
        let code = generate_smoke_test("my_lib", &[&spec]);
        assert!(
            code.contains("fn smoke_create_thing()"),
            "零参全局函数应生成独立测试\n{}",
            code
        );
    }

    #[test]
    fn generate_unsafe_global_fn_test() {
        let spec = spec_with(vec![], vec![unsafe_fn_binding("raw_operation")]);
        let code = generate_smoke_test("my_lib", &[&spec]);
        assert!(
            code.contains("fn smoke_raw_operation()"),
            "unsafe 零参函数应生成测试\n{}",
            code
        );
        assert!(
            code.contains("unsafe {"),
            "unsafe 函数测试应包含 unsafe 块\n{}",
            code
        );
    }

    #[test]
    fn generate_lists_untestable_fns_as_smoke_todo() {
        let spec = spec_with(
            vec![class_with_method("Counter")],
            vec![FnBinding {
                cpp_sig: "void do_stuff(int, int)".to_string(),
                rust_name: "do_stuff".to_string(),
                params: vec![
                    ("a".to_string(), "i32".to_string()),
                    ("b".to_string(), "i32".to_string()),
                ],
                ret_type: None,
                is_unsafe: false,
                has_fn_ptr_param: false,
            }],
        );
        let code = generate_smoke_test("my_lib", &[&spec]);
        assert!(
            code.contains("cpp2rust-todo[SMOKE]"),
            "有参数的函数应生成 SMOKE 占位说明\n{}",
            code
        );
        assert!(
            code.contains("- do_stuff"),
            "SMOKE 占位说明应列出函数名\n{}",
            code
        );
    }

    #[test]
    fn generate_fallback_link_test_when_nothing_testable() {
        let spec = spec_with(vec![], vec![]);
        let code = generate_smoke_test("my_lib", &[&spec]);
        assert!(
            code.contains("fn smoke_crate_links()"),
            "无可测试项时应生成最小链接测试\n{}",
            code
        );
    }
}
