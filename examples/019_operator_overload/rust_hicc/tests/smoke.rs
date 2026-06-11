//! 019_operator_overload 冒烟测试
//!
//! C++ 运算符重载在 FFI 中映射为命名 shim 函数；验证各运算结果。

use operator_overload::*;
use hicc::AbiClass;

#[test]
fn smoke_number_arithmetic() {
    let a = number_new(10);
    let b = number_new(3);

    let sum = number_add(&a.as_ptr(), &b.as_ptr());
    assert_eq!(number_getValue(&sum.as_ptr()), 13, "a + b 应为 13");

    let diff = number_sub(&a.as_ptr(), &b.as_ptr());
    assert_eq!(number_getValue(&diff.as_ptr()), 7, "a - b 应为 7");

    let prod = number_mul(&a.as_ptr(), &b.as_ptr());
    assert_eq!(number_getValue(&prod.as_ptr()), 30, "a * b 应为 30");

    let quot = number_div(&a.as_ptr(), &b.as_ptr());
    assert_eq!(number_getValue(&quot.as_ptr()), 3, "a / b 应为 3");
}

#[test]
fn smoke_number_unary_and_compare() {
    let a = number_new(10);
    let b = number_new(3);

    let neg = number_negate(&a.as_ptr());
    assert_eq!(number_getValue(&neg.as_ptr()), -10, "-a 应为 -10");

    let cmp = number_compare(&a.as_ptr(), &b.as_ptr());
    assert_eq!(cmp, 7, "compare(a, b) 应为 value 差 10-3=7");
}
