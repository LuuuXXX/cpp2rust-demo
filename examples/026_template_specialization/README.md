# 026_template_specialization - 模板特化（hicc 直出，去 shim）

## C++ 特性

本示例展示 C++ **模板特化**的 FFI 处理方式。通用类模板 `ValueHolder<T>` 提供默认实现，
全特化 `ValueHolder<std::string>` 行为不同。模板/特化本身没有可链接符号、不可裸绑定，
本示例将每个具体类型暴露为 idiomatic 命名空间类（`IntHolder`/`DoubleHolder` 走通用模板，
`StringHolder` 走特化版本），hicc 直出按普通类绑定其方法与 `make_unique` 工厂，无需
extern-C shim。

## C++ 代码

### template_specialization.h

```cpp
namespace template_specialization_ns {

// 通用类模板
template <typename T>
class ValueHolder {
    T value_;
public:
    explicit ValueHolder(T value) : value_(value) {}
    T get() const { return value_; }
    const char* describe() const { /* "ValueHolder<T>(generic)" */ }
};

// 全特化：ValueHolder<std::string>（构造函数私有 + friend StringHolder，
// 避免被 hicc 直出当作可独立实例化的普通类绑定）
template <>
class ValueHolder<std::string> {
    std::string value_;
    explicit ValueHolder(std::string value);
    friend class StringHolder;
public:
    const char* get() const { return value_.c_str(); }
    const char* describe() const { /* 含 length 信息 */ }
};

// 具体实例化类
class IntHolder    { ValueHolder<int>         impl_; /* ... */ };
class DoubleHolder { ValueHolder<double>      impl_; /* ... */ };
class StringHolder { ValueHolder<std::string> impl_; /* 走特化 */ };

} // namespace template_specialization_ns
```

## Rust FFI 代码

hicc 直出无需 extern-C shim，直接绑定 3 个具体类与 `make_unique` 工厂：

```rust
hicc::cpp! {
    #include "template_specialization.h"
}

hicc::import_class! {
    #[cpp(class = "template_specialization_ns::IntHolder")]
    pub class IntHolder {
        #[cpp(method = "int get() const")]
        pub fn get(&self) -> i32;
        #[cpp(method = "const char* describe() const")]
        pub fn describe(&self) -> *const i8;

        pub fn new(value: i32) -> Self { int_holder_new(value) }
    }
}

// DoubleHolder / StringHolder 同理

hicc::import_lib! {
    #![link_name = "template_specialization"]

    #[cpp(func = "std::unique_ptr<template_specialization_ns::IntHolder> hicc::make_unique<template_specialization_ns::IntHolder, int>(int&&)")]
    pub fn int_holder_new(value: i32) -> IntHolder;
    // double_holder_new / string_holder_new 同理
}
```

## FFI 对比分析

| 方面 | C++ | Rust FFI |
|------|-----|----------|
| 通用模板 | `ValueHolder<T>` | 暴露为 `IntHolder` / `DoubleHolder` |
| 全特化 | `ValueHolder<std::string>` | 暴露为 `StringHolder` |
| 行为差异 | 特化版 `describe()` 含 length | 调用结果不同 |
| 构造 | `IntHolder(int)` | `IntHolder::new(i32)`（`make_unique` 工厂） |

## 运行结果

```
=== 026_template_specialization - 模板特化 ===

ValueHolder<T>(generic)
  get(): 42

ValueHolder<T>(generic)
  get(): 3.14159

ValueHolder<std::string>(value="Hello, World!", length=13)
  get(): Hello, World!

Rust FFI: 通用模板与特化各自暴露为独立的具体类
通用版本: IntHolder, DoubleHolder（ValueHolder<T>）
全特化:  StringHolder（ValueHolder<std::string>）
```

## 冒烟测试

本示例包含集成冒烟测试（`rust_hicc/tests/smoke.rs`），验证生成的 Rust FFI 绑定可编译、
可链接 C++ 实现，且通用/特化行为差异正确。

### 测试用例

| 测试函数 | 验证内容 |
|---------|---------|
| `smoke_int_holder_get` / `smoke_double_holder_get` | 通用模板取值 |
| `smoke_int_holder_describe` / `smoke_double_holder_describe` | 通用模板 describe 标注 generic |
| `smoke_string_holder_get` | 特化版本取值 |
| `smoke_string_holder_specialized_describe` | 特化版本 describe 含 std::string / length |

### 运行方式

```bash
cd examples/026_template_specialization/rust_hicc
cargo test --test smoke
```

### 各平台支持

| 平台 | 状态 | 备注 |
|------|------|------|
| Linux (Ubuntu) | ✅ | CI `l-smoke` job 已覆盖 |
| Windows MinGW | ✅ | 支持 |

## 总结

- C++ 模板特化无法直接 FFI 导出（模板/特化本身没有可链接符号）
- 策略：将通用模板与特化各自的具体实例化暴露为 idiomatic 命名空间类
- hicc 直出按普通类绑定方法与 `make_unique` 工厂，无需 extern-C shim
- 特化类的私有构造 + friend 写法可避免工具误绑定带模板实参的类名
