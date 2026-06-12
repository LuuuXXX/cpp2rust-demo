//! 019_operator_overload 冒烟测试
//!
//! 验证生成的 Rust FFI 绑定可编译、可链接 C++ 实现，且基本行为正确。

use operator_overload::*;
use hicc::AbiClass;

#[test]
fn smoke_number_create() {
    let n = number_new(42);
    assert_eq!(n.get_value(), 42, "number_new(42) 的值应为 42");
}

#[test]
fn smoke_number_add() {
    let a = number_new(10);
    let b = number_new(3);
    let sum = number_add(&a.as_ptr(), &b.as_ptr());
    let result = number_get_value(&sum.as_ptr());
    assert_eq!(result, 13, "10 + 3 应为 13");
}

#[test]
fn smoke_number_sub() {
    let a = number_new(10);
    let b = number_new(3);
    let diff = number_sub(&a.as_ptr(), &b.as_ptr());
    let result = number_get_value(&diff.as_ptr());
    assert_eq!(result, 7, "10 - 3 应为 7");
}

#[test]
fn smoke_number_mul() {
    let a = number_new(10);
    let b = number_new(3);
    let prod = number_mul(&a.as_ptr(), &b.as_ptr());
    let result = number_get_value(&prod.as_ptr());
    assert_eq!(result, 30, "10 * 3 应为 30");
}

#[test]
fn smoke_number_div() {
    let a = number_new(10);
    let b = number_new(3);
    let quot = number_div(&a.as_ptr(), &b.as_ptr());
    let result = number_get_value(&quot.as_ptr());
    assert_eq!(result, 3, "10 / 3 (整数除法) 应为 3");
}

#[test]
fn smoke_number_negate() {
    let a = number_new(10);
    let neg = number_negate(&a.as_ptr());
    let result = number_get_value(&neg.as_ptr());
    assert_eq!(result, -10, "-10 的否定应为 -10");
}

#[test]
fn smoke_number_compare_greater() {
    let a = number_new(10);
    let b = number_new(3);
    let cmp = number_compare(&a.as_ptr(), &b.as_ptr());
    assert!(cmp > 0, "10 与 3 比较应返回正值 (a > b)");
}

#[test]
fn smoke_number_compare_less() {
    let a = number_new(3);
    let b = number_new(10);
    let cmp = number_compare(&a.as_ptr(), &b.as_ptr());
    assert!(cmp < 0, "3 与 10 比较应返回负值 (a < b)");
}

#[test]
fn smoke_number_compare_equal() {
    let a = number_new(5);
    let b = number_new(5);
    let cmp = number_compare(&a.as_ptr(), &b.as_ptr());
    assert_eq!(cmp, 0, "5 与 5 比较应返回 0 (a == b)");
}
