//! 028_variadic_template 冒烟测试
//!
//! 可变参数模板按元数展开为固定参数包装函数；验证各元数求和。

use variadic_template::*;

#[test]
fn smoke_sum_int_arities() {
    assert_eq!(sum_zero(), 0, "sum() 应为 0");
    assert_eq!(sum_1(1), 1, "sum(1) 应为 1");
    assert_eq!(sum_2(1, 2), 3, "sum(1,2) 应为 3");
    assert_eq!(sum_3(1, 2, 3), 6, "sum(1,2,3) 应为 6");
    assert_eq!(sum_4(1, 2, 3, 4), 10, "sum(1..4) 应为 10");
    assert_eq!(sum_5(1, 2, 3, 4, 5), 15, "sum(1..5) 应为 15");
}

#[test]
fn smoke_sum_double_arities() {
    assert!((sum_double_2(1.5, 2.5) - 4.0).abs() < 1e-12, "sum(1.5,2.5) 应为 4.0");
    assert!((sum_double_3(1.0, 2.0, 3.0) - 6.0).abs() < 1e-12, "sum(1,2,3) 应为 6.0");
}
