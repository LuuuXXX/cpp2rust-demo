# 025_template_class - 类模板

## C++ 特性

本示例展示 C++ 类模板的 FFI 处理方式。类模板必须为每种类型显式实例化。

## C++ 代码

### template_class.h

```cpp
// Stack<int> 和 Stack<double> 是两个完全独立的类
struct IntStack { /* ... */ };
struct DoubleStack { /* ... */ };
```

### template_class.cpp

```cpp
// 内部使用 std::stack 实现
struct IntStack {
    std::stack<int> data;
};

struct DoubleStack {
    std::stack<double> data;
};
```

## Rust FFI 代码

### main.rs

```rust
// 每个模板实例 = 独立的 Rust 类型
struct IntStack;
struct DoubleStack;

// IntStack 方法
fn intstack_new() -> *mut IntStack;
fn intstack_push(self_: *mut IntStack, value: i32);
fn intstack_top(self_: *mut IntStack) -> i32;

// DoubleStack 方法
fn doublestack_new() -> *mut DoubleStack;
fn doublestack_push(self_: *mut DoubleStack, value: f64);
fn doublestack_top(self_: *mut DoubleStack) -> f64;
```

## FFI 对比分析

| 方面 | C++ 模板 | Rust FFI |
|------|----------|----------|
| 类型参数化 | `Stack<T>` | 手动实例化 |
| 实例化方式 | 编译器自动 | 手动为每种类型创建 |
| 类型安全 | 编译器保证 | 命名约定保证 |
| 代码复用 | 高 | 低（重复代码） |

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

Rust FFI: 类模板 = 为每种类型实例化独立结构
Stack<int> -> IntStack
Stack<double> -> DoubleStack
```

## 总结

- 类模板的 FFI 需要为每种类型创建独立结构
- C++ 内部可以使用 `std::stack<T>` 实现
- 导出时通过命名约定区分不同实例
- 这是模板 FFI 的标准做法