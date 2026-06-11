//! 014_inheritance_multiple 冒烟测试
//!
//! 验证多继承下两个基类的方法均提升进派生类。

use inheritance_multiple::*;

#[test]
fn smoke_derived_values() {
    let d = unsafe { derived_new(10, 20, 30) };
    assert_eq!(d.get_value1(), 10, "Base1::getValue1 应为 10");
    assert_eq!(d.get_value2(), 20, "Base2::getValue2 应为 20");
    assert_eq!(d.get_derived_value(), 30, "Derived::getDerivedValue 应为 30");
    // compute 仅打印，验证可调用不 panic。
    d.compute();
}
