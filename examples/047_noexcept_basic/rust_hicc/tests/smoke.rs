//! 047_noexcept_basic 冒烟测试
//!
//! 验证生成的 Rust FFI 绑定可编译、可链接 C++ 实现，且 noexcept 基本行为正确。

use noexcept_basic::*;

#[test]
fn smoke_noexcept_functions() {
    assert_eq!(noexcept_add(2, 3), 5);
    assert_eq!(noexcept_multiply(4, 5), 20);
    assert_eq!(conditional_abs(-7), 7);
    assert_eq!(conditional_abs(7), 7);
    assert_eq!(safe_divide(10, 2), 5);
    assert_eq!(safe_divide(10, 0), -1);
}

#[test]
fn smoke_noexcept_mover_is_per_object() {
    let mover = NoexceptMover::new(42);
    assert_eq!(mover.get_value(), 42);
}
