//! 020_friend_function 冒烟测试
//!
//! 验证生成的 Rust FFI 绑定可编译、可链接 C++ 实现，且基本行为正确。

use friend_function::*;
use hicc::AbiClass;

#[test]
fn smoke_myclass_create() {
    let obj = myclass_new(42);
    assert_eq!(obj.get_value(), 42, "myclass_new(42) 的值应为 42");
}

#[test]
fn smoke_myclass_set_value() {
    let mut obj = myclass_new(10);
    assert_eq!(obj.get_value(), 10, "初始值应为 10");
    obj.set_value(99);
    assert_eq!(obj.get_value(), 99, "set_value(99) 后值应为 99");
}

#[test]
fn smoke_friend_get_sum() {
    let a = myclass_new(10);
    let b = myclass_new(20);
    let sum = friend_function_get_sum(&a.as_ptr(), &b.as_ptr());
    assert_eq!(sum, 30, "10 + 20 的友元求和应为 30");
}

#[test]
fn smoke_friend_get_product() {
    let a = myclass_new(10);
    let b = myclass_new(20);
    let product = friend_function_get_product(&a.as_ptr(), &b.as_ptr());
    assert_eq!(product, 200, "10 * 20 的友元求积应为 200");
}

#[test]
fn smoke_friend_compare_less() {
    let a = myclass_new(10);
    let b = myclass_new(20);
    let cmp = friend_function_compare(&a.as_ptr(), &b.as_ptr());
    assert_eq!(cmp, -1, "10 < 20 友元比较应返回 -1");
}

#[test]
fn smoke_friend_compare_greater() {
    let a = myclass_new(20);
    let b = myclass_new(10);
    let cmp = friend_function_compare(&a.as_ptr(), &b.as_ptr());
    assert_eq!(cmp, 1, "20 > 10 友元比较应返回 1");
}

#[test]
fn smoke_friend_compare_equal() {
    let a = myclass_new(15);
    let b = myclass_new(15);
    let cmp = friend_function_compare(&a.as_ptr(), &b.as_ptr());
    assert_eq!(cmp, 0, "15 == 15 友元比较应返回 0");
}
