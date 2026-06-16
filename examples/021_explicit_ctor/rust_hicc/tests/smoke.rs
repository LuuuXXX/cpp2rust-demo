//! 021_explicit_ctor 冒烟测试：多构造工厂 new(i32) / new_2(f64)。

use explicit_ctor::*;

#[test]
fn ctor_from_int() {
    let a = Widget::new(42);
    assert_eq!(a.get_value(), 42);
}

#[test]
fn ctor_from_double_truncates() {
    let b = Widget::new_2(3.9);
    assert_eq!(b.get_value(), 3);
}
