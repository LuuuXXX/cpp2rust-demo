//! 046_constexpr_basic 冒烟测试
//!
//! 验证生成的 Rust FFI 绑定可编译、可链接 C++ 实现，且 constexpr 类与自由函数行为正确。

use constexpr_basic::*;

#[test]
fn smoke_constexpr_point_positive_is_per_object() {
    let p = ConstexprPoint::new(3, 4);
    assert_eq!(p.x(), 3);
    assert_eq!(p.y(), 4);
    assert_eq!(p.manhattan_distance(), 7);
}

#[test]
fn smoke_constexpr_point_negative_is_per_object() {
    let p = ConstexprPoint::new(-2, -5);
    assert_eq!(p.manhattan_distance(), 7);
}

#[test]
fn smoke_constexpr_free_functions() {
    assert_eq!(fibonacci_10(), 55);
    assert_eq!(array_size(), 16);
}
