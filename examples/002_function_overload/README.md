# 002_function_overload - 函数重载

## C++ 特性

本示例展示 C++ 函数重载特性：同名函数，不同参数类型或个数。

## C++ 代码

### function_overload.h

```cpp
#pragma once
#ifdef __cplusplus
extern "C" {
#endif

// 函数重载：不同的参数类型
int add_int(int a, int b);
double add_double(double a, double b);
const char* add_strings(const char* a, const char* b);

// 重载：不同的参数个数
int sum3(int a, int b, int c);

#ifdef __cplusplus
}
#endif
```

### function_overload.cpp

```cpp
#include "function_overload.h"
#include <iostream>
#include <cstring>

int add_int(int a, int b) {
    std::cout << "add_int(" << a << ", " << b << ")" << std::endl;
    return a + b;
}

double add_double(double a, double b) {
    std::cout << "add_double(" << a << ", " << b << ")" << std::endl;
    return a + b;
}

const char* add_strings(const char* a, const char* b) {
    static char result[256];
    snprintf(result, sizeof(result), "%s%s", a, b);
    return result;
}

int sum3(int a, int b, int c) {
    return a + b + c;
}
```

## Rust FFI 代码

```rust
hicc::cpp! {
    #include "function_overload.h"
}

hicc::import_lib! {
    #![link_name = "function_overload"]

    #[cpp(func = "int add_int(int, int)")]
    fn add_int(a: i32, b: i32) -> i32;

    #[cpp(func = "double add_double(double, double)")]
    fn add_double(a: f64, b: f64) -> f64;

    #[cpp(func = "const char* add_strings(const char*, const char*)")]
    unsafe fn add_strings(a: *const i8, b: *const i8) -> *const i8;

    #[cpp(func = "int sum3(int, int, int)")]
    fn sum3(a: i32, b: i32, c: i32) -> i32;
}
```
## 关键点

### C++ 函数重载与 FFI

C++ 支持函数重载，但 C 没有这个特性。因此在 FFI 时：
- C++ 编译器会将重载函数进行 **名字修饰 (Name Mangling)**
- `extern "C"` 告诉编译器不要进行名字修饰
- 每个重载函数必须有**不同的 C 链接符号**

### 名字修饰对比

| C++ 函数 | C 链接名 | Rust 调用 |
|----------|----------|-----------|
| `add_int(int, int)` | `add_int` | `add_int(a, b)` |
| `add_double(double, double)` | `add_double` | `add_double(a, b)` |
| `sum3(int, int, int)` | `sum3` | `sum3(a, b, c)` |

### Rust 端处理

Rust 没有重载概念，所以：
- 每个 C++ 重载函数映射到**不同的 Rust 函数**
- 通过完整的 C++ 函数签名区分
- 字符串参数需要 `unsafe` 块处理原始指针

## 构建方法

### C++ 编译

```bash
cd cpp
g++ -c function_overload.cpp -o function_overload.o
g++ -shared -fPIC function_overload.cpp -o libfunction_overload.so
```

### Rust 编译

```bash
cd rust_hicc
cargo build
cargo run
```

## 运行结果

```
add_int(1, 2)
add_int result: 3
add_double(1.5, 2.5)
add_double result: 4
add_strings("Hello", " World")
add_strings result: Hello World
sum3(1, 2, 3)
sum3 result: 6

Rust FFI: All overloads called successfully!
```

## FFI 对比分析

| 方面 | C++ | Rust |
|------|-----|------|
| 函数重载 | 编译器处理 | 每个签名单独声明 |
| 名字修饰 | 自动（无 extern "C"） | N/A |
| 字符串传递 | `const char*` | `*const i8` |
| 安全性 | 原生 | `unsafe` 块包裹 |

## 注意事项

1. **重载不是多态**：重载在编译时解析，多态在运行时通过虚函数实现
2. **FFI 边界**：重载函数在 FFI 边界必须有不同的链接名
3. **unsafe 必要性**：涉及指针操作必须用 `unsafe`
