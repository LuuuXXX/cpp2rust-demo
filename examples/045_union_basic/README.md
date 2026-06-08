# 045_union_basic - 共用体

## C++ 特性

本示例展示 C++ 共用体（Union）的内存 overlay 特性及其在 FFI 中的处理方式。Union 所有成员共享同一块内存，节省空间但需要注意类型安全。

## C++ 代码

### union_basic.h

```cpp
#pragma once

#ifdef __cplusplus
extern "C" {
#endif

// 值类型
#define VALUE_TYPE_INT 0
#define VALUE_TYPE_FLOAT 1
#define VALUE_TYPE_STRING 2

// Variant 使用 union 存储不同类型的值
struct Variant* variant_new_int(int value);
int variant_get_int(struct Variant* self);
void variant_set_float(struct Variant* self, float value);

#ifdef __cplusplus
}
#endif
```

### union_basic.cpp

```cpp
#include "union_basic.h"

struct Variant {
    int type;
    union {
        int int_value;
        float float_value;
        char string_buffer[64];
    } data;
};

// 设置 float 时，int_value 的内存被覆盖
void variant_set_float(struct Variant* self, float value) {
    self->type = VALUE_TYPE_FLOAT;
    self->data.float_value = value;  // 覆盖 int_value 的内存
}
```

## Rust FFI 代码

```rust
hicc::cpp! {
    #include <cstddef>
    #include <cstdint>
    #include <iostream>
    #include <cstring>

    #include "union_basic.h"
}

hicc::import_class! {
    #[cpp(class = "IntFloatUnion", destroy = "union_delete")]
    pub class IntFloatUnion {}
}

hicc::import_class! {
    #[cpp(class = "Variant", destroy = "variant_delete")]
    pub class Variant {
        #[cpp(method = "int get_type() const")]
        fn get_type(&self) -> i32;

        #[cpp(method = "void set_int(int value)")]
        fn set_int(&mut self, value: i32);

        #[cpp(method = "void set_float(float value)")]
        fn set_float(&mut self, value: f32);

        #[cpp(method = "void set_string(const char* value)")]
        fn set_string(&mut self, value: *const i8);

        #[cpp(method = "int get_int() const")]
        fn get_int(&self) -> i32;

        #[cpp(method = "float get_float() const")]
        fn get_float(&self) -> f32;

        #[cpp(method = "const char* get_string() const")]
        fn get_string(&self) -> *const i8;
    }
}

hicc::import_lib! {
    #![link_name = "union_basic"]

    class IntFloatUnion;
    class Variant;

    #[cpp(func = "IntFloatUnion* union_new()")]
    fn union_new() -> IntFloatUnion;

    #[cpp(func = "Variant* variant_new_int(int)")]
    fn variant_new_int(value: i32) -> Variant;

    #[cpp(func = "Variant* variant_new_float(float)")]
    fn variant_new_float(value: f32) -> Variant;

    #[cpp(func = "Variant* variant_new_string(const char*)")]
    unsafe fn variant_new_string(value: *const i8) -> Variant;

    #[cpp(func = "int union_get_int(IntFloatUnion* u)")]
    fn union_get_int(u: *mut IntFloatUnion) -> i32;

    #[cpp(func = "float union_get_float(IntFloatUnion* u)")]
    fn union_get_float(u: *mut IntFloatUnion) -> f32;

    #[cpp(func = "void union_set_int(IntFloatUnion* u, int)")]
    unsafe fn union_set_int(u: *mut IntFloatUnion, value: i32);

    #[cpp(func = "void union_set_float(IntFloatUnion* u, float)")]
    unsafe fn union_set_float(u: *mut IntFloatUnion, value: f32);
}
```
## Union vs Struct

| 特性 | Union | Struct |
|------|-------|--------|
| 内存布局 | 所有成员共享 | 各成员独立 |
| 大小 | 最大成员大小 | 所有成员之和 |
| 同时访问 | 不安全 | 安全 |
| 用途 | 类型转换、节省内存 | 组合不同类型 |

## 内存 overlay 示例

```cpp
union IntFloat {
    int int_value;
    float float_value;
};

IntFloat u;
u.int_value = 0x41414141;      // 设置为 'AAAA'
printf("%f", u.float_value);    // 读取为浮点数
// 输出: 731306568.000000
```

## 构建方法

### C++ 编译

```bash
cd cpp
g++ -c union_basic.cpp -o union_basic.o
g++ -shared -fPIC union_basic.cpp -o libunion_basic.so
```

### Rust 编译

```bash
cd rust_hicc
cargo build
```

## FFI 对比分析

| 方面 | C++ | Rust |
|------|-----|------|
| 声明 | `union { int i; float f; }` | `#[repr(C)] union` |
| 内存共享 | 原生支持 | 通过 `#[repr(C)]` |
| 类型安全 | 运行时检查 | 编译期不保证 |
| FFI 传递 | 通过 opaque pointer | 通过 wrapper 结构 |

## 运行结果

```
=== 045_union_basic - Unions ===

--- Variant Demo ---
Type: INT, Value: 42
Type: FLOAT, Value: 3.14
Type: STRING, Value: Hello, Union!

--- Memory Overlay Demo ---
sizeof(int) = 4, sizeof(float) = 4
Set as int: 1094795585 (0x41414141)
Read as float: 12.078431 (bits: 0x41414141)

--- Summary ---
1. union all members share the same memory
2. Modifying one member affects other members
3. union size equals the largest member size
4. Often used to save memory or for type punning
5. FFI passes union via variant wrapper
```

## 总结

1. Union 所有成员共享同一块内存
2. 修改一个成员会影响其他成员的值
3. Union 大小等于最大成员的大小
4. 常用于节省内存或位级类型转换
5. FFI 中通过 variant 包装器传递 union
6. 使用时必须记录当前活跃的成员类型