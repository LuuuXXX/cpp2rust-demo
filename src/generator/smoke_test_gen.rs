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
//! ## 生成策略（表驱动 + 行为级断言）
//!
//! 对每个 FfiSpec 中的可测试项生成独立 `#[test]` 函数：
//!
//! 1. **编译期类型断言**：所有 `pub class` 类型可用。
//! 2. **setter/getter 往返**（行为级·双值）：类含零参构造、且存在严格配对的
//!    `set_<x>(标量)` 与 `<x>()`/`get_<x>()`/`is_<x>()` 标量 getter 时，
//!    生成「构造 → set(A) → `assert_eq!(get, A)` → set(B) → `assert_eq!(get, B)`」
//!    的双值确定性往返断言（A≠B，证明 getter 真实回读而非返回常量）。
//! 3. **工厂函数**（associated_fns，如 `counter_new()`）：零参数时生成调用测试。
//! 4. **类方法**（methods）：所在类有零参构造函数时，构造实例并调用方法。
//! 5. **全局函数**（lib_spec.fn_bindings）：零参数时生成调用测试。
//! 6. 其余含非平凡参数、结果不可静态确定的项以**最小化** `cpp2rust-todo[SMOKE]`
//!    注释列出，把占位比例降到最低。
//!
//! ## 安全约束
//!
//! 真实项目 E2E 会对生成的 `tests/smoke.rs` 运行 `cargo test`，因此行为级
//! `assert_eq!` 仅在语义可确定（严格命名 + 标量类型配对）时生成，避免对未知库
//! 实现做出错误假设而导致运行期失败。

use crate::ffi_model::{ClassSpec, FfiSpec, FnBinding, MethodBinding, SelfKind};

/// 冒烟测试文件相对生成项目根目录的路径（用于生成与用户提示保持一致）。
pub const SMOKE_TEST_PATH: &str = "tests/smoke.rs";

// ─────────────────────────────────────────────────────────────────
//  辅助函数
// ─────────────────────────────────────────────────────────────────

/// 收集所有会经 `lib.rs` 重导出的 `pub class` 类型名。
fn collect_pub_class_names(specs: &[&FfiSpec]) -> Vec<String> {
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut names: Vec<String> = Vec::new();
    for spec in specs {
        for cs in &spec.class_specs {
            if cs.is_empty() {
                continue;
            }
            if seen.insert(cs.name.clone()) {
                names.push(cs.name.clone());
            }
        }
    }
    names
}

