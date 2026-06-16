//! 028_variadic_template 冒烟测试
//!
//! 验证生成的 Rust FFI 绑定可编译、可链接 C++ 模板实例化，且基本行为正确。

use variadic_template::*;

#[test]
fn smoke_sum_i32() {
    assert_eq!(sum_i32_0(), 0);
    assert_eq!(sum_i32_2(10, 20), 30);
    assert_eq!(sum_i32_3(1, 2, 3), 6);
    assert_eq!(sum_i32_5(1, 2, 3, 4, 5), 15);
}

#[test]
fn smoke_sum_f64() {
    assert!((sum_f64_2(1.5, 2.5) - 4.0).abs() < 1e-10);
    assert!((sum_f64_3(1.5, 2.5, 3.0) - 7.0).abs() < 1e-10);
}

#[test]
fn smoke_anchor() {
    assert_eq!(variadic_template_anchor(), 0);
}
