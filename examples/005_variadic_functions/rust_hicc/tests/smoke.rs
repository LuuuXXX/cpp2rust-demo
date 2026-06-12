//! 005_variadic_functions 冒烟测试
//!
//! 验证生成的 Rust FFI 绑定可编译、可链接 C++ 实现，且基本行为正确。

use variadic_functions::*;

#[test]
fn smoke_sum_3() {
    assert_eq!(sum_3(1, 2, 3), 6, "sum_3(1, 2, 3) 应返回 6");
    assert_eq!(sum_3(10, 20, 30), 60, "sum_3(10, 20, 30) 应返回 60");
}

#[test]
fn smoke_sum_5() {
    assert_eq!(sum_5(1, 2, 3, 4, 5), 15, "sum_5(1..5) 应返回 15");
    assert_eq!(sum_5(10, 20, 30, 40, 50), 150, "sum_5(10..50) 应返回 150");
}

#[test]
fn smoke_sum_negative() {
    assert_eq!(sum_3(-1, -2, -3), -6, "sum_3(-1, -2, -3) 应返回 -6");
    assert_eq!(sum_5(-1, 2, -3, 4, -5), -3, "sum_5 混合正负应正确求和");
}
