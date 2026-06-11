//! 007_class_constructor 冒烟测试
//!
//! 验证多构造函数（笛卡尔/极坐标）与 const getter 行为正确。

use class_constructor::*;

#[test]
fn smoke_point_xy() {
    let p = point_new_xy(3, 4);
    assert_eq!(p.get_x(), 3, "getX 应返回构造时的 x");
    assert_eq!(p.get_y(), 4, "getY 应返回构造时的 y");
    assert!((p.get_magnitude() - 5.0).abs() < 1e-9, "magnitude 应为 sqrt(3²+4²)=5");
}

#[test]
fn smoke_point_polar() {
    // polar(5, 0) -> x=5, y=0
    let p = point_new_polar(5.0, 0.0);
    assert_eq!(p.get_x(), 5, "polar(5,0) 的 x 应为 5");
    assert_eq!(p.get_y(), 0, "polar(5,0) 的 y 应为 0");
}
