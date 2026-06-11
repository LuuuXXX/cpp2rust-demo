//! 023_typeid_rtti 冒烟测试
//!
//! 验证生成的 Rust FFI 绑定可编译、可链接 C++ 实现，且基本行为正确。

use hicc::AbiClass;
use typeid_rtti::*;

#[test]
fn smoke_circle_type_and_area() {
    let circle = unsafe { shape_new_circle(5.0).into_unique() };
    assert_eq!(circle.get_type(), 0, "Circle 的 getType() 应返回 SHAPE_TYPE_CIRCLE=0");
    let area = circle.area();
    // area ≈ π * 5² ≈ 78.539...，用 3.14159 近似
    assert!(area > 78.0 && area < 79.0, "Circle 面积应约为 78.5");
}

#[test]
fn smoke_rectangle_type_and_area() {
    let rect = unsafe { shape_new_rectangle(4.0, 6.0).into_unique() };
    assert_eq!(rect.get_type(), 1, "Rectangle 的 getType() 应返回 SHAPE_TYPE_RECTANGLE=1");
    assert!((rect.area() - 24.0).abs() < 1e-10, "Rectangle 面积应为 24.0");
}

#[test]
fn smoke_triangle_type_and_area() {
    let tri = unsafe { shape_new_triangle(3.0, 4.0).into_unique() };
    assert_eq!(tri.get_type(), 2, "Triangle 的 getType() 应返回 SHAPE_TYPE_TRIANGLE=2");
    assert!((tri.area() - 6.0).abs() < 1e-10, "Triangle 面积应为 0.5 * 3 * 4 = 6.0");
}

#[test]
fn smoke_get_type_name() {
    let circle = unsafe { shape_new_circle(1.0).into_unique() };
    let name = unsafe { std::ffi::CStr::from_ptr(circle.get_type_name()) };
    assert_eq!(name.to_str().unwrap(), "Circle", "getTypeName() 应返回 \"Circle\"");
}
