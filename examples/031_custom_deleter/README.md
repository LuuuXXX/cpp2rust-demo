# 031_custom_deleter - 自定义删除器

## C++ 特性

本示例展示 C++ 中自定义删除器（Custom Deleter）的概念，以及如何通过 FFI 传递给 Rust 使用。

## C++ 代码

### custom_deleter.h

```cpp
#pragma once
#ifdef __cplusplus
extern "C" {
#endif

// 文件句柄结构体
struct FileHandle;

// 创建文件句柄，第三个参数是自定义删除器函数指针
FileHandle* file_open(const char* filename, const char* mode,
                      void (*deleter)(struct FileHandle*));

// 关闭文件句柄
void file_close(FileHandle* handle);

#ifdef __cplusplus
}
#endif
```

### custom_deleter.cpp

```cpp
#include "custom_deleter.h"
#include <iostream>
#include <cstdio>

struct FileHandle {
    FILE* file;
    void (*deleter)(struct FileHandle*);
    const char* filename;
};

// 默认删除器
void default_file_deleter(struct FileHandle* handle) {
    if (handle) {
        if (handle->file) fclose(handle->file);
        delete handle;
    }
}

// 带日志的自定义删除器
void logging_file_deleter(struct FileHandle* handle) {
    std::cout << "[LOG] Closing: " << handle->filename << std::endl;
    if (handle->file) fclose(handle->file);
    delete handle;
}

FileHandle* file_open(const char* filename, const char* mode,
                      void (*deleter)(struct FileHandle*)) {
    FileHandle* handle = new FileHandle();
    handle->file = fopen(filename, mode);
    handle->deleter = deleter ? deleter : default_file_deleter;
    handle->filename = filename;
    return handle;
}

void file_close(FileHandle* handle) {
    if (handle && handle->deleter) {
        handle->deleter(handle);
    }
}
```

## 自定义删除器模式

### 什么是自定义删除器

自定义删除器是一种函数对象，用于控制资源的释放方式：

```cpp
std::unique_ptr<T, Deleter> ptr;
```

### 常见的删除器类型

1. **默认删除器**：`delete ptr`
2. **自定义函数**：`void (*deleter)(T*)`
3. **lambda 表达式**
4. **std::function**

## Rust FFI 代码

### main.rs

```rust
hicc::import_lib! {
    #![link_name = "custom_deleter"]

    struct FileHandle;

    #[cpp(func = "struct FileHandle* file_open_default(const char*, const char*)")]
    fn file_open_default(filename: *const i8, mode: *const i8) -> *mut FileHandle;

    #[cpp(func = "void file_close(struct FileHandle*)")]
    unsafe fn file_close(handle: *mut FileHandle);
}
```

### Safe Wrapper

```rust
struct FileHandle {
    ptr: *mut FileHandle,
}

impl FileHandle {
    fn new(filename: &str, mode: &str) -> Option<Self> {
        let filename = CString::new(filename).ok()?;
        let mode = CString::new(mode).ok()?;
        let ptr = unsafe { file_open_default(filename.as_ptr(), mode.as_ptr()) };
        if ptr.is_null() { None } else { Some(Self { ptr }) }
    }
}

impl Drop for FileHandle {
    fn drop(&mut self) {
        unsafe { file_close(self.ptr) }
    }
}
```

## FFI 对比分析

| 方面 | C++ 自定义删除器 | Rust FFI |
|------|------------------|----------|
| 删除器传递 | 函数指针作为模板参数 | 函数指针作为函数参数 |
| 调用时机 | 析构时自动调用 | 手动调用 close() |
| 类型安全 | 模板参数类型安全 | 类型擦除（void*） |
| 灵活性 | 编译时确定 | 运行时可更换 |

## 关键点

1. **函数指针传递**：删除器作为函数指针传递
2. **类型擦除**：C++ 模板提供类型安全，FFI 需要手动保证
3. **生命周期**：Rust 侧需要确保删除器在对象生命周期内有效
4. **错误处理**：FFI 边界需要处理 nullptr 情况

## 运行结果

```
=== 031_custom_deleter - 自定义删除器 ===

Written 22 bytes
[DEFAULT] Closing file: test_default.txt

Rust FFI: 自定义删除器模式
1. C++ 允许传递函数指针作为删除器
2. 删除器在对象销毁时自动调用
3. Rust 可以传入自己的清理函数
4. 适用于文件、内存、网络连接等资源
```

## 总结

- 自定义删除器是 C++ RAII 的延伸
- FFI 边界需要显式传递删除器
- Rust 侧通过 wrapper 类型提供安全接口
- 适用于文件、数据库连接、网络 socket 等资源管理
