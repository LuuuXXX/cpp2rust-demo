# 004_inline_functions - 内联函数

## C++ 特性

本示例展示 C++ 内联函数特性及其在 FFI 边界的行为。

## C++ 代码

### inline_functions.h

```cpp
#pragma once
#ifdef __cplusplus
extern "C" {
#endif

// 内联函数
inline int min(int a, int b) {
    return a < b ? a : b;
}

inline int max(int a, int b) {
    return a > b ? a : b;
}

// 普通函数（用于对比）
int min_v2(int a, int b);
int max_v2(int a, int b);

#ifdef __cplusplus
}
#endif
```

### inline_functions.cpp

```cpp
#include "inline_functions.h"

int min_v2(int a, int b) {
    return a < b ? a : b;
}

int max_v2(int a, int b) {
    return a > b ? a : b;
}
```

## 内联函数与 FFI

### 内联函数特性

1. **编译时展开**：内联函数在调用点直接展开，不生成函数调用
2. **头文件实现**：通常在头文件中实现
3. **链接器不可见**：如果完全内联，链接器看不到函数符号

### FFI 边界处理

在内联函数的情况下：
- **完全内联**：调用点直接展开，不产生外部符号，FFI 无法调用
- **非完全内联**：如果内联失败或显式禁止内联，会产生符号，可被 FFI 调用

在 hicc 中，`cpp!` 宏会尝试编译内联函数，但由于 `extern "C"` 修饰，编译器可能拒绝内联，从而产生可链接的符号。

## Rust FFI 代码

### main.rs

```rust
hicc::import_lib! {
    #![link_name = "inline_functions"]

    #[cpp(func = "int min(int, int)")]
    fn min_val(a: i32, b: i32) -> i32;

    #[cpp(func = "int max(int, int)")]
    fn max_val(a: i32, b: i32) -> i32;

    #[cpp(func = "int min_v2(int, int)")]
    fn min_v2(a: i32, b: i32) -> i32;

    #[cpp(func = "int max_v2(int, int)")]
    fn max_v2(a: i32, b: i32) -> i32;
}
```

## 关键点

### 内联函数的 FFI 限制

| 场景 | 内联成功 | 内联失败/禁止 |
|------|----------|----------------|
| 符号生成 | 无符号 | 有符号 |
| FFI 调用 | 不可调用 | 可调用 |
| 性能 | 零开销调用 | 正常函数调用 |

### Rust 端调用

Rust 调用时，内联和非内联函数**没有区别**：
```rust
let result = min_val(10, 20);  // 看起来像函数调用
                                   // 实际可能内联展开
```

## 构建方法

### C++ 编译

```bash
cd cpp
g++ -c inline_functions.cpp -o inline_functions.o
# 注意：如果 cpp! 宏编译了 inline_functions.h，
# 可能会产生重复符号
```

### Rust 编译

```bash
cd rust_hicc
cargo build
cargo run
```

## 运行结果

```
min(10, 20) = 10
max(10, 20) = 20
min_v2(10, 20) = 10
max_v2(10, 20) = 20

Rust FFI: Inline and normal functions work the same way!
```

## 总结

1. **内联函数是编译时特性**：FFI 边界无法保证内联
2. **符号可能不存在**：完全内联时，链接器看不到符号
3. **hicc 处理方式**：通过 `extern "C"` 修饰，可能使编译器生成可链接符号
4. **Rust 调用无区别**：无论是否内联，Rust 端调用方式相同
