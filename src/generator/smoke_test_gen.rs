//! 冒烟测试生成器（Phase D）
//!
//! 由 `init` 在 `.cpp2rust/<feature>/rust/tests/smoke.rs` 生成冒烟测试，
//! 用于"生成即验证"：确认生成的 Rust FFI 绑定能够**编译**并**链接** C++ 实现。
//!
//! ## 设计来源
//!
//! 参考 hicc 自身的验证方式（`hicc-std/src/std_test/*`）——测试与 FFI 绑定位于同一
//! crate，通过 `cargo test` 链接 C++ 静态库并真实使用绑定，验证 ABI 与基本可用性。
//!
//! 由于 `init` 生成的 `import_lib!` 自由函数为模块私有（仅在所属 unit 模块内可见，
//! 不经 `lib.rs` 的 `pub use` 重导出），集成测试 `tests/smoke.rs` 只能访问被重导出的
//! **`pub class` 类型**。因此本生成器采取如下策略：
//!
//! 1. 对每个 `pub class` 类型生成**编译期类型可用性断言**——保证 FFI 类型层编译通过；
//!    集成测试二进制会链接 lib crate 及其 C++ shim，从而验证链接完整性。
//! 2. 对无法从集成测试直接调用的工厂/全局函数，生成 `cpp2rust-todo[SMOKE]` 占位说明，
//!    提示用户补充行为断言（将函数声明为 `pub fn` 或在 crate 内部添加测试）。

use crate::ffi_model::FfiSpec;

/// 控制是否生成冒烟测试的环境变量。设为 `"0"`（或 `"false"`）时关闭生成。
pub const GEN_SMOKE_ENV: &str = "CPP2RUST_GEN_SMOKE";

/// 根据环境变量判断是否应生成冒烟测试（默认开启）。
///
/// 仅当 `CPP2RUST_GEN_SMOKE` 显式设为 `0` / `false` / `no` / `off`（忽略大小写）时返回
/// `false`，其余情况（含未设置）均返回 `true`。
pub fn smoke_enabled() -> bool {
    match std::env::var(GEN_SMOKE_ENV) {
        Ok(v) => !matches!(
            v.trim().to_ascii_lowercase().as_str(),
            "0" | "false" | "no" | "off"
        ),
        Err(_) => true,
    }
}

/// 收集所有会经 `lib.rs` 重导出的 `pub class` 类型名（与 `hicc_codegen::generate`
/// 的跳过条件一致：空 `ClassSpec` 不生成 `import_class!` 块，因此也不在此列出）。
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

