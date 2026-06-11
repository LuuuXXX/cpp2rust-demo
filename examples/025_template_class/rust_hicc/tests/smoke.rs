//! 025_template_class 冒烟测试
//!
//! 验证生成的 Rust FFI 绑定（IntStack / DoubleStack）可编译、链接并正确调用。

use template_class::*;

#[test]
fn smoke_int_stack_basic() {
    let mut s = intstack_new();
    assert!(s.empty(), "新建栈应为空");
    assert_eq!(s.size(), 0);

    s.push(10);
    s.push(20);
    s.push(30);

    assert!(!s.empty());
    assert_eq!(s.size(), 3);
    assert_eq!(s.top(), 30);

    s.pop();
    assert_eq!(s.size(), 2);
    assert_eq!(s.top(), 20);
}

#[test]
fn smoke_double_stack_basic() {
    let mut s = doublestack_new();
    assert!(s.empty(), "新建栈应为空");

    s.push(1.1);
    s.push(2.2);

    assert_eq!(s.size(), 2);
    assert!((s.top() - 2.2).abs() < 1e-10);

    s.pop();
    assert_eq!(s.size(), 1);
    assert!((s.top() - 1.1).abs() < 1e-10);
}

#[test]
fn smoke_int_stack_type_available() {
    fn assert_type_available<T>() {}
    assert_type_available::<IntStack>();
    assert_type_available::<DoubleStack>();
}
