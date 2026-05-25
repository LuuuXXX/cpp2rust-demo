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

### main.rs

```rust
hicc::import_lib! {
    #![link_name = "array_basic"]

    struct IntArray5;

    #[cpp(func = "struct IntArray5* int_array5_new(void)")]
    fn int_array5_new() -> *mut IntArray5;

    #[cpp(func = "int int_array5_get(struct IntArray5*, size_t)")]
    unsafe fn int_array5_get(arr: *mut IntArray5, index: usize) -> i32;

    #[cpp(func = "int* int_array5_data(struct IntArray5*)")]
    unsafe fn int_array5_data(arr: *mut IntArray5) -> *mut i32;
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

## 总结

- std::array 是固定大小的数组容器
- FFI 边界需要分离不同大小的类型
- 与 Rust 的 `[T; N]` 语义相似
- 适用于固定大小的集合
