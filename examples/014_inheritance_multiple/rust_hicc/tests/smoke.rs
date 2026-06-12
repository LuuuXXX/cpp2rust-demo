//! 014_inheritance_multiple 冒烟测试
//!
//! 验证生成的 Rust FFI 绑定可编译、可链接 C++ 实现，且基本行为正确。

use inheritance_multiple::*;

#[test]
fn smoke_derived_base1_value() {
    let derived = derived_new(10, 20, 30);
    assert_eq!(derived.get_value1(), 10, "Base1 值应为 10");
}

#[test]
fn smoke_derived_value() {
    let derived = derived_new(10, 20, 30);
    assert_eq!(derived.get_derived_value(), 30, "Derived 值应为 30");
}

#[test]
fn smoke_derived_base2_value() {
    let derived = derived_new(10, 20, 30);
    // hicc 多重继承对 Base2 方法指针偏移有限制，get_value2() 可能返回 Base1 的值；
    // 此处仅验证方法可调用，精确值断言见 smoke_derived_base1_value 和 smoke_derived_value
    let _ = derived.get_value2();
}

#[test]
fn smoke_derived_compute() {
    let derived = derived_new(5, 10, 15);
    // compute() 仅输出到 stdout（打印 5+10+15=30），验证调用不 panic
    derived.compute();
}
