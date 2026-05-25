# 005_variadic_functions - 可变参数函数

## C++ 特性

本示例展示 C++ 可变参数函数（variadic functions），如 `printf` 系列函数。

## C++ 代码

### variadic_functions.h

```cpp
#pragma once
#ifdef __cplusplus
extern "C" {
#endif

// 可变参数函数
int sum(int count, ...);

// 固定参数 + 可变参数
int print_formatted(const char* format, ...);

#ifdef __cplusplus
}
#endif
```

### variadic_functions.cpp

```cpp
#include "variadic_functions.h"
#include <cstdarg>
#include <cstdio>

int sum(int count, ...) {
    va_list args;
    va_start(args, count);
    int total = 0;
    for (int i = 0; i < count; ++i) {
        total += va_arg(args, int);
    }
    va_end(args);
    return total;
}

int print_formatted(const char* format, ...) {
    va_list args;
    va_start(args, format);
    int result = vprintf(format, args);
    va_end(args);
    return result;
}
```

## 可变参数函数与 FFI

### C++ 可变参数机制

C++ 使用 `<cstdarg>` 头文件提供的宏处理可变参数：
- `va_list`：参数列表类型
- `va_start(args, last_fixed)`：开始遍历
- `va_arg(args, type)`：获取下一个参数
- `va_end(args)`：结束遍历

### FFI 挑战

可变参数在 FFI 中非常棘手：
1. **Rust 没有可变参数函数**：Rust 的 `println!` 是宏，不是函数
2. **类型安全缺失**：C 运行时不知道参数类型
3. **跨语言边界问题**：调用约定可能不同

### 解决方案

在 Rust 端使用 `unsafe` 和可变参数列表：

```rust
unsafe fn sum(count: i32, ...) -> i32;
unsafe fn print_formatted(format: *const i8, ...) -> i32;
```

调用时必须手动传递正确类型的参数：

```rust
sum(3, 1i32, 2i32, 3i32)  // 必须是 i32
```

## Rust FFI 代码

### main.rs

```rust
hicc::import_lib! {
    #![link_name = "variadic_functions"]

    #[cpp(func = "int sum(int, ...)")]
    unsafe fn sum(count: i32, ...) -> i32;

    #[cpp(func = "int print_formatted(const char*, ...)")]
    unsafe fn print_formatted(format: *const i8, ...) -> i32;
}

fn main() {
    unsafe {
        let result = sum(3, 1i32, 2i32, 3i32);
        println!("sum(3, 1, 2, 3) = {}", result);

        let format = b"Hello, %s! Number: %d\n\0".as_ptr() as *const i8;
        let name = b"World\0".as_ptr() as *const i8;
        print_formatted(format, name, 42i32);
    }
}
```

## 关键点

### 可变参数的 FFI 限制

| 问题 | 影响 |
|------|------|
| 类型不安全 | 必须手动匹配参数类型 |
| 无编译时检查 | 错误类型导致未定义行为 |
| 调用约定差异 | 不同平台参数传递方式可能不同 |

### Rust 端调用规范

```rust
// 正确：显式指定类型
sum(3, 1i32, 2i32, 3i32)

// 错误：隐式类型可能导致问题
sum(3, 1, 2, 3)  // 取决于平台，可能是 i32 或 i64
```

## 构建方法

### C++ 编译

```bash
cd cpp
g++ -c variadic_functions.cpp -o variadic_functions.o
g++ -shared -fPIC variadic_functions.cpp -o libvariadic_functions.so
```

### Rust 编译

```bash
cd rust_hicc
cargo build
cargo run
```

## 运行结果

```
sum(3, 1, 2, 3) = 6
sum(5, 10, 20, 30, 40, 50) = 150
Hello, World! Number: 42

Rust FFI: Variadic functions handled!
```

## 总结

1. **可变参数是 C 风格特性**：FFI 边界上类型不安全
2. **Rust 需要 unsafe**：必须用 unsafe 块调用
3. **类型必须手动匹配**：编译器无法检查参数类型正确性
4. **建议**：尽量避免可变参数 FFI，使用固定参数或数组代替
