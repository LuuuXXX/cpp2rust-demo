//! 016_virtual_pure 冒烟测试：纯虚接口的具体实现。

use virtual_pure::*;

#[test]
fn circle_implements_area() {
    let c = Circle::new(2.0);
    assert_eq!(c.radius(), 2.0);
    let expect = std::f64::consts::PI * 4.0;
    assert!((c.area() - expect).abs() < 1e-9, "area={}", c.area());
}

#[test]
fn rectangle_implements_area() {
    let r = Rectangle::new(3.0, 4.0);
    assert_eq!(r.area(), 12.0);
}
