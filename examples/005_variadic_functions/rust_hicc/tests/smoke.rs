//! 005_variadic_functions 冒烟测试
//!
//! C 可变参数无法直接经 FFI 调用，需固定参数包装函数；验证包装函数行为。

use variadic_functions::*;

#[test]
fn smoke_sum_3() {
    assert_eq!(sum_3(1, 2, 3), 6, "sum_3 应返回三数之和");
}

#[test]
fn smoke_sum_5() {
    assert_eq!(sum_5(1, 2, 3, 4, 5), 15, "sum_5 应返回五数之和");
}
