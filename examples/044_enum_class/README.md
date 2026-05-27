# 044_enum_class - 强类型枚举

## C++ 特性

本示例展示 C++11 引入的 `enum class`（强类型枚举）如何在 FFI 中处理。强类型枚举不会隐式转换为整数，提供了类型安全。

## 重要说明

由于 hicc 的 `import_class!` 宏不支持命名空间类（如 `example::OperationResult`），本示例使用 **raw extern "C" + void\*** 模式来实现 FFI。

## C++ 代码

### enum_class.h

```cpp
#pragma once

#include <cstddef>
#include <cstdint>

// C++ enum class definitions (must be outside extern "C")
namespace example {

enum class ErrorCode : int {
    None = 0,
    InvalidInput = 1,
    OutOfMemory = 2,
    NotFound = 3,
    PermissionDenied = 4,
    Unknown = 99
};

enum class State : unsigned char {
    Idle = 0,
    Running = 1,
    Paused = 2,
    Stopped = 3
};

enum class Flags : unsigned int {
    None = 0,
    Read = 1,
    Write = 2,
    Execute = 4,
    All = 7
};

class OperationResult {
private:
    ErrorCode error_;
    State state_;
    Flags flags_;
public:
    OperationResult();
    ~OperationResult();
    void set_error(int code);
    int get_error() const;
    void set_state(unsigned char s);
    unsigned char get_state() const;
    void set_flags(unsigned int f);
    unsigned int get_flags() const;
};

}  // namespace example

#ifdef __cplusplus
extern "C" {
#endif

// FFI functions - 使用 void* 作为 opaque pointer
void* operation_result_new(void);
void operation_result_delete(void* p);
void operation_result_set_error(void* p, int error_code);
int operation_result_get_error(void* p);
void operation_result_set_state(void* p, unsigned char state);
unsigned char operation_result_get_state(void* p);
void operation_result_set_flags(void* p, unsigned int flags);
unsigned int operation_result_get_flags(void* p);
unsigned int combine_flags(unsigned int f1, unsigned int f2);
int has_flag(unsigned int flags, unsigned int flag);

#ifdef __cplusplus
}
#endif
```

### enum_class.cpp

```cpp
#include "enum_class.h"

namespace example {

OperationResult::OperationResult() : error_(ErrorCode::None), state_(State::Idle), flags_(Flags::None) {}

void OperationResult::set_error(int code) {
    error_ = static_cast<ErrorCode>(code);
}

int OperationResult::get_error() const {
    return static_cast<int>(error_);
}

// ... 其他方法实现

}  // namespace example

// FFI wrapper functions - 使用 void*
void* operation_result_new(void) {
    return new example::OperationResult();
}

void operation_result_delete(void* p) {
    delete static_cast<example::OperationResult*>(p);
}
```

## Rust FFI 代码

### main.rs

```rust
// 使用 void* opaque pointer 模式
type OperationResult = *mut std::ffi::c_void;

#[link(name = "enum_class")]
unsafe extern "C" {
    fn operation_result_new() -> OperationResult;
    fn operation_result_delete(p: OperationResult);
    fn operation_result_set_error(p: OperationResult, error_code: i32);
    fn operation_result_get_error(p: OperationResult) -> i32;
    fn operation_result_set_state(p: OperationResult, state: u8);
    fn operation_result_get_state(p: OperationResult) -> u8;
    fn operation_result_set_flags(p: OperationResult, flags: u32);
    fn operation_result_get_flags(p: OperationResult) -> u32;
    fn combine_flags(f1: u32, f2: u32) -> u32;
    fn has_flag(flags: u32, flag: u32) -> i32;
}

// Enum constants for Rust
pub const ERROR_NONE: i32 = 0;
pub const ERROR_INVALID_INPUT: i32 = 1;
pub const STATE_IDLE: u8 = 0;
pub const STATE_RUNNING: u8 = 1;
pub const FLAG_READ: u32 = 1;
```

## enum class vs enum

| 特性 | enum | enum class |
|------|------|------------|
| 类型安全 | 无（可隐式转int） | 强类型 |
| 作用域 | 枚举名在enum所在作用域 | 枚举名在enum内部 |
| 底层类型 | int | 可指定 |
| 隐式转换 | 可以 | 不可以 |

## 构建方法

### Rust 编译

```bash
cd rust_hicc
cargo build
cargo run
```

## FFI 对比分析

| 方面 | C++ enum class | Rust |
|------|----------------|------|
| 声明 | `enum class Foo : int` | `const FOO: i32 = 0;` |
| 作用域 | `Foo::Bar` | `FOO_BAR` (flat) |
| 转换 | `static_cast<int>` | 直接使用整数 |
| 类型安全 | 编译期检查 | 运行时检查 |

## 运行结果

```
=== 044_enum_class - 强类型枚举 ===

--- ErrorCode Demo ---
Error: InvalidInput (code=1)
Error: NotFound (code=3)

--- State Demo ---
State: Running (value=1)
State: Paused (value=2)

--- Flags Demo ---
Flags: 011 (read=true, write=true, execute=false)
Combined flags: 101

--- 总结 ---
1. enum class 是强类型，不会隐式转换为 int
2. 可以指定底层类型：enum class Foo : int
3. FFI 传递枚举值作为整数
4. Rust 端定义相应常量来模拟枚举
5. 强类型枚举更安全，避免枚举值混淆
```

## 总结

1. `enum class` 是强类型枚举，不会隐式转换为整数
2. 可以指定底层类型：`enum class Foo : int`
3. FFI 中通过整数传递枚举值
4. Rust 端定义相应的常量来模拟枚举
5. **使用 void\* + static_cast 模式处理命名空间类**
6. 强类型枚举更安全，避免枚举值混淆
