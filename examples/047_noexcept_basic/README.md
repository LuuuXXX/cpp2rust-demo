# 047_noexcept_basic - noexcept

## C++ 特性

本示例展示 C++ `noexcept` 异常规格说明如何在 FFI 中体现。`noexcept` 声明函数不会抛出异常，允许编译器进行优化。

## C++ 代码

### noexcept_basic.h

```cpp
#pragma once

#ifdef __cplusplus
extern "C" {
#endif

// noexcept 函数
int noexcept_add(int a, int b) noexcept;
int noexcept_multiply(int a, int b) noexcept;

// 可能抛出异常的函数
int throwing_divide(int a, int b);

// 移动构造函数标记为 noexcept
struct NoexceptMover;
struct NoexceptMover* noexcept_mover_move(struct NoexceptMover* other) noexcept;

#ifdef __cplusplus
}
#endif
```

### noexcept_basic.cpp

```cpp
#include "noexcept_basic.h"

// noexcept 移动构造函数
struct NoexceptMover {
    int value;

    NoexceptMover(NoexceptMover&& other) noexcept : value(other.value) {
        other.value = 0;
    }
};
```

## noexcept 的作用

1. **编译优化**：编译器知道函数不会抛异常，可以进行更多优化
2. **移动语义**：STL 容器在重新分配内存时使用 `noexcept` 移动
3. **析构函数**：C++11 起所有析构函数隐式 `noexcept`

## noexcept 运算符

```cpp
// C++17
static_assert(noexcept(some_func()));  // 编译期检查
```

## 构建方法

### C++ 编译

```bash
cd cpp
g++ -c noexcept_basic.cpp -o noexcept_basic.o
g++ -shared -fPIC noexcept_basic.cpp -o libnoexcept_basic.so
```

### Rust 编译

```bash
cd rust_hicc
cargo build
```

## FFI 对比分析

| 方面 | C++ | Rust |
|------|-----|------|
| 异常规格 | `noexcept` | `panic = "abort"` 或 `Result` |
| 移动语义 | `noexcept MoveConstr` | `impl Drop` + `mem::forget` |
| ABI 影响 | 否（仅优化提示） | 不适用 |


## Rust FFI 代码

```rust
hicc::cpp! {
    #include <cstddef>
    #include <iostream>
    #include <stdexcept>
    #include <utility>

    #include "noexcept_basic.h"
}

hicc::import_class! {
    #[cpp(class = "NoexceptMover", destroy = "noexcept_mover_delete")]
    pub class NoexceptMover {
        #[cpp(method = "int get_value() const")]
        fn get_value(&self) -> i32;
    }
}

hicc::import_lib! {
    #![link_name = "noexcept_basic"]

    class NoexceptMover;

    #[cpp(func = "NoexceptMover* noexcept_mover_new(int)")]
    fn noexcept_mover_new(value: i32) -> NoexceptMover;

    #[cpp(func = "int noexcept_add(int, int)")]
    fn noexcept_add(a: i32, b: i32) -> i32;

    #[cpp(func = "int noexcept_multiply(int, int)")]
    fn noexcept_multiply(a: i32, b: i32) -> i32;

    #[cpp(func = "int throwing_divide(int, int)")]
    fn throwing_divide(a: i32, b: i32) -> i32;

    // cpp2rust-todo[FP]: 含函数指针参数，需确保回调符合 extern "C" 调用约定
    #[cpp(func = "int check_noexcept(int (*)(int, int))")]
    unsafe fn check_noexcept(fn_: unsafe extern "C" fn(i32, i32) -> i32) -> i32;

    #[cpp(func = "int conditional_abs(int)")]
    fn conditional_abs(value: i32) -> i32;

    #[cpp(func = "NoexceptMover* noexcept_mover_move(NoexceptMover* other)")]
    unsafe fn noexcept_mover_move(other: *mut NoexceptMover) -> *mut NoexceptMover;

    // cpp2rust-todo[FP]: 含函数指针参数，需确保回调符合 extern "C" 调用约定
    #[cpp(func = "int is_noexcept(int (*)(int, int))")]
    unsafe fn is_noexcept(fn_: unsafe extern "C" fn(i32, i32) -> i32) -> i32;
}
```

## 运行结果

```
=== 047_noexcept_basic - noexcept ===

--- noexcept Functions ---
noexcept_add(10, 20) = 30
noexcept_multiply(6, 7) = 42
conditional_abs(-42) = 42

--- noexcept Move Semantics ---
Original mover created, value = 100
noexcept_mover_move: transferred ownership
Mover moved (noexcept), new value = 100

--- Summary ---
1. noexcept declares function won't throw
2. Move constructors and move assignment operators often use noexcept
3. noexcept move operations have better performance in STL containers
4. noexcept functions cannot call potentially throwing functions
5. In FFI, noexcept is part of function signature
```

## 总结

1. `noexcept` 声明函数不抛出异常
2. 移动构造函数和移动赋值运算符常用 `noexcept`
3. `noexcept` 移动操作在 STL 容器中有更好的性能
4. `noexcept` 函数不能调用可能抛出异常的函数
5. FFI 中 `noexcept` 是函数签名的一部分
6. C++17 引入了 `noexcept(expr)` 运算符可在运行时检查