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

```rust
hicc::cpp! {
    #include <iostream>
    #include <cstring>
    #include <cstdlib>
    #include <cstdio>

    #include "template_specialization.h"
}

hicc::import_class! {
    #[cpp(class = "IntHolder", destroy = "intholder_delete")]
    pub class IntHolder {
        #[cpp(method = "int get() const")]
        fn get(&self) -> i32;

        #[cpp(method = "const char* describe() const")]
        fn describe(&self) -> *const i8;
    }
}

hicc::import_class! {
    #[cpp(class = "DoubleHolder", destroy = "doubleholder_delete")]
    pub class DoubleHolder {
        #[cpp(method = "double get() const")]
        fn get(&self) -> f64;

        #[cpp(method = "const char* describe() const")]
        fn describe(&self) -> *const i8;
    }
}

hicc::import_class! {
    #[cpp(class = "StringHolder", destroy = "stringholder_delete")]
    pub class StringHolder {
        #[cpp(method = "const char* get() const")]
        fn get(&self) -> *const i8;

        #[cpp(method = "const char* describe() const")]
        fn describe(&self) -> *const i8;
    }
}

hicc::import_lib! {
    #![link_name = "template_specialization"]

    class IntHolder;
    class DoubleHolder;
    class StringHolder;

    #[cpp(func = "IntHolder* intholder_new(int)")]
    fn intholder_new(value: i32) -> IntHolder;

    #[cpp(func = "DoubleHolder* doubleholder_new(double)")]
    fn doubleholder_new(value: f64) -> DoubleHolder;

    #[cpp(func = "StringHolder* stringholder_new(const char*)")]
    unsafe fn stringholder_new(value: *const i8) -> StringHolder;
}
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