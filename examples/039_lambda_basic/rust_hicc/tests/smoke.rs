//! 039_lambda_basic 冒烟测试
//!
//! Lambda 经 class wrapper / 函数指针映射；验证直接函数、wrapper 与有状态 lambda。

use lambda_basic::*;

#[test]
fn smoke_direct_impls() {
    assert_eq!(add_impl(3, 4), 7, "add_impl 应为求和");
    assert_eq!(multiply_impl(3, 4), 12, "multiply_impl 应为乘积");
    assert_eq!(max_impl(3, 4), 4, "max_impl 应为较大值");
}

#[test]
fn smoke_lambda_wrapper_invoke() {
    let mut add_wrapper = make_add_lambda();
    assert_eq!(add_wrapper.invoke(5, 6), 11, "add wrapper invoke 应为 11");
    let mut mul_wrapper = make_multiply_lambda();
    assert_eq!(mul_wrapper.invoke(5, 6), 30, "multiply wrapper invoke 应为 30");
}

#[test]
fn smoke_state_lambda_accumulates() {
    let mut state = state_lambda_new(10);
    assert_eq!(state.get_value(), 10, "初始值应为 10");
    assert_eq!(state.add(5), 15, "add(5) 后应为 15");
    assert_eq!(state.add(3), 18, "再 add(3) 后应为 18");
}

#[test]
fn smoke_comparator_uses_add() {
    // comparator_new_add 以 add_impl 作为比较器。
    let mut cmp = comparator_new_add();
    assert_eq!(cmp.compare(2, 3), 5, "compare(2,3) 应等于 add(2,3)=5");
}
