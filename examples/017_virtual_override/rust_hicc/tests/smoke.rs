//! 017_virtual_override 冒烟测试：显式 override 覆写。

use virtual_override::*;

#[test]
fn base_area_is_zero() {
    let b = Base::new();
    assert_eq!(b.area(), 0.0);
}

#[test]
fn derived_overrides_area() {
    let d = Derived::new(6.0);
    assert_eq!(d.value(), 6.0);
    assert_eq!(d.area(), 36.0);
}