/// 收集所有 FFI 函数名（工厂函数、方法、全局函数），用于 SMOKE 占位说明。
fn collect_all_fn_names(specs: &[&FfiSpec]) -> Vec<String> {
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut names: Vec<String> = Vec::new();
    for spec in specs {
        for cs in &spec.class_specs {
            for fb in &cs.associated_fns {
                if seen.insert(fb.rust_name.clone()) {
                    names.push(fb.rust_name.clone());
                }
            }
            for mb in &cs.methods {
                if seen.insert(mb.rust_name.clone()) {
                    names.push(mb.rust_name.clone());
                }
            }
        }
        for fb in &spec.lib_spec.fn_bindings {
            if seen.insert(fb.rust_name.clone()) {
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

/// 返回某标量类型用于「双值往返」断言的两个**互不相等**字面量 `(first, second)`。
///
/// 双值断言在单值（仅写 `42`）基础上再写入一个不同值并断言，能进一步证明 getter
/// 真实回读 setter 写入的内容，而非恰好返回与首个字面量相等的常量。两个字面量均取
/// 小整数/常见浮点/布尔，避免触发被绑定库可能存在的取值范围约束。
fn scalar_literal_pair(ty: &str) -> Option<(&'static str, &'static str)> {
    match ty.trim() {
        "i8" | "i16" | "i32" | "i64" | "i128" | "isize" | "u8" | "u16" | "u32" | "u64" | "u128"
        | "usize" => Some(("42", "7")),
        "f32" | "f64" => Some(("42.0", "7.0")),
        "bool" => Some(("true", "false")),
        _ => None,
    }
}

/// setter/getter 往返配对结果。
struct RoundTrip<'a> {
    setter: &'a MethodBinding,
    getter: &'a MethodBinding,
    /// 用于断言的首个标量字面量。
    literal: &'static str,
    /// 用于「双值往返」二次断言的另一个标量字面量（与 `literal` 不相等）。
    literal2: &'static str,
}

/// 在单个类中检测严格配对的 setter/getter，用于生成行为级往返断言。
///
/// 规则（从严，避免对未知库做错误假设）：
/// - setter：`rust_name` 形如 `set_<root>`，恰好 1 个标量参数，`&mut self`，无返回值；
/// - getter：`rust_name` 为 `<root>` / `get_<root>` / `is_<root>` 之一，零参数，
///   返回类型与 setter 参数标量类型完全一致。
fn detect_round_trips(cs: &ClassSpec) -> Vec<RoundTrip<'_>> {
    let mut pairs: Vec<RoundTrip<'_>> = Vec::new();
    for setter in &cs.methods {
        let root = match setter.rust_name.strip_prefix("set_") {
            Some(r) if !r.is_empty() => r,
            _ => continue,
        };
        if setter.self_kind != SelfKind::RefMut || setter.ret_type.is_some() {
            continue;
        }
        if setter.params.len() != 1 || setter.has_fn_ptr_param {
            continue;
        }
        let param_ty = &setter.params[0].1;
        let (literal, literal2) = match scalar_literal_pair(param_ty) {
            Some(pair) => pair,
            None => continue,
        };
        let getter = cs.methods.iter().find(|m| {
            m.params.is_empty()
                && !m.has_fn_ptr_param
                && m.ret_type.as_deref() == Some(param_ty.as_str())
                && (m.rust_name == root
                    || m.rust_name == format!("get_{}", root)
                    || m.rust_name == format!("is_{}", root))
        });
        if let Some(getter) = getter {
            pairs.push(RoundTrip {
                setter,
                getter,
                literal,
                literal2,
            });
        }
    }
    pairs
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
    // 使用 `::lib_name` 绝对路径前缀，避免 crate 名与其导出的同名模块产生 E0659 歧义。
    // 例：crate `tomlplusplus_ffi` 导出模块 `tomlplusplus_ffi` 时，`use tomlplusplus_ffi::*`
    // 会报"ambiguous name"，而 `use ::tomlplusplus_ffi::*` 明确指向外部 crate。
    out.push_str(&format!("use ::{}::*;\n\n", lib_name));

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

    // ── B2. setter/getter 往返（行为级 assert_eq! 断言）──────────
    // 仅对「有零参构造 + 严格标量 setter/getter 配对」的类生成确定性往返断言。
    for spec in specs {
        for cs in &spec.class_specs {
            if cs.methods.is_empty() {
                continue;
            }
            let factory = match find_zero_param_factory(specs, &cs.name) {
                Some(f) => f,
                None => continue,
            };
            for rt in detect_round_trips(cs) {
                if tested_fns.contains(&rt.setter.rust_name)
                    || tested_fns.contains(&rt.getter.rust_name)
                {
                    continue;
                }
                tested_fns.push(rt.setter.rust_name.clone());
                tested_fns.push(rt.getter.rust_name.clone());

                out.push_str(&format!(
                    "/// 行为级双值往返：set_* 写入两个不同标量后，get_* 均应回读写入值。\n#[test]\nfn smoke_{}_roundtrip() {{\n",
                    rt.setter.rust_name
                ));
                if factory.is_unsafe {
                    out.push_str(&format!(
                        "    let mut obj = unsafe {{ {}() }};\n",
                        factory.rust_name
                    ));
                } else {
                    out.push_str(&format!("    let mut obj = {}();\n", factory.rust_name));
                }
                out.push_str(&format!(
                    "    obj.{}({});\n",
                    rt.setter.rust_name, rt.literal
                ));
                out.push_str(&format!(
                    "    assert_eq!(obj.{}(), {}, \"{} 写入 {} 后 {} 应回读写入值\");\n",
                    rt.getter.rust_name,
                    rt.literal,
                    rt.setter.rust_name,
                    rt.literal,
                    rt.getter.rust_name
                ));
                out.push_str(&format!(
                    "    obj.{}({});\n",
                    rt.setter.rust_name, rt.literal2
                ));
                out.push_str(&format!(
                    "    assert_eq!(obj.{}(), {}, \"{} 改写为 {} 后 {} 应回读新值（验证真实存储而非常量）\");\n",
                    rt.getter.rust_name, rt.literal2, rt.setter.rust_name, rt.literal2, rt.getter.rust_name
                ));
                out.push_str("}\n\n");
            }
        }
    }

    // ── C. 类方法测试（有零参构造函数的类）──────────────────────

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

                // 调用方法
                let method_call = format!("obj.{}()", mb.rust_name);
                match &mb.ret_type {
                    Some(_) => {
                        out.push_str(&format!("    let _result = {};\n", method_call));
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
            ..Default::default()
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
            ..Default::default()
        }
    }

    fn empty_class(name: &str) -> ClassSpec {
        ClassSpec {
            name: name.to_string(),
            methods: vec![],
            associated_fns: vec![],
            destroy_fn: None,
            is_interface: false,
            ..Default::default()
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
            code.contains("use ::my_lib::*;"),
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
        assert!(
            code.contains("let _result = obj.value();"),
            "方法测试应构造实例并调用方法\n{}",
            code
        );
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
    fn generate_setter_getter_roundtrip_assertion() {
        let cs = ClassSpec {
            name: "Box".to_string(),
            methods: vec![
                MethodBinding {
                    cpp_sig: "void set_value(int)".to_string(),
                    rust_name: "set_value".to_string(),
                    self_kind: SelfKind::RefMut,
                    params: vec![("v".to_string(), "i32".to_string())],
                    ret_type: None,
                    has_fn_ptr_param: false,
                },
                MethodBinding {
                    cpp_sig: "int value() const".to_string(),
                    rust_name: "value".to_string(),
                    self_kind: SelfKind::Ref,
                    params: vec![],
                    ret_type: Some("i32".to_string()),
                    has_fn_ptr_param: false,
                },
            ],
            associated_fns: vec![FnBinding {
                cpp_sig: "Box* box_new()".to_string(),
                rust_name: "box_new".to_string(),
                params: vec![],
                ret_type: Some("Box".to_string()),
                is_unsafe: false,
                has_fn_ptr_param: false,
            }],
            destroy_fn: None,
            is_interface: false,
            ..Default::default()
        };
        let spec = spec_with(vec![cs], vec![]);
        let code = generate_smoke_test("my_lib", &[&spec]);
        assert!(
            code.contains("fn smoke_set_value_roundtrip()"),
            "应为 setter/getter 配对生成往返测试\n{}",
            code
        );
        assert!(
            code.contains("obj.set_value(42);"),
            "往返测试应调用 setter 写入标量字面量\n{}",
            code
        );
        assert!(
            code.contains("assert_eq!(obj.value(), 42,"),
            "往返测试应对 getter 结果生成 assert_eq! 行为级断言\n{}",
            code
        );
        assert!(
            code.contains("obj.set_value(7);") && code.contains("assert_eq!(obj.value(), 7,"),
            "往返测试应写入第二个不同标量并再次断言（双值往返，验证真实存储）\n{}",
            code
        );
    }

    #[test]
    fn generate_no_roundtrip_for_non_scalar_setter() {
        let cs = ClassSpec {
            name: "Holder".to_string(),
            methods: vec![
                MethodBinding {
                    cpp_sig: "void set_name(const char*)".to_string(),
                    rust_name: "set_name".to_string(),
                    self_kind: SelfKind::RefMut,
                    params: vec![("v".to_string(), "*const ::std::os::raw::c_char".to_string())],
                    ret_type: None,
                    has_fn_ptr_param: false,
                },
                MethodBinding {
                    cpp_sig: "const char* name() const".to_string(),
                    rust_name: "name".to_string(),
                    self_kind: SelfKind::Ref,
                    params: vec![],
                    ret_type: Some("*const ::std::os::raw::c_char".to_string()),
                    has_fn_ptr_param: false,
                },
            ],
            associated_fns: vec![FnBinding {
                cpp_sig: "Holder* holder_new()".to_string(),
                rust_name: "holder_new".to_string(),
                params: vec![],
                ret_type: Some("Holder".to_string()),
                is_unsafe: false,
                has_fn_ptr_param: false,
            }],
            destroy_fn: None,
            is_interface: false,
            ..Default::default()
        };
        let spec = spec_with(vec![cs], vec![]);
        let code = generate_smoke_test("my_lib", &[&spec]);
        assert!(
            !code.contains("_roundtrip()"),
            "非标量 setter 不应生成往返 assert_eq!（避免对未知库实现做错误假设）\n{}",
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
