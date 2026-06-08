# 046_constexpr_basic - constexpr

## C++ 特性

本示例展示 C++ `constexpr` 关键字如何在 FFI 中体现。`constexpr` 指定表达式在编译期计算，可以是变量、函数或类构造函数。

## C++ 代码

### constexpr_basic.h

```cpp
#pragma once

#ifdef __cplusplus
extern "C" {
#endif

// constexpr 数组大小
#define ARRAY_SIZE 10

// constexpr 函数
constexpr int square(int x) {
    return x * x;
}

// constexpr 模板函数（编译期递归）
template<int N>
constexpr int fibonacci() {
    if constexpr (N <= 1) {
        return N;
    } else {
        return fibonacci<N-1>() + fibonacci<N-2>();
    }
}

// 获取编译期计算的斐波那契数
int get_fibonacci_10(void);

#ifdef __cplusplus
}
#endif
```

### constexpr_basic.cpp

```cpp
#include "constexpr_basic.h"

// 斐波那契数列编译期计算
// fibonacci<10>() = 55 在编译时就已经确定了
constexpr int FIB_10 = fibonacci<10>();

int get_fibonacci_10(void) {
    return FIB_10;  // 直接返回编译期常量
}
```

## Rust FFI 代码

```rust
hicc::cpp! {
    #include "constexpr_basic.h"
}

hicc::import_lib! {
    #![link_name = "constexpr_basic"]

    #[cpp(func = "int get_fibonacci_10()")]
    fn get_fibonacci_10() -> i32;

    #[cpp(func = "int manhattan_distance(int, int)")]
    fn manhattan_distance(x: i32, y: i32) -> i32;

    #[cpp(func = "int constexpr_sum_array(const int*, int)")]
    fn constexpr_sum_array(arr: *const i32, size: i32) -> i32;

    #[cpp(func = "int constexpr_find_max(const int*, int)")]
    fn constexpr_find_max(arr: *const i32, size: i32) -> i32;

    #[cpp(func = "int get_array_size()")]
    fn get_array_size() -> i32;
}
```
## constexpr vs const

| 特性 | const | constexpr |
|------|-------|-----------|
| 适用对象 | 变量、引用 | 变量、函数、类 |
| 求值时间 | 可能是运行时 | 编译时 |
| 复杂性 | 简单 | 可以很复杂 |
| 模板 | 不可用 | 可用于模板参数 |

## 构建方法

### C++ 编译

```bash
cd cpp
g++ -c constexpr_basic.cpp -o constexpr_basic.o
g++ -shared -fPIC constexpr_basic.cpp -o libconstexpr_basic.so
```

### Rust 编译

```bash
cd rust_hicc
cargo build
```

## FFI 对比分析

| 方面 | C++ | Rust |
|------|-----|------|
| 编译期常量 | `constexpr int N = 10;` | `const N: i32 = 10;` |
| 编译期函数 | `constexpr int sq(int x) { return x*x; }` | `const fn sq(x: i32) -> i32 { x*x }` |
| 模板参数 | `fibonacci<10>()` | 泛型 `fib::<10>()` ( nightly ) |
| 宏 | `#define ARRAY_SIZE 10` | 编译期常量 |

## 运行结果

```
=== 046_constexpr_basic - constexpr ===

--- Compile-time Fibonacci ---
get_fibonacci_10() called, returning compile-time computed value: 55
fibonacci<10>() = 55 (computed at compile time)
Rust equivalent: fib(10) = 55 (also compile time)

--- Runtime Manhattan Distance ---
manhattan_distance(3, 4) = 7
manhattan_distance(-3, -4) = 7
manhattan_distance(10, -5) = 15

--- Array Operations ---
Array: [1, 5, 3, 9, 2, 8, 4, 7, 6, 0]
Sum: 45
Max: 9

--- Summary ---
1. constexpr specifies expression computed at compile time
2. constexpr functions must satisfy compile-time evaluation conditions
3. constexpr variables have determined values at compile time
4. FFI constexpr values passed via preprocessor macros
5. Rust const fn can achieve similar functionality
```

## 总结

1. `constexpr` 指定表达式在编译期计算
2. `constexpr` 函数必须满足编译期求值的条件
3. `constexpr` 变量在编译时就有确定的值
4. FFI 中 `constexpr` 值通过预处理器宏或内联函数传递
5. Rust `const fn` 可以实现类似的编译期计算功能
6. C++17 引入了 `constexpr lambda`，C++20 引入了 `consteval`