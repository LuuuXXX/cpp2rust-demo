//! 028_variadic_template: 可变参数模板（命名空间折叠表达式模板 + 显式实例化包装）。
//!
//! C++ 可变参数函数模板 `sum<Args...>`（C++17 折叠表达式）可对任意个数/类型的实参求和，
//! 但每个具体的「实参个数 + 类型」组合是一次独立实例化，无法整体绑定到 Rust。hicc 直出
//! 仅绑定可链接的非模板锚点 `variadic_template_anchor()`（见 `lib_scaffold.rs`）。各具体
//! 组合在手写 `lib.rs` 中以 `hicc::cpp!` 命名包装函数补全：每个包装显式实例化模板（固定
//! 实参个数与类型），再用 `#[cpp(func = ...)]` 绑定为普通自由函数。

hicc::cpp! {
    #include "variadic_template.h"

    using namespace variadic_template_ns;

    // 显式实例化：固定「实参个数 + 类型」即一次模板实例化。
    int sum_i32_0() { return sum(); }
    int sum_i32_2(int a, int b) { return sum(a, b); }
    int sum_i32_3(int a, int b, int c) { return sum(a, b, c); }
    int sum_i32_5(int a, int b, int c, int d, int e) { return sum(a, b, c, d, e); }

    double sum_f64_2(double a, double b) { return sum(a, b); }
    double sum_f64_3(double a, double b, double c) { return sum(a, b, c); }
}

hicc::import_lib! {
    #![link_name = "variadic_template"]

    #[cpp(func = "int sum_i32_0()")]
    pub fn sum_i32_0() -> i32;

    #[cpp(func = "int sum_i32_2(int, int)")]
    pub fn sum_i32_2(a: i32, b: i32) -> i32;

    #[cpp(func = "int sum_i32_3(int, int, int)")]
    pub fn sum_i32_3(a: i32, b: i32, c: i32) -> i32;

    #[cpp(func = "int sum_i32_5(int, int, int, int, int)")]
    pub fn sum_i32_5(a: i32, b: i32, c: i32, d: i32, e: i32) -> i32;

    #[cpp(func = "double sum_f64_2(double, double)")]
    pub fn sum_f64_2(a: f64, b: f64) -> f64;

    #[cpp(func = "double sum_f64_3(double, double, double)")]
    pub fn sum_f64_3(a: f64, b: f64, c: f64) -> f64;

    #[cpp(func = "int variadic_template_anchor()")]
    pub fn variadic_template_anchor() -> i32;
}
