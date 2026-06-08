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

```rust
hicc::cpp! {
    #include <iostream>

    #include "class_basic.h"
}

hicc::import_class! {
    #[cpp(class = "Counter", destroy = "counter_delete")]
    pub class Counter {
        #[cpp(method = "int get() const")]
        fn get(&self) -> i32;

        #[cpp(method = "void increment()")]
        fn increment(&mut self);

        #[cpp(method = "void decrement()")]
        fn decrement(&mut self);
    }
}

hicc::import_lib! {
    #![link_name = "class_basic"]

    class Counter;

    #[cpp(func = "Counter* counter_new()")]
    fn counter_new() -> Counter;
}
```
## 关键点

### C++ 类到 hicc FFI 的映射

| C++ 概念 | hicc 映射方式 |
|----------|--------------|
| 构造函数 | `import_lib!` 中 `fn counter_new() -> Counter`（返回 owned 类型） |
| 析构函数 | `import_class!` 的 `destroy = "counter_delete"` 属性自动管理 |
| const 成员函数 | `fn get(&self) -> i32`（`&self`） |
| 非 const 成员函数 | `fn increment(&mut self)`（`&mut self`） |
| 访问控制 | FFI 边界不可见，hicc 通过 opaque pointer 封装 |

### hicc 内存管理

1. **owned 类型**：`counter_new() -> Counter` 返回 hicc 管理的 owned 对象（非原始指针）
2. **自动析构**：`destroy = "counter_delete"` 使 hicc 在 Drop 时自动调用 `counter_delete`
3. **安全引用**：`&self` / `&mut self` 保证 borrow checker 的安全性

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
Initial value: 0
After 3 increments: 3
After 1 decrement: 2

Rust FFI: Basic class operations completed!
```

## 总结

1. **Opaque 指针模式**：hicc 通过 `class Counter;` 前向声明隐藏 C++ 类内部结构
2. **owned 返回**：`fn counter_new() -> Counter` 返回 hicc owned 对象，析构时自动调用 `counter_delete`
3. **安全方法绑定**：`import_class!` 将 C++ 成员方法映射为 `&self`/`&mut self` 的 Rust 方法
4. **零 unsafe 调用**：hicc 将不安全的 FFI 封装为安全 Rust API
