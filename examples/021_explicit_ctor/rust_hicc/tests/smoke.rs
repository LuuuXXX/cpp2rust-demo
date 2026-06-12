//! 021_explicit_ctor 冒烟测试
//!
//! 验证生成的 Rust FFI 绑定可编译、可链接 C++ 实现，且基本行为正确。

use explicit_ctor::*;

#[test]
fn smoke_widget_new_int() {
    let w = widget_new(42);
    assert_eq!(w.get_value(), 42, "widget_new(42) 的值应为 42");
}

#[test]
fn smoke_widget_from_int() {
    let w = widget_from_int(100);
    assert_eq!(w.get_value(), 100, "widget_from_int(100) 的值应为 100");
}

#[test]
fn smoke_widget_from_double() {
    let w = widget_from_double(3.14);
    // C++ truncates double to int: static_cast<int>(3.14) = 3
    assert_eq!(w.get_value(), 3, "widget_from_double(3.14) 的值应为 3 (截断为整数)");
}

#[test]
fn smoke_widget_from_double_zero() {
    let w = widget_from_double(0.99);
    assert_eq!(w.get_value(), 0, "widget_from_double(0.99) 的值应为 0 (截断为整数)");
}

#[test]
fn smoke_widget_from_double_exact() {
    let w = widget_from_double(7.0);
    assert_eq!(w.get_value(), 7, "widget_from_double(7.0) 的值应为 7");
}
