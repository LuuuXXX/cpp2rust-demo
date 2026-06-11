//! 015_virtual_basic 冒烟测试
//!
//! 验证生成的 Rust FFI 绑定可编译、可链接 C++ 实现，且基本行为正确。

use virtual_basic::*;

#[test]
fn smoke_circle_radius() {
    let circle = circle_new(5.0);
    assert!((circle.get_radius() - 5.0).abs() < 1e-10, "Circle::getRadius() 应返回构造时传入的半径");
}

#[test]
fn smoke_circle_area() {
    let circle = circle_new(5.0);
    let expected = std::f64::consts::PI * 25.0;
    assert!((circle.area() - expected).abs() < 1e-6, "Circle::area() 应等于 π * r²");
}

#[test]
fn smoke_circle_get_name() {
    let circle = circle_new(3.0);
    let name = unsafe { std::ffi::CStr::from_ptr(circle.get_name()) };
    let s = name.to_str().unwrap();
    assert!(!s.is_empty(), "Circle::getName() 不应返回空字符串");
}

#[test]
fn smoke_shape_new() {
    // 测试 shape_new() 返回 Shape 基类实例可以正常调用虚函数
    let shape = shape_new();
    let _ = shape.area();
    let name = unsafe { std::ffi::CStr::from_ptr(shape.get_name()) };
    assert!(!name.to_bytes().is_empty(), "Shape::getName() 不应返回空字符串");
}
