//! 020_friend_function 冒烟测试：友元函数经 hicc::cpp! 命名包装绑定为关联方法。

use friend_function::*;

#[test]
fn create_and_accessors() {
    let mut a = MyClass::new(42);
    assert_eq!(a.get_value(), 42);
    a.set_value(7);
    assert_eq!(a.get_value(), 7);
}

#[test]
fn friend_sum_and_product() {
    let a = MyClass::new(10);
    let b = MyClass::new(20);
    assert_eq!(a.friend_sum(&b), 30);
    assert_eq!(a.friend_product(&b), 200);
}

#[test]
fn friend_compare_three_way() {
    let a = MyClass::new(10);
    let b = MyClass::new(20);
    assert_eq!(a.friend_compare(&b), -1);
    assert_eq!(b.friend_compare(&a), 1);
    assert_eq!(a.friend_compare(&MyClass::new(10)), 0);
}
