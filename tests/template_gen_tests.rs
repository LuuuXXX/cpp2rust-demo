//! Phase B 模板绑定生成集成测试
//!
//! 验证两件事：
//! 1. **默认关闭**：未设置 `CPP2RUST_GEN_TEMPLATES` 时，生成产物不含任何模板块
//!    （保证 L1 黄金 / L2 编译基线零变更）。
//! 2. **开启后**：设置 `CPP2RUST_GEN_TEMPLATES=1` 时，模板类生成泛型 `import_class!`
//!    骨架，模板函数生成泛型 `import_lib!` 骨架，并带 `cpp2rust-todo[TPL]` 提示。
//!
//! 这些用例依赖 libclang（同 L1）；与 golden 测试共享 `common` 的串行化 Mutex。

mod common;

/// 在设置/清除环境变量的前提下，对示例运行工具并返回生成的 hicc 块。
fn run_with_templates(example_dir: &str, enabled: bool) -> String {
    // 注意：环境变量是进程级的；本测试二进制独立运行，且 common 内部以 Mutex
    // 串行化 libclang 调用，故在调用前后成对设置/清除即可避免相互干扰。
    if enabled {
        std::env::set_var("CPP2RUST_GEN_TEMPLATES", "1");
    } else {
        std::env::remove_var("CPP2RUST_GEN_TEMPLATES");
    }
    let out = common::run_tool_on(example_dir);
    std::env::remove_var("CPP2RUST_GEN_TEMPLATES");
    out
}

#[test]
#[cfg_attr(
    not(feature = "full-test"),
    ignore = "requires libclang; run with --features full-test --test-threads=1"
)]
fn default_off_emits_no_template_blocks() {
    let generated = run_with_templates("examples/025_template_class", false);
    assert!(
        !generated.contains("cpp2rust-todo[TPL]"),
        "默认（未开启 CPP2RUST_GEN_TEMPLATES）不应生成模板块，实际：\n{generated}"
    );
    assert!(
        !generated.contains("pub class Stack<"),
        "默认不应生成泛型 Stack<T> 骨架，实际：\n{generated}"
    );
    assert!(
        !generated.contains("= Stack<hicc::Pod<"),
        "默认不应生成实例化别名，实际：\n{generated}"
    );
}

#[test]
#[cfg_attr(
    not(feature = "full-test"),
    ignore = "requires libclang; run with --features full-test --test-threads=1"
)]
fn enabled_emits_generic_class_skeleton() {
    let generated = run_with_templates("examples/025_template_class", true);
    assert!(
        generated.contains("pub class Stack<T>"),
        "开启后应生成泛型 Stack<T> 的 import_class! 骨架，实际：\n{generated}"
    );
    assert!(
        generated.contains("cpp2rust-todo[TPL]"),
        "泛型骨架应带 cpp2rust-todo[TPL] 提示，实际：\n{generated}"
    );
    // 模板成员方法签名应被映射（如 size / push / pop）
    assert!(
        generated.contains("fn size(&self)"),
        "应映射 const 成员方法 size，实际：\n{generated}"
    );
    // 模板类 #[cpp(class = ...)] 应使用 hicc 要求的完整模板形式，而非裸类名
    assert!(
        generated.contains("#[cpp(class = \"template<class T> Stack<T>\")]"),
        "模板类应声明完整模板形式 template<class T> Stack<T>，实际：\n{generated}"
    );
}

#[test]
#[cfg_attr(
    not(feature = "full-test"),
    ignore = "requires libclang; run with --features full-test --test-threads=1"
)]
fn enabled_emits_instantiation_aliases() {
    // 025 中 IntStack/DoubleStack 分别以字段 Stack<int> / Stack<double> 实例化模板。
    let generated = run_with_templates("examples/025_template_class", true);
    assert!(
        generated.contains("class StackInt = Stack<hicc::Pod<i32>>;"),
        "应为 Stack<int> 生成实例化别名 StackInt，实际：\n{generated}"
    );
    assert!(
        generated.contains("class StackDouble = Stack<hicc::Pod<f64>>;"),
        "应为 Stack<double> 生成实例化别名 StackDouble，实际：\n{generated}"
    );
}

#[test]
#[cfg_attr(
    not(feature = "full-test"),
    ignore = "requires libclang; run with --features full-test --test-threads=1"
)]
fn enabled_emits_generic_template_fn_skeleton() {
    let generated = run_with_templates("examples/024_template_function", true);
    assert!(
        generated.contains("cpp2rust-todo[TPL]"),
        "开启后模板函数应生成带 cpp2rust-todo[TPL] 提示的骨架，实际：\n{generated}"
    );
    // 模板函数签名应保留泛型形参（如 do_swap<T>）
    assert!(
        generated.contains("<T>"),
        "模板函数 C++ 签名应保留泛型形参 <T>，实际：\n{generated}"
    );
}
