//! 028_variadic_template 冒烟测试
//!
//! 验证生成的 Rust FFI 绑定可编译、可链接 C++ 实现，且基本行为正确。

use variadic_template::*;

#[test]
fn smoke_sum_zero() {
    assert_eq!(sum_zero(), 0, "sum() 无参数应返回 0");
}

#[test]
fn smoke_sum_1() {
    assert_eq!(sum_1(42), 42, "sum_1(42) 应返回 42");
}

#[test]
fn smoke_sum_2() {
    assert_eq!(sum_2(1, 2), 3, "sum_2(1, 2) 应返回 3");
}

#[test]
fn smoke_sum_3() {
    assert_eq!(sum_3(1, 2, 3), 6, "sum_3(1, 2, 3) 应返回 6");
    assert_eq!(sum_3(-1, -2, -3), -6, "sum_3(-1, -2, -3) 应返回 -6");
}

#[test]
fn smoke_sum_4() {
    assert_eq!(sum_4(1, 2, 3, 4), 10, "sum_4(1, 2, 3, 4) 应返回 10");
}

#[test]
fn smoke_sum_5() {
    assert_eq!(sum_5(1, 2, 3, 4, 5), 15, "sum_5(1..5) 应返回 15");
    assert_eq!(sum_5(10, 20, 30, 40, 50), 150, "sum_5(10..50) 应返回 150");
}

#[test]
fn smoke_sum_double_2() {
    let result = sum_double_2(1.5, 2.5);
    assert!((result - 4.0).abs() < 1e-10, "sum_double_2(1.5, 2.5) 应返回 4.0");
}

#[test]
fn smoke_sum_double_3() {
    let result = sum_double_3(1.1, 2.2, 3.3);
    assert!((result - 6.6).abs() < 1e-10, "sum_double_3(1.1, 2.2, 3.3) 应返回 6.6");
}

#[test]
fn smoke_sum_double_4() {
    let result = sum_double_4(1.0, 2.0, 3.0, 4.0);
    assert!((result - 10.0).abs() < 1e-10, "sum_double_4(1, 2, 3, 4) 应返回 10.0");
}

#[test]
fn smoke_sum_get_format() {
    use std::ffi::CStr;
    let fmt_ptr = unsafe { sum_get_format(3) };
    let fmt = unsafe { CStr::from_ptr(fmt_ptr) }.to_str().expect("应为有效 UTF-8");
    assert!(!fmt.is_empty(), "格式字符串不应为空");
}
