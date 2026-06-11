//! v6 Phase A/B：模板类 / 模板函数泛型骨架生成测试。
//!
//! 验证：
//! 1. 默认（未设置 `CPP2RUST_GEN_TEMPLATES`）时，生成器不输出任何模板骨架；
//! 2. 开启开关后，模板类生成泛型 `import_class!`、模板函数生成泛型 `import_lib!` 骨架。
//!
//! 因为开关通过进程级环境变量控制，所有断言集中在单个 `#[test]` 中串行执行，
//! 避免与其他测试并发设置/读取环境变量产生竞态。

mod common;

const TEMPLATE_CLASS_SRC: &str = r#"
template<typename T>
class Stack {
public:
    Stack(T initial);
    void push(T value);
    T top() const;
    bool empty() const;
};

class IntStack {
public:
    Stack<int> impl;
};

class DoubleStack {
public:
    Stack<double> impl;
};

// 方法参数中的实例化使用点（v6 Phase B 增强（续）：追踪来源扩展到方法参数 / 返回类型）
class StackUser {
public:
    void use_short(Stack<short>& s);
};

// 显式实例化（v6 Phase B 增强（再续）：追踪 `template class Foo<T>;`）
template class Stack<long>;

template<typename T>
void do_swap(T* a, T* b) {
    T tmp = *a;
    *a = *b;
    *b = tmp;
}
"#;

#[test]
#[cfg_attr(
    not(feature = "full-test"),
    ignore = "requires libclang; run with --features full-test --test-threads=1"
)]
fn template_skeleton_gated_by_env() {
    // ── 默认关闭：不应输出任何模板骨架 ──
    std::env::remove_var("CPP2RUST_GEN_TEMPLATES");
    let off = match common::generate_from_source("tmpl_off", TEMPLATE_CLASS_SRC) {
        Some(s) => s,
        None => {
            eprintln!("跳过：当前环境缺少 C++ 预处理器或 libclang");
            return;
        }
    };
    assert!(
        !off.contains("pub class Stack<T>"),
        "默认关闭时不应生成模板类骨架，实际输出：\n{}",
        off
    );
    assert!(
        !off.contains("do_swap<T>"),
        "默认关闭时不应生成模板函数骨架，实际输出：\n{}",
        off
    );
    assert!(
        !off.contains("cpp2rust-todo[TMPL]"),
        "默认关闭时不应出现 TMPL 占位，实际输出：\n{}",
        off
    );
    assert!(
        !off.contains("pub type StackI32"),
        "默认关闭时不应生成模板实例化别名，实际输出：\n{}",
        off
    );
    assert!(
        !off.contains("stack_i32_new"),
        "默认关闭时不应生成模板构造工厂骨架，实际输出：\n{}",
        off
    );
    assert!(
        !off.contains("pub type StackI64"),
        "默认关闭时不应生成显式实例化别名，实际输出：\n{}",
        off
    );

    // ── 开启开关：应输出泛型骨架 ──
    std::env::set_var("CPP2RUST_GEN_TEMPLATES", "1");
    let on = common::generate_from_source("tmpl_on", TEMPLATE_CLASS_SRC)
        .expect("已确认环境可用，生成不应失败");
    std::env::remove_var("CPP2RUST_GEN_TEMPLATES");

    // 模板类：泛型 import_class! + #[cpp(class = "template<...> Stack<T>")]
    assert!(
        on.contains("pub class Stack<T>"),
        "应生成泛型模板类骨架，实际输出：\n{}",
        on
    );
    assert!(
        on.contains("#[cpp(class = \"template<class T> Stack<T>\")]"),
        "应生成正确的模板类 #[cpp(class = ...)] 声明，实际输出：\n{}",
        on
    );
    // 成员方法签名保留泛型 T
    assert!(
        on.contains("#[cpp(method = \"void push(T value)\")]"),
        "应保留模板成员方法签名，实际输出：\n{}",
        on
    );

    // 模板函数：泛型 import_lib! + #[cpp(func = "void do_swap<T>(T*, T*)")]
    assert!(
        on.contains("#[cpp(func = \"void do_swap<T>(T*, T*)\")]"),
        "应生成泛型模板函数骨架，实际输出：\n{}",
        on
    );
    // 含 TMPL 占位注释，提示用户按实例化类型补全
    assert!(
        on.contains("cpp2rust-todo[TMPL]"),
        "模板骨架应附带 TMPL 占位注释，实际输出：\n{}",
        on
    );

    // 模板实例化别名（v6 Phase B 增强）：包装类字段 Stack<int>/Stack<double>
    // 应生成 hicc::Pod 包装的类型别名
    assert!(
        on.contains("pub type StackI32 = Stack<hicc::Pod<i32>>;"),
        "应生成 Stack<int> 的实例化别名，实际输出：\n{}",
        on
    );
    assert!(
        on.contains("pub type StackF64 = Stack<hicc::Pod<f64>>;"),
        "应生成 Stack<double> 的实例化别名，实际输出：\n{}",
        on
    );

    // v6 Phase B 增强（续）：追踪来源扩展到方法参数 —— StackUser::use_short(Stack<short>&)
    // 应生成 Stack<short> 的实例化别名
    assert!(
        on.contains("pub type StackI16 = Stack<hicc::Pod<i16>>;"),
        "应从方法参数 Stack<short>& 收集到实例化别名，实际输出：\n{}",
        on
    );

    // v6 Phase B 增强（续）：构造工厂骨架 —— Stack(T initial) 派生
    // StackI32 / StackF64 / StackI64 的工厂函数
    assert!(
        on.contains("pub unsafe fn stack_i32_new(initial: i32) -> StackI32;"),
        "应生成 StackI32 构造工厂骨架，实际输出：\n{}",
        on
    );
    assert!(
        on.contains("#[cpp(func = \"Stack<int>* stack_i32_new(int initial)\")]"),
        "应生成正确的工厂 #[cpp(func = ...)] 声明，实际输出：\n{}",
        on
    );
    assert!(
        on.contains("pub unsafe fn stack_f64_new(initial: f64) -> StackF64;"),
        "应生成 StackF64 构造工厂骨架，实际输出：\n{}",
        on
    );

    // v6 Phase B 增强（再续）：显式实例化 `template class Stack<long>;`
    // 应生成 Stack<long> 的实例化别名与构造工厂骨架
    assert!(
        on.contains("pub type StackI64 = Stack<hicc::Pod<i64>>;"),
        "应从显式实例化 template class Stack<long>; 收集到别名，实际输出：\n{}",
        on
    );
    assert!(
        on.contains("pub unsafe fn stack_i64_new(initial: i64) -> StackI64;"),
        "应为显式实例化派生 StackI64 构造工厂骨架，实际输出：\n{}",
        on
    );
}
