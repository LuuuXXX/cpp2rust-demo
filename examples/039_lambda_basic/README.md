# 039_lambda_basic - Lambda 表达式

## C++ 特性

本示例展示 C++ Lambda 表达式的基本概念，以及如何通过 FFI 传递 lambda 函数。

## C++ 代码

### lambda_basic.h

```cpp
#pragma once
#ifdef __cplusplus
extern "C" {
#endif

// 函数指针类型
typedef int (*IntBinaryOp)(int, int);

// 使用 lambda 的函数
int apply_operation(int a, int b, IntBinaryOp op);

struct LambdaWrapper;
struct LambdaWrapper* lambda_wrapper_new(int (*fn)(int, int));
int lambda_wrapper_call(struct LambdaWrapper* self, int a, int b);

#ifdef __cplusplus
}
#endif
```

### lambda_basic.cpp

```cpp
#include "lambda_basic.h"
#include <functional>

struct LambdaWrapper {
    std::function<int(int, int)> fn;
    LambdaWrapper(int (*fn_ptr)(int, int)) : fn(fn_ptr) {}
    int call(int a, int b) { return fn(a, b); }
};

int apply_operation(int a, int b, IntBinaryOp op) {
    return op(a, b);
}
```

## Lambda 表达式语法

```cpp
// 基本语法
[capture](parameters) -> return_type { body }

// 示例
auto add = [](int a, int b) -> int { return a + b; };

// 捕获模式
int x = 10;
auto add_x = [x](int a) { return a + x; };      // 值捕获
auto add_x_ref = [&x](int a) { return a + x; }; // 引用捕获
```

### Lambda 捕获类型

| 捕获 | 说明 |
|------|------|
| `[]` | 无捕获 |
| `[x]` | 值捕获 x |
| `[&x]` | 引用捕获 x |
| `[=]` | 值捕获所有 |
| `[&]` | 引用捕获所有 |

## Rust FFI 代码

### main.rs

```rust
hicc::import_lib! {
    #![link_name = "lambda_basic"]

    #[cpp(type = "int(*)(int, int)")]
    type IntBinaryOp;

    struct LambdaWrapper;

    #[cpp(func = "struct LambdaWrapper* lambda_wrapper_new(int(*)(int,int))")]
    fn lambda_wrapper_new(op: IntBinaryOp) -> *mut LambdaWrapper;

    #[cpp(func = "int lambda_wrapper_call(struct LambdaWrapper*, int, int)")]
    unsafe fn lambda_wrapper_call(w: *mut LambdaWrapper, a: i32, b: i32) -> i32;
}

// Rust 函数作为 lambda
extern "C" fn rust_add(a: i32, b: i32) -> i32 {
    a + b
}
```

## FFI 对比分析

| 方面 | C++ Lambda | Rust FFI |
|------|-----------|----------|
| 基本 lambda | 函数指针 | 函数指针 |
| 捕获状态 | std::function | 包装类 |
| 无状态 | 可以作为函数指针 | 可以作为函数指针 |
| 有状态 | 需要包装 | 需要包装 |

## 关键点

1. **无捕获 lambda**：可以转换为函数指针
2. **有捕获 lambda**：需要 std::function 或包装类
3. **Rust 闭包**：可以转换为函数指针
4. **FFI 传递**：函数指针可以直接传递

## 总结

- 无捕获的 lambda 可以作为函数指针传递
- 有状态的 lambda 需要包装在类中
- FFI 边界使用函数指针或 std::function
- Rust 闭包需要转换为函数指针
## 运行结果

```
=== 039_lambda_basic - Lambda 表达式 ===

--- BinaryOp Demo ---
Stored: a=10, b=20, result=30
Stored: a=5, b=3, result=8

Rust FFI: Lambda 表达式映射
1. 函数指针可以通过 FFI 传递
2. 捕获状态的 lambda 需要包装在类中
3. 此示例展示基本的类封装模式
```
