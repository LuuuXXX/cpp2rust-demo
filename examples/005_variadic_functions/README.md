# 005_variadic_functions - 可变参数与固定参数包装

## C++ 特性

C 可变参数函数（`int sum(int, ...)`）无法直接经 FFI 调用，工具会跳过其绑定；
通过命名空间内的固定参数包装函数（`sum_3`/`sum_5`）暴露给 Rust，由 hicc
`import_lib!` 直出绑定。

## C++ 代码

### variadic_functions.h

```cpp
#pragma once

namespace variadic_functions_ns {

// 真正的可变参数函数（工具跳过绑定）
int sum(int count, ...);
int print_formatted(const char* format, ...);

// FFI 固定参数包装函数（直出绑定）
int sum_3(int a, int b, int c);
int sum_5(int a, int b, int c, int d, int e);

} // namespace variadic_functions_ns
```

## Rust FFI 代码

```rust
hicc::import_lib! {
    #![link_name = "variadic_functions"]

    #[cpp(func = "int variadic_functions_ns::sum_3(int, int, int)")]
    pub fn sum_3(a: i32, b: i32, c: i32) -> i32;

    #[cpp(func = "int variadic_functions_ns::sum_5(int, int, int, int, int)")]
    pub fn sum_5(a: i32, b: i32, c: i32, d: i32, e: i32) -> i32;
}
```

## 构建方法

```bash
cd cpp && ./standalone.sh    # 独立 C++ 验证
cd rust_hicc && cargo run    # Rust 端运行
```

## 运行结果

```
=== 005_variadic_functions - 可变参数函数 ===

--- sum (via wrapper) ---
sum_3(1, 2, 3) = 6
sum_5(1, 2, 3, 4, 5) = 15

--- print_formatted ---
Hello from variadic_functions!

--- 总结 ---
1. C 可变参数函数无法直接通过 FFI 调用
2. 需要为每种参数组合提供固定参数包装函数
3. Rust 调用这些固定参数包装函数
```

## 总结

1. 可变参数是 C 风格特性，FFI 边界类型不安全，工具默认跳过其绑定
2. 用命名空间内的固定参数包装函数暴露给 Rust，以 `ns::fn()` 直出绑定
3. 尽量避免可变参数 FFI，使用固定参数或数组代替
