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

```rust
hicc::cpp! {
    #include <stddef.h>
    #include <iostream>
    #include <functional>
    #include <algorithm>

    #include "lambda_basic.h"

    typedef int (*IntBinaryOp)(int, int);
}

hicc::import_class! {
    #[cpp(class = "LambdaWrapper", destroy = "lambda_wrapper_delete")]
    pub class LambdaWrapper {
        #[cpp(method = "int invoke(int a, int b)")]
        fn invoke(&mut self, a: i32, b: i32) -> i32;
    }
}

hicc::import_class! {
    #[cpp(class = "StateLambda", destroy = "state_lambda_delete")]
    pub class StateLambda {
        #[cpp(method = "int get_value() const")]
        fn get_value(&self) -> i32;

        #[cpp(method = "int add(int delta)")]
        fn add(&mut self, delta: i32) -> i32;
    }
}

hicc::import_class! {
    #[cpp(class = "Comparator", destroy = "comparator_delete")]
    pub class Comparator {
        #[cpp(method = "int compare(int a, int b) const")]
        fn compare(&self, a: i32, b: i32) -> i32;
    }
}

hicc::import_lib! {
    #![link_name = "lambda_basic"]

    class LambdaWrapper;
    class StateLambda;
    class Comparator;

    // cpp2rust-todo[FP]: 含函数指针参数，需确保回调符合 extern "C" 调用约定
    #[cpp(func = "LambdaWrapper* lambda_wrapper_new(int (*)(int, int))")]
    unsafe fn lambda_wrapper_new(fn_: unsafe extern "C" fn(i32, i32) -> i32) -> LambdaWrapper;

    #[cpp(func = "StateLambda* state_lambda_new(int)")]
    fn state_lambda_new(initial_value: i32) -> StateLambda;

    // cpp2rust-todo[FP]: 含函数指针参数，需确保回调符合 extern "C" 调用约定
    #[cpp(func = "Comparator* comparator_new(int (*)(int, int))")]
    unsafe fn comparator_new(cmp: unsafe extern "C" fn(i32, i32) -> i32) -> Comparator;

    #[cpp(func = "Comparator* comparator_new_add()")]
    fn comparator_new_add() -> Comparator;

    // cpp2rust-todo[FP]: 含函数指针参数，需确保回调符合 extern "C" 调用约定
    #[cpp(func = "int apply_operation(int, int, int (*)(int, int))")]
    unsafe fn apply_operation(a: i32, b: i32, op: unsafe extern "C" fn(i32, i32) -> i32) -> i32;

    // cpp2rust-todo[FP]: 含函数指针参数，需确保回调符合 extern "C" 调用约定
    #[cpp(func = "int apply_twice(int, int (*)(int, int))")]
    unsafe fn apply_twice(x: i32, op: unsafe extern "C" fn(i32, i32) -> i32) -> i32;

    #[cpp(func = "int add_impl(int, int)")]
    fn add_impl(a: i32, b: i32) -> i32;

    #[cpp(func = "int multiply_impl(int, int)")]
    fn multiply_impl(a: i32, b: i32) -> i32;

    #[cpp(func = "int max_impl(int, int)")]
    fn max_impl(a: i32, b: i32) -> i32;

    #[cpp(func = "LambdaWrapper* make_add_lambda()")]
    fn make_add_lambda() -> *mut LambdaWrapper;

    #[cpp(func = "LambdaWrapper* make_multiply_lambda()")]
    fn make_multiply_lambda() -> *mut LambdaWrapper;

    #[cpp(func = "LambdaWrapper* make_max_lambda()")]
    fn make_max_lambda() -> *mut LambdaWrapper;
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

## 运行结果

```
=== 039_lambda_basic - Lambda 表达式 ===

--- Direct function calls ---
add lambda called: 3 + 4
add_impl(3, 4) = 7
multiply lambda called: 3 * 4
multiply_impl(3, 4) = 12
max lambda called: 3 vs 4
max_impl(3, 4) = 4

--- LambdaWrapper Demo ---
add lambda called: 5 + 6
add invoke(5, 6) = 11
multiply lambda called: 5 * 6
multiply invoke(5, 6) = 30

--- StateLambda Demo ---
initial value = 10
add(5) = 15
add(3) = 18

--- Comparator Demo ---
add lambda called: 2 + 3
compare(2, 3) = 5

Rust FFI: Lambda 表达式映射
1. 函数指针可以通过 FFI 传递
2. 捕获状态的 lambda 需要包装在类中
3. 此示例展示基本的类封装模式
```

## 总结

- 无捕获的 lambda 可以作为函数指针传递
- 有状态的 lambda 需要包装在类中
- FFI 边界使用函数指针或 std::function
- Rust 闭包需要转换为函数指针
