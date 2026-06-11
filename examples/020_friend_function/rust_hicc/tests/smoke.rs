//! 020_friend_function 冒烟测试
//!
//! 友元函数在 FFI 中即普通函数；验证其访问对象数据后的计算结果。

use friend_function::*;
use hicc::AbiClass;

#[test]
fn smoke_friend_sum_product() {
    let a = myclass_new(10);
    let b = myclass_new(20);
    assert_eq!(a.get_value(), 10, "MyClass a 值应为 10");
    assert_eq!(b.get_value(), 20, "MyClass b 值应为 20");

    assert_eq!(friend_function_get_sum(&a.as_ptr(), &b.as_ptr()), 30, "友元求和应为 30");
    assert_eq!(friend_function_get_product(&a.as_ptr(), &b.as_ptr()), 200, "友元乘积应为 200");
}

#[test]
fn smoke_friend_compare() {
    let a = myclass_new(10);
    let b = myclass_new(20);
    assert_eq!(friend_function_compare(&a.as_ptr(), &b.as_ptr()), -1, "a < b 应返回 -1");
    assert_eq!(friend_function_compare(&b.as_ptr(), &a.as_ptr()), 1, "b > a 应返回 1");
}
