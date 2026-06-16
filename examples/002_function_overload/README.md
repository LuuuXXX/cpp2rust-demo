# 002_function_overload - 命名空间内的重载族

## C++ 特性

命名空间内的一组自由函数（不同参数类型/个数），无需 `extern "C"`，由 hicc
`import_lib!` 以 `ns::fn()` 形式直出绑定。

## C++ 代码

### function_overload.h

```cpp
#pragma once

namespace function_overload_ns {

int add_int(int a, int b);
double add_double(double a, double b);
const char* add_strings(const char* a, const char* b);
int sum3(int a, int b, int c);

} // namespace function_overload_ns
```

实现见 `function_overload.cpp`（同一命名空间内定义）。

## Rust FFI 代码

```rust
hicc::import_lib! {
    #![link_name = "function_overload"]

    #[cpp(func = "int function_overload_ns::add_int(int, int)")]
    pub fn add_int(a: i32, b: i32) -> i32;

    #[cpp(func = "double function_overload_ns::add_double(double, double)")]
    pub fn add_double(a: f64, b: f64) -> f64;

    #[cpp(func = "const char* function_overload_ns::add_strings(const char*, const char*)")]
    pub unsafe fn add_strings(a: *const i8, b: *const i8) -> *const i8;

    #[cpp(func = "int function_overload_ns::sum3(int, int, int)")]
    pub fn sum3(a: i32, b: i32, c: i32) -> i32;
}
```

## 构建方法

```bash
cd cpp && ./standalone.sh    # 独立 C++ 验证
cd rust_hicc && cargo run    # Rust 端运行
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

## 注意事项

1. 命名空间函数以 `#[cpp(func = "ns::fn()")]` 直出绑定，无需 `extern "C"`
2. 字符串参数/返回为 `const char*`，对应 Rust `*const i8`，调用需 `unsafe`
3. 被绑定的函数需在实现单元（`.cpp`）内定义
