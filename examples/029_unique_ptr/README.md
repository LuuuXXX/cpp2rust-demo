# 029_unique_ptr - std::unique_ptr

## C++ 特性

本示例展示 C++ `std::unique_ptr`（独占所有权的智能指针）的 FFI 处理方式。

## C++ 代码

### unique_ptr.cpp

```cpp
// 模拟 std::unique_ptr 的包装类
template<typename T>
class UniquePtr {
    T* ptr_;
public:
    explicit UniquePtr(T* p = nullptr) : ptr_(p) {}
    ~UniquePtr() { delete ptr_; }

    // 禁用拷贝
    UniquePtr(const UniquePtr&) = delete;

    // 移动语义
    UniquePtr(UniquePtr&& other) noexcept : ptr_(other.ptr_) {
        other.ptr_ = nullptr;
    }
};
```

## Rust FFI 方案

### main.rs

```rust
// 策略 1: C++ 侧管理 unique_ptr
#[cpp(func = "void uniquebuffer_delete(struct UniqueBuffer*)")]
unsafe fn uniquebuffer_delete(self_: *mut UniqueBuffer);

// 策略 2: 使用 hicc-std 提供的 unique_ptr 包装
// hicc-std 有 std::unique_ptr 的安全 Rust 包装
```

## FFI 对比分析

| 方面 | C++ unique_ptr | Rust FFI |
|------|-----------------|----------|
| 所有权 | 独占（唯一） | 通过 delete 函数释放 |
| 移动语义 | `std::move` | 传递指针所有权 |
| 自动释放 | 析构函数 | Rust 必须显式调用 delete |
| use_count | 始终 1 | 始终 1（无共享） |

## unique_ptr 特点

1. **独占所有权** - 同一时刻只有一个 unique_ptr 拥有对象
2. **自动释放** - 析构时自动 delete
3. **不可拷贝** - 只能移动
4. **轻量级** - 只包含一个指针

## FFI 策略

### 策略 1: C++ 侧管理（推荐）

```cpp
// C++ 导出
void buffer_delete(Buffer*);  // 释放 unique_ptr
```

### 策略 2: 使用 hicc-std（仅限 Linux / Windows）

hicc-std 库提供了 `std::unique_ptr` 的安全 Rust 包装，可以直接使用。

> **注意**：`hicc-std 0.2` 在 macOS Apple Clang 下存在编译问题（链接 `stdc++` 与
> Apple Clang 使用的 `libc++` 不兼容），仅在 Linux / Windows 平台可用。
> 跨平台项目推荐使用策略 1（C++ 侧自定义包装类 + `extern "C"` 接口）。


## Rust FFI 代码

```rust
hicc::cpp! {
    #include <string>
    #include <iostream>
    #include <memory>
    #include <cstring>

    #include "unique_ptr.h"
}

hicc::import_class! {
    #[cpp(class = "UniqueBuffer", destroy = "uniquebuffer_delete")]
    pub class UniqueBuffer {
        #[cpp(method = "int getSize() const")]
        fn get_size(&self) -> i32;

        #[cpp(method = "char* getData()")]
        fn get_data(&mut self) -> *mut i8;

        #[cpp(method = "int useCount() const")]
        fn use_count(&self) -> i32;
    }
}

hicc::import_class! {
    #[cpp(class = "Processor", destroy = "processor_delete")]
    pub class Processor {
        #[cpp(method = "char* process(const char* input)")]
        fn process(&mut self, input: *const i8) -> *mut i8;
    }
}

hicc::import_lib! {
    #![link_name = "unique_ptr"]

    class UniqueBuffer;
    class Processor;

    #[cpp(func = "UniqueBuffer* uniquebuffer_new(int)")]
    fn uniquebuffer_new(size: i32) -> UniqueBuffer;

    #[cpp(func = "Processor* processor_new()")]
    fn processor_new() -> Processor;
}
```

## 运行结果

```
=== 029_unique_ptr - std::unique_ptr ===

Buffer size: 16
Buffer data: 
Use count: 1 (unique_ptr always = 1)

Processed result: Hello, unique_ptr! [processed]

Rust FFI: unique_ptr 的处理方式
1. C++ 侧管理对象生命周期
2. Rust 侧通过 FFI 函数调用管理
3. 相当于 Rust 的 Box<T>

hicc-std 提供了 std::unique_ptr 的安全 Rust 包装
```

## 总结

- `unique_ptr` 的 FFI 相对简单（独占所有权）
- 需要导出 release/delete 函数
- Rust 侧调用后不再拥有指针
- 跨平台推荐使用 C++ 侧自定义包装类（策略 1）；在 Linux/Windows 上也可使用 hicc-std 的安全包装（策略 2）