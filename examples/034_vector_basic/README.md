# 034_vector_basic - std::vector

## C++ 特性

本示例展示 C++ `std::vector` 的基本操作，以及如何通过 FFI 导出给 Rust 使用。

## C++ 代码

### vector_basic.h

```cpp
#pragma once
#ifdef __cplusplus
extern "C" {
#endif

struct IntVector;

struct IntVector* int_vector_new(void);
void int_vector_delete(struct IntVector* self);

size_t int_vector_size(struct IntVector* self);
size_t int_vector_capacity(struct IntVector* self);
int int_vector_empty(struct IntVector* self);

void int_vector_push_back(struct IntVector* self, int value);
int int_vector_get(struct IntVector* self, size_t index);
void int_vector_set(struct IntVector* self, size_t index, int value);
int* int_vector_data(struct IntVector* self);

#ifdef __cplusplus
}
#endif
```

### vector_basic.cpp

```cpp
#include "vector_basic.h"
#include <vector>

struct IntVector {
    std::vector<int> data;
};

struct IntVector* int_vector_new() {
    return new IntVector();
}

void int_vector_delete(struct IntVector* self) {
    delete self;
}

size_t int_vector_size(struct IntVector* self) {
    return self->data.size();
}

void int_vector_push_back(struct IntVector* self, int value) {
    self->data.push_back(value);
}

int int_vector_get(struct IntVector* self, size_t index) {
    return self->data[index];
}
```

## std::vector 特点

| 操作 | C++ | Rust 等效 |
|------|-----|-----------|
| 创建 | `vector<T> v` | `Vec::new()` |
| 添加 | `v.push_back(x)` | `v.push(x)` |
| 大小 | `v.size()` | `v.len()` |
| 访问 | `v[i]` | `v[i]` |
| 数据指针 | `v.data()` | `v.as_ptr()` |
| 清空 | `v.clear()` | `v.clear()` |

## Rust FFI 代码

```rust
hicc::cpp! {
    #include <stddef.h>
    #include <iostream>
    #include <vector>
    #include <string>
    #include <cstring>

    #include "vector_basic.h"
}

hicc::import_class! {
    #[cpp(class = "IntVector", destroy = "int_vector_delete")]
    pub class IntVector {
        #[cpp(method = "void push_back(int val)")]
        fn push_back(&mut self, val: i32);

        #[cpp(method = "int get(size_t i) const")]
        fn get(&self, i: usize) -> i32;

        #[cpp(method = "void set(size_t i, int val)")]
        fn set(&mut self, i: usize, val: i32);

        #[cpp(method = "size_t size() const")]
        fn size(&self) -> usize;

        #[cpp(method = "bool empty() const")]
        fn empty(&self) -> bool;

        #[cpp(method = "size_t capacity() const")]
        fn capacity(&self) -> usize;

        #[cpp(method = "void reserve(size_t n)")]
        fn reserve(&mut self, n: usize);

        #[cpp(method = "int* data()")]
        fn data(&mut self) -> *mut i32;

        #[cpp(method = "void clear()")]
        fn clear(&mut self);
    }
}

hicc::import_class! {
    #[cpp(class = "StringVector", destroy = "string_vector_delete")]
    pub class StringVector {
        #[cpp(method = "size_t size() const")]
        fn size(&self) -> usize;
    }
}

hicc::import_lib! {
    #![link_name = "vector_basic"]

    class IntVector;
    class StringVector;

    #[cpp(func = "IntVector* int_vector_new()")]
    fn int_vector_new() -> IntVector;

    #[cpp(func = "StringVector* string_vector_new()")]
    fn string_vector_new() -> StringVector;
}
```
## FFI 对比分析

| 方面 | C++ std::vector | Rust FFI |
|------|-----------------|----------|
| 内存管理 | 自动扩容 | C++ 侧管理 |
| 元素访问 | `v[i]` | `get(i)` 函数 |
| 迭代 | 迭代器 | `data()` + 长度 |
| 字符串 | `std::string` | C 字符串传递 |

## 关键点

1. **Opaque 指针**：vector 内部结构对 Rust 隐藏
2. **函数式 API**：push_back/get/set 等操作
3. **容量管理**：C++ 侧自动处理 reallocation
4. **字符串处理**：需要 CString 转换

## 运行结果

```
=== 034_vector_basic - std::vector ===

--- IntVector Demo ---
Empty: true
Size: 5, Capacity: 8
Elements:
  [0] = 0
  [1] = 10
  [2] = 20
  [3] = 30
  [4] = 40
After set [2] = 999: 999
Raw data pointer: 0x...
After clear, size: 0

Rust FFI: std::vector 映射
1. Opaque 指针隐藏 vector 内部结构
2. push_back/get/set 等价于 Rust 的 push/get/index
3. size()/capacity() 提供容器信息
4. data() 获取原始指针用于批量操作

Note: StringVector example omitted due to FFI complexity with const char*
```

## 总结

- std::vector 是最常用的 STL 容器
- FFI 边界需要显式函数调用
- Rust 侧通过 unsafe 函数操作
- 推荐封装为安全的 Rust Vec 类型
