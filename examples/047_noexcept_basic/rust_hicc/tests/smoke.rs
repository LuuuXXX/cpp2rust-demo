//! 047_noexcept_basic 冒烟测试
//!
//! 验证生成的 Rust FFI 绑定可编译、可链接 C++ 实现，且基本行为正确。

use noexcept_basic::*;

#[test]
fn smoke_noexcept_add() {
    assert_eq!(noexcept_add(10, 20), 30, "noexcept_add(10, 20) 应返回 30");
    assert_eq!(noexcept_add(-5, 5), 0, "noexcept_add(-5, 5) 应返回 0");
    assert_eq!(noexcept_add(0, 0), 0, "noexcept_add(0, 0) 应返回 0");
}

#[test]
fn smoke_noexcept_multiply() {
    assert_eq!(noexcept_multiply(6, 7), 42, "noexcept_multiply(6, 7) 应返回 42");
    assert_eq!(noexcept_multiply(-3, 4), -12, "noexcept_multiply(-3, 4) 应返回 -12");
}

#[test]
fn smoke_conditional_abs() {
    assert_eq!(conditional_abs(-42), 42, "conditional_abs(-42) 应返回 42");
    assert_eq!(conditional_abs(42), 42, "conditional_abs(42) 应返回 42");
    assert_eq!(conditional_abs(0), 0, "conditional_abs(0) 应返回 0");
}

#[test]
fn smoke_noexcept_mover() {
    let mover = noexcept_mover_new(100);
    assert_eq!(mover.get_value(), 100, "NoexceptMover 初始值应为 100");
}

#[test]
fn smoke_noexcept_mover_move() {
    use hicc::AbiClass;
    let mut mover1 = noexcept_mover_new(200);
    let mover2 = unsafe { noexcept_mover_move(&mover1.as_mut_ptr()) };
    assert_eq!(mover2.get_value(), 200, "移动后的 NoexceptMover 值应为 200");
}
