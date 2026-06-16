//! 015_virtual_basic 冒烟测试：虚函数覆写。

use virtual_basic::*;

#[test]
fn base_area_is_zero() {
    let s = Shape::new();
    assert_eq!(s.area(), 0.0);
}

#[test]
fn derived_overrides_area() {
    let c = Circle::new(2.0);
    assert_eq!(c.radius(), 2.0);
    let expect = std::f64::consts::PI * 4.0;
    assert!((c.area() - expect).abs() < 1e-9, "area={}", c.area());
}
