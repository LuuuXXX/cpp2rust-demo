//! 039_lambda_basic 冒烟测试
//!
//! 验证生成的 Rust FFI 绑定可编译、可链接 C++ 实现，且 lambda/闭包行为正确。

use lambda_basic::*;

#[test]
fn smoke_operation_add() {
    let op = Operation::new(0);
    assert_eq!(op.apply(3, 4), 7);
}

#[test]
fn smoke_operation_multiply() {
    let op = Operation::new(1);
    assert_eq!(op.apply(3, 4), 12);
}

#[test]
fn smoke_operation_max() {
    let op = Operation::new(2);
    assert_eq!(op.apply(3, 4), 4);
    assert_eq!(op.apply(9, 4), 9);
}

#[test]
fn smoke_accumulator_captures_state() {
    let mut acc = Accumulator::new(10);
    assert_eq!(acc.apply(5), 15, "闭包应把 delta 累加进捕获的状态");
    assert_eq!(acc.apply(3), 18);
    assert_eq!(acc.value(), 18);
}
