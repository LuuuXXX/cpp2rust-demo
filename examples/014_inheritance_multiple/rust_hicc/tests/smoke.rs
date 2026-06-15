//! 014_inheritance_multiple 冒烟测试：多继承基类/派生类绑定与数据组合。

use inheritance_multiple::*;

#[test]
fn base_classes_independent() {
    let b1 = Base1::new(7);
    let b2 = Base2::new(9);
    assert_eq!(b1.value1(), 7);
    assert_eq!(b2.value2(), 9);
}

#[test]
fn derived_combines_two_bases() {
    let d = Derived::new(10, 20, 12);
    assert_eq!(d.derived_value(), 12);
    // compute() 复用两个基类数据成员：10 + 20 + 12
    assert_eq!(d.compute(), 42);
}
