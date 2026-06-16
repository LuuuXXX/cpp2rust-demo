# 025_template_class - 类模板（hicc 直出，去 shim）

## C++ 特性

本示例展示 C++ **类模板**的 FFI 处理方式。类模板 `Stack<T>` 是「蓝图」，须按具体类型
实例化（`Stack<int>`、`Stack<double>` …）才成为可链接的具体类型。本示例将每个具体类型
暴露为 idiomatic 命名空间类（`IntStack` / `DoubleStack`，内部复用 `Stack<T>`），hicc 直出
按普通类绑定其方法与 `make_unique` 构造工厂，无需任何 extern-C shim。

## C++ 代码

### template_class.h

```cpp
namespace template_class_ns {

// 类模板：泛型栈蓝图
template <typename T>
class Stack {
public:
    std::stack<T> data;
    Stack() = default;
    int size() const { return static_cast<int>(data.size()); }
    bool empty() const { return data.empty(); }
    void push(T value) { data.push(value); }
    T top() const { return data.top(); }
    void pop() { data.pop(); }
};

// 显式实例化为具体类
class IntStack {
public:
    Stack<int> impl;
    IntStack() = default;
    int size() const { return impl.size(); }
    /* ... push/top/pop/empty ... */
};

class DoubleStack { /* Stack<double> 同理 */ };

} // namespace template_class_ns
```

## Rust FFI 代码

hicc 直出无需 extern-C shim，直接绑定具体类与 `make_unique` 工厂：

```rust
hicc::cpp! {
    #include "template_class.h"
}

hicc::import_class! {
    #[cpp(class = "template_class_ns::IntStack")]
    pub class IntStack {
        #[cpp(method = "int size() const")]
        pub fn size(&self) -> i32;
        #[cpp(method = "bool empty() const")]
        pub fn empty(&self) -> bool;
        #[cpp(method = "void push(int value)")]
        pub fn push(&mut self, value: i32);
        #[cpp(method = "int top() const")]
        pub fn top(&self) -> i32;
        #[cpp(method = "void pop()")]
        pub fn pop(&mut self);

        pub fn new() -> Self { int_stack_new() }
    }
}

// DoubleStack 同理（Stack<double>）

hicc::import_lib! {
    #![link_name = "template_class"]

    #[cpp(func = "std::unique_ptr<template_class_ns::IntStack> hicc::make_unique<template_class_ns::IntStack>()")]
    pub fn int_stack_new() -> IntStack;

    #[cpp(func = "std::unique_ptr<template_class_ns::DoubleStack> hicc::make_unique<template_class_ns::DoubleStack>()")]
    pub fn double_stack_new() -> DoubleStack;
}
```

## FFI 对比分析

| 方面 | C++ | Rust FFI |
|------|-----|----------|
| 模板定义 | `template<T> class Stack` | 不可直接导出 |
| 模板实例化 | `Stack<int>` / `Stack<double>` | 暴露为具体类 `IntStack` / `DoubleStack` |
| 构造 | `IntStack()` | `IntStack::new()`（`make_unique` 工厂） |
| 内存管理 | RAII | hicc `unique_ptr` 自动析构 |

## 运行结果

```
=== 025_template_class - 类模板 ===

IntStack empty: true
IntStack size: 3
IntStack top: 30
After pop, top: 20

DoubleStack empty: true
DoubleStack size: 3
DoubleStack top: 3.3
After pop, top: 2.2

Rust FFI: 类模板 = 为每种类型实例化独立的具体类
Stack<int> -> IntStack
Stack<double> -> DoubleStack
```

## 冒烟测试

本示例包含集成冒烟测试（`rust_hicc/tests/smoke.rs`），验证生成的 Rust FFI 绑定可编译、
链接并正确调用。

### 测试用例

| 测试函数 | 验证内容 |
|---------|---------|
| `smoke_int_stack_basic` | `IntStack` 的 push/top/pop/size/empty 行为 |
| `smoke_double_stack_basic` | `DoubleStack` 的 push/top/pop/size 行为 |
| `smoke_int_stack_type_available` | `IntStack` / `DoubleStack` 类型可用 |

### 运行方式

```bash
cd examples/025_template_class/rust_hicc
cargo test --test smoke
```

### 各平台支持

| 平台 | 状态 | 备注 |
|------|------|------|
| Linux (Ubuntu) | ✅ | CI `l-smoke` job 已覆盖 |
| macOS | ✅ | 支持 |
| Windows MinGW | ✅ | 支持 |

## 总结

- C++ 类模板无法直接 FFI 导出（模板本身没有可链接符号）
- 策略：将每个具体实例化暴露为 idiomatic 命名空间类
- hicc 直出按普通类绑定方法与 `make_unique` 工厂，无需 extern-C shim
- `Stack<int>` → `IntStack`，`Stack<double>` → `DoubleStack`
