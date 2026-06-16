//! 040_std_function 冒烟测试
//!
//! 验证生成的 Rust FFI 绑定可编译、可链接 C++ 实现，且 std::function 行为正确。

use std_function::*;

#[test]
fn smoke_callback_double() {
    let cb = Callback::new(0);
    assert_eq!(cb.invoke(5), 10);
    assert_eq!(cb.invoke(-3), -6);
}

#[test]
fn smoke_callback_triple() {
    let cb = Callback::new(1);
    assert_eq!(cb.invoke(5), 15);
    assert_eq!(cb.invoke(-3), -9);
}

#[test]
fn smoke_callback_negate() {
    let cb = Callback::new(2);
    assert_eq!(cb.invoke(5), -5);
    assert_eq!(cb.invoke(-3), 3);
}

#[test]
fn smoke_pipeline_add_and_run() {
    let mut p = Pipeline::new();
    p.add(0);
    p.add(1);
    assert_eq!(p.run(2), 12, "应按顺序执行 double 再 triple");
}

#[test]
fn smoke_pipeline_size_and_state() {
    let mut p = Pipeline::new();
    assert_eq!(p.size(), 0);
    p.add(2);
    assert_eq!(p.size(), 1);
    assert_eq!(p.run(5), -5);
}
