//! 021_explicit_ctor 冒烟测试
//!
//! explicit 不影响 FFI（FFI 中所有构造都是显式调用）；验证不同构造入口的值。

use explicit_ctor::*;

#[test]
fn smoke_widget_from_int() {
    let w = widget_new(42);
    assert_eq!(w.get_value(), 42, "隐式构造入口应保留值 42");
    let w2 = widget_from_int(100);
    assert_eq!(w2.get_value(), 100, "explicit int 构造入口应保留值 100");
}

#[test]
fn smoke_widget_from_double_truncates() {
    // Widget(double) 将 3.14 截断为 int 3。
    let w = widget_from_double(3.14);
    assert_eq!(w.get_value(), 3, "double 构造应将 3.14 截断为 3");
}
