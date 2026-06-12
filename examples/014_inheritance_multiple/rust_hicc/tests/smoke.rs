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
    assert_eq!(derived.get_value2(), 20, "Base2 值应为 20");
}

#[test]
fn smoke_derived_compute() {
    let derived = derived_new(5, 10, 15);
    // compute() 仅输出到 stdout（打印 5+10+15=30），验证调用不 panic
    derived.compute();
}
