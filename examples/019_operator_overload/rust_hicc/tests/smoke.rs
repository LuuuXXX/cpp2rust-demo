//! 019_operator_overload 冒烟测试：运算符重载经命名包装绑定。

use operator_overload::*;

#[test]
fn arithmetic_operators() {
    let a = Number::new(10);
    let b = Number::new(3);
    assert_eq!(a.op_add(&b).value(), 13);
    assert_eq!(a.op_sub(&b).value(), 7);
    assert_eq!(a.op_mul(&b).value(), 30);
    assert_eq!(a.op_div(&b).value(), 3);
}

#[test]
fn unary_and_compare() {
    let a = Number::new(10);
    let b = Number::new(3);
    assert_eq!(a.op_neg().value(), -10);
    assert_eq!(a.compare(&b), 1);
    assert_eq!(b.compare(&a), -1);
    assert_eq!(a.compare(&Number::new(10)), 0);
}

#[test]
fn increment_decrement_and_compound() {
    let mut a = Number::new(5);
    a.increment();
    assert_eq!(a.value(), 6);
    a.decrement();
    a.decrement();
    assert_eq!(a.value(), 4);

    let b = Number::new(10);
    a.add_assign(&b);
    assert_eq!(a.value(), 14);
    a.sub_assign(&b);
    assert_eq!(a.value(), 4);
}
