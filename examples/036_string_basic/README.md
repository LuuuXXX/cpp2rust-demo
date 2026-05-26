# 036_string_basic - std::string

## C++ 特性

本示例展示 C++ `std::string` 的基本操作，以及如何通过 FFI 导出给 Rust 使用。

## C++ 代码

### string_basic.h

```cpp
#pragma once
#ifdef __cplusplus
extern "C" {
#endif

struct String;

struct String* string_new_from(const char* str);
void string_delete(struct String* self);

size_t string_size(struct String* self);
const char* string_c_str(struct String* self);

struct String* string_concat(struct String* self, const char* other);
void string_append(struct String* self, const char* other);

int string_compare(struct String* self, const char* other);
int string_equals(struct String* self, const char* other);

#ifdef __cplusplus
}
#endif
```

### string_basic.cpp

```cpp
#include "string_basic.h"
#include <string>

struct String {
    std::string data;
};

struct String* string_new_from(const char* str) {
    return new String(str ? str : "");
}

void string_delete(struct String* self) {
    delete self;
}

size_t string_size(struct String* self) {
    return self->data.size();
}

const char* string_c_str(struct String* self) {
    return self->data.c_str();
}

struct String* string_concat(struct String* self, const char* other) {
    return new String(self->data + other);
}

void string_append(struct String* self, const char* other) {
    self->data += other;
}

int string_equals(struct String* self, const char* other) {
    return self->data == other;
}
```

## std::string 特点

| 操作 | C++ | Rust 等效 |
|------|-----|-----------|
| 创建 | `string s(str)` | `CString::new()` |
| 大小 | `s.size()` | `s.len()` |
| C 字符串 | `s.c_str()` | `c_str()` |
| 连接 | `s + other` | `format!()` |
| 比较 | `s == other` | `s == other` |

## Rust FFI 代码

### main.rs

```rust
hicc::import_lib! {
    #![link_name = "string_basic"]

    struct String;

    #[cpp(func = "struct String* string_new_from(const char*)")]
    fn string_new_from(s: *const i8) -> *mut String;

    #[cpp(func = "void string_delete(struct String*)")]
    unsafe fn string_delete(s: *mut String);

    #[cpp(func = "const char* string_c_str(struct String*)")]
    unsafe fn string_c_str(s: *mut String) -> *const i8;

    #[cpp(func = "void string_append(struct String*, const char*)")]
    unsafe fn string_append(s: *mut String, other: *const i8);
}
```

### Safe Wrapper

```rust
struct CppString {
    ptr: *mut String,
}

impl CppString {
    fn from_rust(s: &str) -> Option<Self> {
        let cstr = CString::new(s).ok()?;
        let ptr = unsafe { string_new_from(cstr.as_ptr()) };
        if ptr.is_null() { None } else { Some(Self { ptr }) }
    }

    fn as_str(&self) -> &str {
        let c_str = unsafe { CStr::from_ptr(string_c_str(self.ptr)) };
        c_str.to_str().unwrap_or("")
    }
}

impl Drop for CppString {
    fn drop(&mut self) {
        unsafe { string_delete(self.ptr) }
    }
}
```

## FFI 对比分析

| 方面 | C++ std::string | Rust FFI |
|------|-----------------|----------|
| 内存管理 | 自动 | C++ 侧管理 |
| 字符串内容 | 内部存储 | `c_str()` 获取 |
| 可变性 | 可修改 | 通过函数修改 |
| 编码 | UTF-8（通常） | 字节数组 |

## 关键点

1. **Opaque 指针**：字符串内部结构对 Rust 隐藏
2. **C 字符串转换**：`c_str()` 返回 const char*
3. **内存管理**：Rust 侧通过 Drop trait 释放
4. **CString 转换**：Rust 字符串转为 C 字符串

## 总结

- std::string 是 C++ 标准字符串类型
- FFI 边界使用 `c_str()` 和 `CString`
- Rust 需要处理 UTF-8 编码
- 建议封装为安全的 Rust String 类型
## 运行结果

```
=== 036_string_basic - std::string ===

--- Creation Demo ---
Created: "Hello"
Size: 5, Length: 5
Empty: false

--- Comparison Demo ---
Compare with 'Hello': 0
Equals 'Hello': true

--- Concatenation Demo ---
After append: "Hello, World!"

--- Case Conversion Demo ---
To upper: "HELLO WORLD"
To lower: "hello world"

Rust FFI: std::string 映射
1. C++ 字符串映射为 opaque 指针
2. 字符串内容通过 c_str() 获取
3. 修改操作直接在原字符串上进行
4. CString 用于 Rust 到 C 的转换
```
