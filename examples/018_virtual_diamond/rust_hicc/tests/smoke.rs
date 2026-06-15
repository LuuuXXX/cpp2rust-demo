//! 018_virtual_diamond 冒烟测试：菱形虚继承的数据汇聚。

use virtual_diamond::*;

#[test]
fn base_and_middle_classes() {
    let a = A::new(1);
    let b = B::new(1, 2);
    let c = C::new(1, 3);
    assert_eq!(a.a_value(), 1);
    assert_eq!(b.b_value(), 2);
    assert_eq!(c.c_value(), 3);
}

#[test]
fn diamond_compute_sums_unique_a() {
    // D 汇聚 B、C，A 子对象唯一：compute = a(1) + b(2) + c(3) + d(4) = 10
    let d = D::new(1, 2, 3, 4);
    assert_eq!(d.d_value(), 4);
    assert_eq!(d.compute(), 10);
}