/// 收集所有无法从集成测试直接调用的工厂/全局函数名（用于 SMOKE 占位说明）。
fn collect_fn_names(specs: &[&FfiSpec]) -> Vec<String> {
    let mut names: Vec<String> = Vec::new();
    for spec in specs {
        for cs in &spec.class_specs {
            for fb in &cs.associated_fns {
                if !names.contains(&fb.rust_name) {
                    names.push(fb.rust_name.clone());
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

/// 生成 `tests/smoke.rs` 的完整内容。
///
/// `lib_name` 为生成 crate 的库名（`feature` 中的 `-` 已替换为 `_`）。
/// `specs` 为各编译单元的 FFI 规格。
pub fn generate_smoke_test(lib_name: &str, specs: &[&FfiSpec]) -> String {
    let class_names = collect_pub_class_names(specs);
    let fn_names = collect_fn_names(specs);

    let mut out = String::new();
    out.push_str("//! 由 cpp2rust-demo 自动生成的冒烟测试（init 阶段）。\n");
    out.push_str("//!\n");
    out.push_str("//! 目的：验证生成的 Rust FFI 绑定可编译并链接 C++ 实现（\"生成即验证\"）。\n");
    out.push_str("//! 运行：在本目录执行 `cargo test`。\n");
    out.push_str(&format!(
        "//! 关闭生成：设置环境变量 `{}=0` 后重新执行 `cpp2rust-demo init`。\n",
        GEN_SMOKE_ENV
    ));
    out.push_str("//!\n");
    out.push_str("//! 说明：`import_lib!` 中的工厂/全局函数为模块私有，集成测试无法直接调用，\n");
    out.push_str("//! 因此本文件仅做类型可用性与链接验证；如需构造实例做行为断言，\n");
    out.push_str("//! 请将对应函数声明为 `pub fn`，或在生成的 crate 内部补充测试。\n");
    out.push_str("\n#![allow(unused_imports)]\n\n");
    out.push_str(&format!("use {}::*;\n\n", lib_name));

    if class_names.is_empty() {
        // 没有可引用的 pub 类型：生成一个最小测试，仅确保 crate 可链接进测试二进制。
        out.push_str("/// 最小冒烟测试：确保生成的 crate 能编译并链接进测试二进制。\n");
        out.push_str("#[test]\n");
        out.push_str("fn smoke_crate_links() {\n");
        out.push_str("    // 本 crate 未导出可在集成测试中引用的 `pub class` 类型。\n");
        out.push_str("    // 该测试通过即说明生成的 FFI crate（含 C++ shim）链接成功。\n");
        out.push_str("    assert!(true);\n");
        out.push_str("}\n");
    } else {
        out.push_str("/// 编译期断言：所有生成的 FFI 类型均可用。\n");
        out.push_str("///\n");
        out.push_str("/// 此测试编译通过即说明 FFI 类型层正确，且测试二进制成功链接了\n");
        out.push_str("/// 生成 crate 编译出的 C++ shim 静态库。\n");
        out.push_str("#[test]\n");
        out.push_str("fn smoke_ffi_types_available() {\n");
        out.push_str("    fn assert_type_available<T>() {}\n");
        for name in &class_names {
            out.push_str(&format!("    assert_type_available::<{}>();\n", name));
        }
        out.push_str("}\n");
    }

    if !fn_names.is_empty() {
        out.push('\n');
        out.push_str("// cpp2rust-todo[SMOKE]: 以下工厂/全局函数在生成的绑定中为模块私有，\n");
        out.push_str("// 无法从集成测试直接调用。如需构造实例并做行为断言，可：\n");
        out.push_str("//   1) 将对应函数声明为 `pub fn`（在所属 unit 的 import_lib! 中）；或\n");
        out.push_str("//   2) 在生成的 crate 内部添加 #[cfg(test)] 测试。\n");
        out.push_str("// 待补充行为断言的函数：\n");
        for name in &fn_names {
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
        }
    }

    fn fn_binding(name: &str) -> FnBinding {
        FnBinding {
            cpp_sig: format!("Foo* {}()", name),
            rust_name: name.to_string(),
            params: vec![],
            ret_type: Some("*mut Foo".to_string()),
            is_unsafe: false,
            has_fn_ptr_param: false,
        }
    }

    #[test]
    fn smoke_enabled_defaults_true_when_unset() {
        // 在未显式设置时默认开启。
        std::env::remove_var(GEN_SMOKE_ENV);
        assert!(smoke_enabled());
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
        assert!(
            code.contains("smoke_crate_links"),
            "无 pub 类型时应生成最小链接冒烟测试\n{}",
            code
        );
    }

    #[test]
    fn generate_lists_private_fns_as_smoke_todo() {
        let spec = spec_with(
            vec![class_with_method("Counter")],
            vec![fn_binding("counter_new")],
        );
        let code = generate_smoke_test("my_lib", &[&spec]);
        assert!(
            code.contains("cpp2rust-todo[SMOKE]"),
            "存在私有工厂函数时应生成 SMOKE 占位说明\n{}",
            code
        );
        assert!(
            code.contains("- counter_new"),
            "SMOKE 占位说明应列出工厂函数名\n{}",
            code
        );
    }
}
