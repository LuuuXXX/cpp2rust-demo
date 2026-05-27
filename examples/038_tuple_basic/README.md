# 038_tuple_basic - std::tuple

## C++ 特性

本示例展示 C++ `std::tuple`（异构容器）的基本操作。

## C++ 代码

### tuple_basic.h

```cpp
#pragma once
#ifdef __cplusplus
extern "C" {
#endif

struct Tuple2;
struct Tuple3;

struct Tuple2* tuple2_new(int first, const char* second);
void tuple2_delete(struct Tuple2* self);

int tuple2_get_first(struct Tuple2* self);
const char* tuple2_get_second(struct Tuple2* self);

struct Tuple3* tuple3_new(int first, double second, const char* third);
void tuple3_delete(struct Tuple3* self);

int tuple3_get_first(struct Tuple3* self);
double tuple3_get_second(struct Tuple3* self);
const char* tuple3_get_third(struct Tuple3* self);

#ifdef __cplusplus
}
#endif
```

### tuple_basic.cpp

```cpp
#include "tuple_basic.h"
#include <tuple>
#include <string>

struct Tuple2 {
    std::tuple<int, std::string> data;
};

struct Tuple2* tuple2_new(int first, const char* second) {
    return new Tuple2(first, second ? second : "");
}

int tuple2_get_first(struct Tuple2* self) {
    return std::get<0>(self->data);
}

const char* tuple2_get_second(struct Tuple2* self) {
    static std::string temp;
    temp = std::get<1>(self->data);
    return temp.c_str();
}
```

## std::tuple 特点

| 特性 | 说明 |
|------|------|
| 异构类型 | 可以包含不同类型的元素 |
| 固定大小 | 编译时确定大小 |
| 索引访问 | `std::get<N>(t)` |
| 解包 | `std::tie` |

### 与 Rust tuple 对比

```cpp
// C++
std::tuple<int, double, std::string> t(1, 2.0, "hello");
int i = std::get<0>(t);
```

```rust
// Rust
let t = (1, 2.0, "hello");
let i = t.0;
```

## Rust FFI 代码

### main.rs

```rust
hicc::import_lib! {
    #![link_name = "tuple_basic"]

    struct Tuple2;

    #[cpp(func = "struct Tuple2* tuple2_new(int, const char*)")]
    fn tuple2_new(first: i32, second: *const i8) -> *mut Tuple2;

    #[cpp(func = "int tuple2_get_first(struct Tuple2*)")]
    unsafe fn tuple2_get_first(t: *mut Tuple2) -> i32;

    #[cpp(func = "const char* tuple2_get_second(struct Tuple2*)")]
    unsafe fn tuple2_get_second(t: *mut Tuple2) -> *const i8;
}
```

## FFI 对比分析

| 方面 | C++ std::tuple | Rust FFI |
|------|-----------------|----------|
| 元素访问 | `std::get<N>` | 独立 getter 函数 |
| 类型参数 | 编译时确定 | 运行时分离类型 |
| 字符串 | `std::string` | `const char*` |
| 内存管理 | 自动 | 需要手动释放 |

## 关键点

1. **异构容器**：可包含不同类型的元素
2. **编译时大小**：大小不能动态改变
3. **索引访问**：`std::get<index>`
4. **FFI 映射**：需要为每个位置提供独立的 getter

## 运行结果

```
=== 038_tuple_basic - std::tuple ===

--- Tuple2 (int, string) Demo ---
Tuple2: first=42, second=hello

--- Tuple3 (int, double, string) Demo ---
Tuple3: first=100, second=3.14159, third=world

--- Tuple4 (int, double, string, int) Demo ---
Tuple4 elements:
  [0] = 1
  [1] = 2.71828
  [2] = tuple
  [3] = 4

--- Helper Functions Demo ---
make_int_string_pair: (10, pair)

Rust FFI: std::tuple 映射
1. std::tuple 是异构容器的编译时固定版本
2. 通过 std::get<N>(tuple) 访问元素
3. FFI 需要为每个元素类型提供独立的 getter 函数
4. 字符串等复杂类型需要额外的内存管理
```

## 总结

- std::tuple 是固定大小的异构容器
- FFI 边界需要为每个元素提供独立的访问函数
- 与 Rust 的 tuple 语义相似
- 适用于编译时已知结构的数据
