# 037_array_basic - std::array

## C++ 特性

本示例展示 C++ `std::array`（固定大小数组容器）的基本操作。

## C++ 代码

### array_basic.h

```cpp
#pragma once
#ifdef __cplusplus
extern "C" {
#endif

struct IntArray5;

struct IntArray5* int_array5_new(void);
struct IntArray5* int_array5_new_from(const int* values);
void int_array5_delete(struct IntArray5* self);

size_t int_array5_size(struct IntArray5* self);
int int_array5_get(struct IntArray5* self, size_t index);
void int_array5_set(struct IntArray5* self, size_t index, int value);
int* int_array5_data(struct IntArray5* self);

#ifdef __cplusplus
}
#endif
```

### array_basic.cpp

```cpp
#include "array_basic.h"
#include <array>

struct IntArray5 {
    std::array<int, 5> data;
};

struct IntArray5* int_array5_new(void) {
    return new IntArray5();
}

size_t int_array5_size(struct IntArray5* self) {
    return self->data.size();
}

int int_array5_get(struct IntArray5* self, size_t index) {
    return self->data[index];
}

int* int_array5_data(struct IntArray5* self) {
    return self->data.data();
}
```

## std::array 特点

| 特性 | 说明 |
|------|------|
| 固定大小 | 大小在编译时确定 |
| 栈分配 | 通常在栈上分配 |
| 无动态扩容 | 不能改变大小 |
| 聚合语义 | 支持迭代器 |

### 与 std::vector 对比

| 特性 | std::array | std::vector |
|------|------------|-------------|
| 大小 | 固定 | 动态 |
| 内存位置 | 栈/静态 | 堆 |
| 性能 | 更快 | 有分配开销 |
| 灵活性 | 低 | 高 |

## Rust FFI 代码

```rust
hicc::cpp! {
    #include <stddef.h>
    #include <iostream>
    #include <array>
    #include <string>
    #include <cstring>

    #include "array_basic.h"
}

hicc::import_class! {
    #[cpp(class = "IntArray5", destroy = "int_array5_delete")]
    pub class IntArray5 {
        #[cpp(method = "size_t size() const")]
        fn size(&self) -> usize;

        #[cpp(method = "bool empty() const")]
        fn empty(&self) -> bool;

        #[cpp(method = "void set(size_t i, int val)")]
        fn set(&mut self, i: usize, val: i32);

        #[cpp(method = "int get(size_t i) const")]
        fn get(&self, i: usize) -> i32;

        #[cpp(method = "int at(size_t i) const")]
        fn at(&self, i: usize) -> i32;

        #[cpp(method = "int* data()")]
        fn data(&mut self) -> *mut i32;
    }
}

hicc::import_class! {
    #[cpp(class = "DoubleArray3", destroy = "double_array3_delete")]
    pub class DoubleArray3 {
        #[cpp(method = "size_t size() const")]
        fn size(&self) -> usize;
    }
}

hicc::import_class! {
    #[cpp(class = "StringArray4", destroy = "string_array4_delete")]
    pub class StringArray4 {
        #[cpp(method = "size_t size() const")]
        fn size(&self) -> usize;
    }
}

hicc::import_lib! {
    #![link_name = "array_basic"]

    class IntArray5;
    class DoubleArray3;
    class StringArray4;

    #[cpp(func = "IntArray5* int_array5_new()")]
    fn int_array5_new() -> IntArray5;

    #[cpp(func = "IntArray5* int_array5_new_from(const int*)")]
    fn int_array5_new_from(values: *const i32) -> IntArray5;

    #[cpp(func = "DoubleArray3* double_array3_new()")]
    fn double_array3_new() -> DoubleArray3;

    #[cpp(func = "DoubleArray3* double_array3_new_from(const double*)")]
    fn double_array3_new_from(values: *const f64) -> DoubleArray3;

    #[cpp(func = "StringArray4* string_array4_new()")]
    fn string_array4_new() -> StringArray4;
}
```
## FFI 对比分析

| 方面 | C++ std::array | Rust FFI |
|------|----------------|----------|
| 大小 | 编译时模板参数 | 运行时返回 |
| 元素类型 | 模板参数 | 分离的函数 |
| 访问方式 | `v[i]` 或 `v.at(i)` | get(i) 函数 |
| 内存位置 | 栈或静态 | 堆（通过 new） |

## 关键点

1. **固定大小**：大小不能改变
2. **编译时确定**：类型包含大小信息
3. **栈分配**：更高效的内存布局
4. **数据指针**：`data()` 返回连续内存

## 运行结果

```
=== 037_array_basic - std::array ===

--- IntArray5 Demo ---
Size: 5
Empty: false
Elements:
  [0] = 0
  [1] = 10
  [2] = 20
  [3] = 30
  [4] = 40
at(2) = 20
Data pointer: 0x...

--- IntArray5 from values Demo ---
Size: 5
Elements:
  [0] = 1
  [1] = 2
  [2] = 3
  [3] = 4
  [4] = 5

Rust FFI: std::array 映射
1. std::array 是固定大小的数组容器
2. 大小在编译时确定（模板参数）
3. data() 返回原始指针用于批量访问
4. 与 Rust 的 [T; N] 数组语义相似
```

## 冒烟测试

本示例包含集成冒烟测试（`rust_hicc/tests/smoke.rs`），验证生成的 Rust FFI 绑定可编译、
可链接 C++ 实现，且基本行为正确。

### 测试用例

| 测试函数 | 验证内容 |
|---------|---------|
| `smoke_int_array5_new` | `int_array5_new()` 后 `size()` = 5 |
| `smoke_int_array5_set_get` | `set(2, 42)` 后 `get(2)` = 42 |
| `smoke_int_array5_at` | `at(0)` 返回数组首元素 |
| `smoke_int_array5_new_from` | `int_array5_new_from(vals, 5)` 创建预填充数组 |
| `smoke_double_array3_type_available` | `DoubleArray3` 类型可用性断言 |

### 运行方式

```bash
cd examples/037_array_basic/rust_hicc
cargo test --test smoke
```

### 各平台支持

| 平台 | 状态 | 备注 |
|------|------|------|
| Linux (Ubuntu) | ✅ | CI `l-smoke` job 已覆盖 |
| macOS | ✅ | 支持 |
| Windows MinGW | ✅ | 支持 |

## 冒烟测试

本示例在 `.cpp2rust/array_basic/rust/tests/smoke.rs` 中包含以下冒烟测试，CI 通过 `l-smoke` job 自动运行：

| 测试函数 | 验证内容 |
|----------|----------|
| `smoke_int_array5_new` | `int_array5_new()` 分配成功，返回非空指针 |
| `smoke_int_array5_set_get` | `int_array5_set` / `int_array5_get` 写入读取值正确 |
| `smoke_int_array5_at` | `int_array5_at` 越界返回 `-1`（哨兵值） |
| `smoke_int_array5_new_from` | `int_array5_new_from` 从整型数组初始化，索引读取正确 |
| `smoke_double_array3_type_available` | `DoubleArray3` 类型可用（编译期可见性验证） |

运行单个冒烟测试：

```bash
cd examples/037_array_basic/.cpp2rust/array_basic/rust
cargo test smoke_int_array5_set_get -- --nocapture
```

## 总结

- std::array 是固定大小的数组容器
- FFI 边界需要分离不同大小的类型
- 与 Rust 的 `[T; N]` 语义相似
- 适用于固定大小的集合
