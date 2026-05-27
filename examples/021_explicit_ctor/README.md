# 021_explicit_ctor - explicit 构造函数

## C++ 特性

本示例展示 C++ 中 `explicit` 关键字的作用：防止单参数构造函数的隐式类型转换。

## C++ 代码

### explicit_ctor.h

```cpp
struct Widget* widget_fromInt(int value);  // explicit 构造
struct Widget* widget_fromDouble(double value);  // explicit 构造
```

### explicit_ctor.cpp

```cpp
// implicit 构造函数
Widget* widget_new(int value) {
    // 可以隐式调用: Widget w = 42;
}

// explicit 构造函数
Widget* widget_fromInt(int value) {
    // 必须显式调用: Widget w(42) 或 Widget w = Widget(42);
}
```

## Rust FFI 代码

### main.rs

```rust
// implicit 构造函数
fn widget_new(value: i32) -> *mut Widget;

// explicit 构造函数
fn widget_fromInt(value: i32) -> *mut Widget;
fn widget_fromDouble(value: f64) -> *mut Widget;
```

## FFI 对比分析

| 方面 | C++ | Rust FFI |
|------|-----|----------|
| explicit 作用 | 编译时防止隐式转换 | 无影响，FFI 调用都是显式的 |
| 函数调用 | 可能隐式：`Widget w = 42;` | 始终显式：`widget_fromInt(42)` |
| FFI 签名 | `Widget* widget_fromInt(int)` | `fn widget_fromInt(value: i32) -> *mut Widget` |

## 运行结果

```
=== 021_explicit_ctor - explicit 构造函数 ===

C++ explicit 关键字防止隐式类型转换

Created with implicit ctor: value = 42

Created with explicit int ctor: value = 100
Created with explicit double ctor: value = 3

Rust FFI: explicit 不影响 FFI - 只是禁止隐式转换
在 FFI 中，所有构造函数都是显式调用的
```

## 总结

- `explicit` 是编译时检查，不影响运行时 FFI 调用
- 在 FFI 边界，所有调用都是显式的
- FFI 层不需要特别处理 explicit 关键字