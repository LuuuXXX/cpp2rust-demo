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

### main.rs

```rust
hicc::import_lib! {
    #![link_name = "placement_new"]

    struct Buffer;

    #[cpp(func = "struct Buffer* buffer_new(size_t)")]
    fn buffer_new(capacity: usize) -> *mut Buffer;

    #[cpp(func = "void* buffer_data(struct Buffer*)")]
    unsafe fn buffer_data(buf: *mut Buffer) -> *mut std::ffi::c_void;

    #[cpp(func = "void buffer_delete(struct Buffer*)")]
    unsafe fn buffer_delete(buf: *mut Buffer);
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

## 总结

- Placement new 允许在指定内存地址构造对象
- FFI 边界需要协调内存分配和构造责任
- 通常 C++ 侧负责构造，Rust 侧负责分配
- 适用于高性能和实时系统场景
