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

```rust
hicc::cpp! {
    #include <stddef.h>
    #include <iostream>
    #include <stdexcept>
    #include <cstring>

    #include "exception_basic.h"
}

hicc::import_class! {
    #[cpp(class = "Calculator", destroy = "calculator_delete")]
    pub class Calculator {
        #[cpp(method = "void clear_exception()")]
        fn clear_exception(&mut self);

        #[cpp(method = "int get_exception()")]
        fn get_exception(&mut self) -> i32;

        #[cpp(method = "int divide(int a, int b)")]
        fn divide(&mut self, a: i32, b: i32) -> i32;

        #[cpp(method = "int string_to_int(const char* str)")]
        fn string_to_int(&mut self, str: *const i8) -> i32;
    }
}

hicc::import_lib! {
    #![link_name = "exception_basic"]

    class Calculator;

    #[cpp(func = "Calculator* calculator_new()")]
    fn calculator_new() -> Calculator;
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

## 运行结果

```
=== 042_exception_basic - Exception Handling ===

--- Division Tests ---
10 / 2 = 5
  10 / 2: No exception

Testing division by zero:
10 / 0 = 0 (returns 0, check exception)
  10 / 0: Runtime error exception

After clearing exception:
20 / 4 = 5
  20 / 4: No exception

--- String to Int Tests ---
string_to_int("123") = 123
  string_to_int("123"): No exception
string_to_int("abc") = 0 (returns 0, check exception)
  string_to_int("abc"): Invalid argument exception

--- Summary ---
1. C++ exceptions CANNOT propagate across FFI boundary
2. Common FFI pattern: set error code, return error value
3. Check exception/error state after each call
4. Clear exception state before next operation
5. Never throw in FFI boundary - use error codes instead
```

## 总结

1. C++ 异常不能直接跨 FFI 边界传播到 Rust
2. 常见模式：
   - 通过错误码返回值
   - 通过全局/结构体存储异常信息
   - 通过回调函数报告错误
3. `hicc::Exception<T>` 提供类型安全的异常封装
4. 每次调用后应检查异常状态
5. 异常状态需要在操作前清除