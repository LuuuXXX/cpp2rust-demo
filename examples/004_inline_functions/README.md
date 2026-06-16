# 004_inline_functions - 命名空间自由函数

## C++ 特性

命名空间内的一组自由函数。被 FFI 绑定的函数需在实现单元（`.cpp`）内定义，
故此处统一在命名空间头文件声明、`.cpp` 定义，由 hicc `import_lib!` 直出绑定。

## C++ 代码

### inline_functions.h

```cpp
#pragma once

namespace inline_functions_ns {

int min(int a, int b);
int max(int a, int b);
int min_v2(int a, int b);
int max_v2(int a, int b);

} // namespace inline_functions_ns
```

## Rust FFI 代码

```rust
hicc::import_lib! {
    #![link_name = "inline_functions"]

    #[cpp(func = "int inline_functions_ns::min(int, int)")]
    pub fn min(a: i32, b: i32) -> i32;

    #[cpp(func = "int inline_functions_ns::max(int, int)")]
    pub fn max(a: i32, b: i32) -> i32;

    #[cpp(func = "int inline_functions_ns::min_v2(int, int)")]
    pub fn min_v2(a: i32, b: i32) -> i32;

    #[cpp(func = "int inline_functions_ns::max_v2(int, int)")]
    pub fn max_v2(a: i32, b: i32) -> i32;
}
```

## 构建方法

```bash
cd cpp && ./standalone.sh    # 独立 C++ 验证
cd rust_hicc && cargo run    # Rust 端运行
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

1. 内联是编译时特性，FFI 边界需要可链接符号，故被绑定函数定义在 `.cpp`
2. 命名空间函数无需 `extern "C"`，以 `ns::fn()` 直出绑定
3. Rust 端调用方式与是否内联无关
