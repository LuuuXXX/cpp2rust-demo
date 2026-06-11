//! 016_virtual_pure 冒烟测试
//!
//! 验证生成的 Rust FFI 绑定可编译、可链接 C++ 实现，且基本行为正确。

use hicc::AbiClass;
use virtual_pure::*;

#[test]
fn smoke_circle_area() {
    let circle = abstract_shape_create_circle(5.0);
    let expected = std::f64::consts::PI * 25.0;
    assert!((circle.area() - expected).abs() < 1e-4, "圆的面积应等于 π * r²");
    unsafe { circle.into_value().into_unique() };
}

#[test]
fn smoke_rectangle_area() {
    let rect = abstract_shape_create_rectangle(4.0, 6.0);
    assert!((rect.area() - 24.0).abs() < 1e-10, "矩形面积应等于 width * height");
    unsafe { rect.into_value().into_unique() };
}

#[test]
fn smoke_circle_get_name() {
    let circle = abstract_shape_create_circle(3.0);
    let name = unsafe { std::ffi::CStr::from_ptr(circle.get_name()) };
    assert!(!name.to_bytes().is_empty(), "getName() 不应返回空字符串");
    unsafe { circle.into_value().into_unique() };
}
