# 026_template_specialization - 模板偏特化

## C++ 特性

本示例展示 C++ 模板偏特化的 FFI 处理方式。每个模板特化都需要独立导出。

## C++ 代码

### template_specialization.cpp

```cpp
// 通用版本 ValueHolder<T>
struct IntHolder { int value; };
struct DoubleHolder { double value; };

// char* 特化版本 - 专门处理字符串
struct StringHolder {
    char* value;
    int length;
};
```

## Rust FFI 代码

### main.rs

```rust
// 每个特化版本 = 独立结构
struct IntHolder;
struct DoubleHolder;
struct StringHolder;

// IntHolder 方法
fn intholder_new(value: i32) -> *mut IntHolder;
fn intholder_get(self_: *mut IntHolder) -> i32;

// StringHolder 方法
fn stringholder_new(value: *const i8) -> *mut StringHolder;
fn stringholder_get(self_: *mut StringHolder) -> *const i8;
```

## FFI 对比分析

| 方面 | C++ 模板 | Rust FFI |
|------|----------|----------|
| 通用版本 | `ValueHolder<T>` | `IntHolder`, `DoubleHolder` |
| 偏特化 | `ValueHolder<char*>` | `StringHolder` |
| 特化检测 | 编译器自动选择 | 手动区分 |
| 字符串处理 | 特殊实现 | 独立结构处理 |

## 运行结果

```
=== 026_template_specialization - 模板偏特化 ===

IntHolder(value=42)
  get(): 42

DoubleHolder(value=3.14159)
  get(): 3.14159

StringHolder(value="Hello, World!", length=13)
  get(): Hello, World!

Rust FFI: 每个模板特化是独立的结构
通用版本: IntHolder, DoubleHolder
偏特化: StringHolder (处理 char*)
```

## 总结

- 模板偏特化在 FFI 中需要为每个特化创建独立结构
- char* 特化通常需要特殊处理（内存管理）
- 命名约定用于区分不同版本
- 每个特化版本的内部实现可能完全不同