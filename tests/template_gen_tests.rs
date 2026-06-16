//! v6 Phase A/B → v7：模板类 / 模板函数泛型骨架生成测试。
//!
//! 验证 v7 起**默认**（无需任何环境变量开关）即输出：
//! 模板类生成泛型 `import_class!`、模板函数生成泛型 `import_lib!` 骨架，
//! 以及实例化别名与构造工厂骨架。

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

// 局部变量声明中的实例化使用点（v6 Phase B 收尾：追踪函数 / 方法体内
// `Stack<int> s;`、`Stack<int>* p = new Stack<int>();` 等表达式级使用点）
void make_stacks() {
    Stack<unsigned int> local(0u);
    Stack<unsigned int>* heap = new Stack<unsigned int>(1u);
    (void)heap;
}

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
fn template_skeleton_emitted_by_default() {
    // v7：无需任何环境变量开关，默认即输出模板骨架。
    let on = match common::generate_from_source("tmpl_default", TEMPLATE_CLASS_SRC) {
        Some(s) => s,
        None => {
            eprintln!("跳过：当前环境缺少 C++ 预处理器或 libclang");
            return;
        }
    };
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
    // 应生成 Stack<long> 的实例化别名与构造工厂骨架。
    // `long` 的位宽随平台而异：LP64（Linux）映射为 i64，
    // LLP64（Windows）映射为 i32，故别名与工厂名须与平台保持一致。
    #[cfg(not(target_os = "windows"))]
    let (expected_alias, expected_factory) = (
        "pub type StackI64 = Stack<hicc::Pod<i64>>;",
        "pub unsafe fn stack_i64_new(initial: i64) -> StackI64;",
    );
    #[cfg(target_os = "windows")]
    let (expected_alias, expected_factory) = (
        "pub type StackI32 = Stack<hicc::Pod<i32>>;",
        "pub unsafe fn stack_i32_new(initial: i32) -> StackI32;",
    );
    assert!(
        on.contains(expected_alias),
        "应从显式实例化 template class Stack<long>; 收集到别名，实际输出：\n{}",
        on
    );
    assert!(
        on.contains(expected_factory),
        "应为显式实例化派生构造工厂骨架，实际输出：\n{}",
        on
    );

    // v6 Phase B 收尾：局部变量声明追踪 —— make_stacks() 中
    // `Stack<unsigned int> local;` 与 `new Stack<unsigned int>()` 应收集到 StackU32 别名
    assert!(
        on.contains("pub type StackU32 = Stack<hicc::Pod<u32>>;"),
        "应从局部变量声明 Stack<unsigned int> 收集到实例化别名，实际输出：\n{}",
        on
    );

    // v7 关键约定：模板骨架默认以**注释**形式输出（未实例化的模板无可链接符号、
    // 泛型 <T> 无法直接编译），故工具默认产物必须可编译。校验模板函数/模板类/别名
    // 三类骨架均处于注释行（以 `//` 起始），不存在未注释的活动绑定。
    for needle in [
        "pub unsafe fn do_swap(",
        "pub class Stack<T>",
        "pub type StackI32 = Stack<hicc::Pod<i32>>;",
    ] {
        for line in on.lines() {
            if line.contains(needle) {
                assert!(
                    line.trim_start().starts_with("//"),
                    "模板骨架行应为注释（以 // 起始）以保证默认产物可编译，实际行：\n{}",
                    line
                );
            }
        }
    }
}
