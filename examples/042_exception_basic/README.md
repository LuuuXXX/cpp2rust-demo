# 042_exception_basic - 异常处理

## C++ 特性

本示例展示如何在 FFI 边界处理 C++ 异常。由于异常不能直接跨 FFI 边界传播，我们使用错误码和异常状态模式。

## C++ 代码

### exception_basic.h

```cpp
#pragma once

#ifdef __cplusplus
extern "C" {
#endif

// 异常状态码
#define EXCEPTION_NONE 0
#define EXCEPTION_INVALID_ARGUMENT 1
#define EXCEPTION_OUT_OF_RANGE 2
#define EXCEPTION_RUNTIME_ERROR 3

struct Calculator;

struct Calculator* calculator_new(void);
void calculator_delete(struct Calculator* self);
int calculator_get_exception(struct Calculator* self);
void calculator_clear_exception(struct Calculator* self);
int calculator_divide(struct Calculator* self, int a, int b);

#ifdef __cplusplus
}
#endif
```

### exception_basic.cpp

```cpp
#include "exception_basic.h"
#include <stdexcept>

struct Calculator {
    ExceptionInfo last_exception;

    int divide(int a, int b) {
        clear_exception();
        if (b == 0) {
            // 捕获 C++ 异常，转换为错误码
            last_exception.set(EXCEPTION_RUNTIME_ERROR, "division by zero");
            return 0;
        }
        return a / b;
    }
};
```

## Rust FFI 代码

### main.rs

```rust
hicc::import_lib! {
    const EXCEPTION_NONE: i32 = 0;
    const EXCEPTION_RUNTIME_ERROR: i32 = 3;

    struct Calculator;

    #[cpp(func = "int calculator_divide(struct Calculator*, int, int)")]
    unsafe fn calculator_divide(c: *mut Calculator, a: i32, b: i32) -> i32;
}

fn main() {
    let calc = calculator_new();

    // 调用可能抛出异常的函数
    let result = unsafe { calculator_divide(calc, 10, 0) };

    // 检查异常状态
    let code = unsafe { calculator_get_exception(calc) };
    if code != EXCEPTION_NONE {
        println!("Exception occurred: {:?}", code);
    }
}
```

## 构建方法

### C++ 编译

```bash
cd cpp
g++ -c exception_basic.cpp -o exception_basic.o
g++ -shared -fPIC exception_basic.cpp -o libexception_basic.so
```

### Rust 编译

```bash
cd rust_hicc
cargo build
```

## FFI 对比分析

| 方面 | C++ | Rust |
|------|-----|------|
| 异常传播 | `throw` / `catch` | `Result<T, E>` |
| 异常检测 | `try` / `catch` | `match` 或 `?` |
| 异常传递 | 直接传播 | 通过错误码模拟 |
| 多异常类型 | 不同异常类 | 枚举变体 |

## 总结

1. C++ 异常不能直接跨 FFI 边界传播到 Rust
2. 常见模式：
   - 通过错误码返回值
   - 通过全局/结构体存储异常信息
   - 通过回调函数报告错误
3. `hicc::Exception<T>` 提供类型安全的异常封装
4. 每次调用后应检查异常状态
5. 异常状态需要在操作前清除