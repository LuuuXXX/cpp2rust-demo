//! 007_class_constructor 冒烟测试
//!
//! 验证生成的 Rust FFI 绑定可编译、可链接 C++ 实现，且基本行为正确。

use class_constructor::*;

#[test]
fn smoke_point_xy_constructor() {
    let p = point_new_xy(3, 4);
    assert_eq!(p.get_x(), 3, "x 坐标应为 3");
    assert_eq!(p.get_y(), 4, "y 坐标应为 4");
}

#[test]
fn smoke_point_magnitude() {
    let p = point_new_xy(3, 4);
    let mag = p.get_magnitude();
    assert!((mag - 5.0f64).abs() < 1e-6, "3-4-5 三角形幅度应为 5.0");
}

#[test]
fn smoke_point_polar_constructor() {
    let p = point_new_polar(5.0, 0.0);
    assert_eq!(p.get_x(), 5, "极坐标 (5, 0) 的 x 应为 5");
    assert_eq!(p.get_y(), 0, "极坐标 (5, 0) 的 y 应为 0");
}
