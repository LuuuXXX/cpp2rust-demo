//! 039_lambda_basic 冒烟测试
//!
//! 验证生成的 Rust FFI 绑定可编译、可链接 C++ 实现，且基本行为正确。

use lambda_basic::*;

#[test]
fn smoke_add_impl() {
    let result = add_impl(3, 4);
    assert_eq!(result, 7, "add_impl(3, 4) 应返回 7");
}

#[test]
fn smoke_multiply_impl() {
    let result = multiply_impl(3, 4);
    assert_eq!(result, 12, "multiply_impl(3, 4) 应返回 12");
}

#[test]
fn smoke_max_impl() {
    let result = max_impl(3, 4);
    assert_eq!(result, 4, "max_impl(3, 4) 应返回 4");
}

#[test]
fn smoke_lambda_wrapper_add() {
    let mut wrapper = make_add_lambda();
    let result = wrapper.invoke(5, 6);
    assert_eq!(result, 11, "add lambda invoke(5, 6) 应返回 11");
}

#[test]
fn smoke_lambda_wrapper_multiply() {
    let mut wrapper = make_multiply_lambda();
    let result = wrapper.invoke(5, 6);
    assert_eq!(result, 30, "multiply lambda invoke(5, 6) 应返回 30");
}

#[test]
fn smoke_lambda_wrapper_max() {
    let mut wrapper = make_max_lambda();
    let result = wrapper.invoke(3, 7);
    assert_eq!(result, 7, "max lambda invoke(3, 7) 应返回 7");
}

#[test]
fn smoke_state_lambda() {
    let mut state = state_lambda_new(10);
    assert_eq!(state.get_value(), 10, "初始值应为 10");
    let result = state.add(5);
    assert_eq!(result, 15, "add(5) 应返回 15");
    assert_eq!(state.get_value(), 15, "add(5) 后值应为 15");
    let result2 = state.add(3);
    assert_eq!(result2, 18, "add(3) 应返回 18");
    assert_eq!(state.get_value(), 18, "add(3) 后值应为 18");
}

#[test]
fn smoke_comparator_new_add() {
    let cmp = comparator_new_add();
    let result = cmp.compare(2, 3);
    assert_eq!(result, 5, "comparator_new_add compare(2, 3) 应返回 5（add_impl）");
}
