# 006_class_basic - 基础类

## C++ 特性

本示例展示如何将 C++ 类通过 FFI 导出为 Rust 使用。使用"构造/销毁函数对"模式模拟构造函数和析构函数。

## C++ 代码

### class_basic.h

```cpp
#pragma once
#ifdef __cplusplus
extern "C" {
#endif

// Opaque 指针：不暴露类内部结构
struct Counter;

// 工厂函数（模拟构造函数）
struct Counter* counter_new(void);

// 销毁函数（模拟析构函数）
void counter_delete(struct Counter* self);

// 成员函数
int counter_get(struct Counter* self);
void counter_increment(struct Counter* self);
void counter_decrement(struct Counter* self);

#ifdef __cplusplus
}
#endif
```

### class_basic.cpp

```cpp
#include "class_basic.h"
#include <iostream>

struct Counter {
    int value;
};

struct Counter* counter_new(void) {
    std::cout << "Counter created" << std::endl;
    return new Counter{0};
}

void counter_delete(struct Counter* self) {
    if (self) {
        std::cout << "Counter deleted (value was " << self->value << ")" << std::endl;
        delete self;
    }
}

int counter_get(struct Counter* self) {
    return self->value;
}

void counter_increment(struct Counter* self) {
    ++self->value;
}

void counter_decrement(struct Counter* self) {
    --self->value;
}
```

## Opaque 指针模式

### 什么是 Opaque 指针

Opaque（不透明）指针是指针类型前向声明，但不暴露其实际结构：

```cpp
struct Counter;  // 前向声明，不完整类型
```

外部代码只能通过函数操作该类型，无法访问其内部成员。

### 优势

1. **封装**：隐藏内部实现
2. **ABI 稳定**：内部结构变化不影响 FFI
3. **内存安全**：由库负责内存分配/释放

## Rust FFI 代码

### main.rs

```rust
hicc::import_lib! {
    #![link_name = "class_basic"]

    class Counter;

    #[cpp(func = "struct Counter* counter_new(void)")]
    fn counter_new() -> *mut Counter;

    #[cpp(func = "void counter_delete(struct Counter*)")]
    unsafe fn counter_delete(self_: *mut Counter);

    #[cpp(func = "int counter_get(struct Counter*)")]
    unsafe fn counter_get(self_: *mut Counter) -> i32;

    #[cpp(func = "void counter_increment(struct Counter*)")]
    unsafe fn counter_increment(self_: *mut Counter);

    #[cpp(func = "void counter_decrement(struct Counter*)")]
    unsafe fn counter_decrement(self_: *mut Counter);
}
```

### Safe Wrapper

```rust
struct Counter {
    ptr: *mut Counter,
}

impl Counter {
    fn new() -> Self {
        let ptr = counter_new();
        Self { ptr }
    }

    fn get(&self) -> i32 {
        unsafe { counter_get(self.ptr) }
    }

    fn increment(&mut self) {
        unsafe { counter_increment(self.ptr) }
    }
}

impl Drop for Counter {
    fn drop(&mut self) {
        unsafe { counter_delete(self.ptr) }
    }
}
```

## 关键点

### C++ 类到 FFI 的映射

| C++ 概念 | FFI 模式 |
|----------|----------|
| 构造函数 | `Class* class_new()` |
| 析构函数 | `void class_delete(Class*)` |
| 成员函数 | `ReturnType class_method(Class*, ...)` |
| this 指针 | 作为第一个参数传递 |
| 访问控制 | FFI 边界不可见 |

### Rust 端内存管理

1. **raw pointer**：`*mut T` 表示独占访问
2. **Drop trait**：自动调用 `counter_delete`
3. **borrow checker**：确保没有悬垂指针

## 构建方法

### C++ 编译

```bash
cd cpp
g++ -c class_basic.cpp -o class_basic.o
g++ -shared -fPIC class_basic.cpp -o libclass_basic.so
```

### Rust 编译

```bash
cd rust_hicc
cargo build
cargo run
```

## 运行结果

```
Counter created
Initial value: 0
After 3 increments: 3
After 1 decrement: 2
Counter deleted (value was 2)

Rust FFI: Basic class operations completed!
```

## 总结

1. **Opaque 指针模式**：隐藏 C++ 类内部结构
2. **构造/销毁函数对**：模拟构造函数和析构函数
3. **成员函数第一个参数是 this**：符合 C 调用约定
4. **Rust wrapper**：添加 safe interface 和 Drop 实现
