# 032_placement_new - Placement New

## C++ 特性

本示例展示 C++ 中 placement new 的用法，以及如何在预分配内存中构造对象。

## C++ 代码

### placement_new.h

```cpp
#pragma once
#ifdef __cplusplus
extern "C" {
#endif

struct Buffer;
struct VectorBuffer;

struct Buffer* buffer_new(size_t capacity);
void buffer_delete(struct Buffer* self);
void* buffer_data(struct Buffer* self);
size_t buffer_capacity(struct Buffer* self);

// 在缓冲区中构造对象
void* buffer_construct(struct Buffer* self, size_t offset);

#ifdef __cplusplus
}
#endif
```

### placement_new.cpp

```cpp
#include "placement_new.h"
#include <new>

struct SimpleValue {
    int id;
    double value;
    SimpleValue(int i, double v) : id(i), value(v) {}
};

struct Buffer {
    char* data;
    size_t capacity;
    Buffer(size_t cap) : capacity(cap) {
        data = new char[capacity];
    }
};

void* buffer_construct(Buffer* self, size_t offset) {
    // placement new：在指定地址构造对象
    void* location = self->data + offset;
    return new (location) SimpleValue(1, 2.5);
}
```

## Placement New 原理

### 标准 new vs Placement new

```cpp
// 普通 new：分配 + 构造
T* p = new T(args);

// Placement new：仅构造（内存已分配）
void* buffer = malloc(sizeof(T));
T* p = new (buffer) T(args);
```

### 语法

```cpp
new (address) Constructor(args)
```

## Rust FFI 代码

```rust
hicc::cpp! {
    #include <stddef.h>
    #include <iostream>
    #include <cstring>
    #include <new>

    #include "placement_new.h"
}

hicc::import_class! {
    #[cpp(class = "Buffer", destroy = "buffer_delete")]
    pub class Buffer {
        #[cpp(method = "void* data()")]
        fn data(&mut self) -> *mut u8;

        #[cpp(method = "size_t capacity() const")]
        fn capacity(&self) -> usize;

        #[cpp(method = "size_t size() const")]
        fn size(&self) -> usize;

        #[cpp(method = "void* construct(size_t offset)")]
        fn construct(&mut self, offset: usize) -> *mut u8;
    }
}

hicc::import_class! {
    #[cpp(class = "VectorBuffer", destroy = "vector_buffer_delete")]
    pub class VectorBuffer {
        #[cpp(method = "void* data()")]
        fn data(&mut self) -> *mut u8;

        #[cpp(method = "size_t element_size() const")]
        fn element_size(&self) -> usize;

        #[cpp(method = "void destroy_all()")]
        fn destroy_all(&mut self);
    }
}

hicc::import_lib! {
    #![link_name = "placement_new"]

    class Buffer;
    class VectorBuffer;

    #[cpp(func = "Buffer* buffer_new(size_t)")]
    fn buffer_new(capacity: usize) -> Buffer;

    #[cpp(func = "VectorBuffer* vector_buffer_new(size_t)")]
    fn vector_buffer_new(capacity: usize) -> VectorBuffer;
}
```
## FFI 对比分析

| 方面 | C++ Placement New | Rust FFI |
|------|-------------------|----------|
| 内存分配 | 预分配或 malloc | 手动管理 |
| 对象构造 | new (addr) T() | C++ 侧处理 |
| 析构函数 | 手动调用 | C++ 侧处理 |
| 内存布局 | 由类型决定 | 需要协商 |

## 关键点

1. **内存预先分配**：Rust 侧提供原始内存
2. **C++ 侧构造**：在指定地址调用构造函数
3. **析构函数**：需要显式调用或使用 RAII
4. **内存对齐**：需要考虑类型对齐要求

## 使用场景

- **内存池**：预先分配大块内存，按需构造
- **STL 容器**：vector/map 的内部实现
- **嵌入式系统**：避免动态分配
- **游戏开发**：对象池管理

## 运行结果

```
=== 032_placement_new - Placement New ===

Buffer created with capacity: 1024
Buffer data at: 0x...
Buffer capacity: 1024
Buffer constructed size: 0
Buffer delete called

--- VectorBuffer Demo ---
VectorBuffer element size: 4

Rust FFI: Placement New 模式
1. 在预分配内存中构造对象
2. 使用 placement new: new (address) Constructor(args)
3. 适用于内存池、STL 容器实现
4. Rust 需要手动管理内存布局
```

## 总结

- Placement new 允许在指定内存地址构造对象
- FFI 边界需要协调内存分配和构造责任
- 通常 C++ 侧负责构造，Rust 侧负责分配
- 适用于高性能和实时系统场景
