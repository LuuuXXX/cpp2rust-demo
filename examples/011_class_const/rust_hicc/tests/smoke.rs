//! 011_class_const 冒烟测试
//!
//! 验证生成的 Rust FFI 绑定可编译、可链接 C++ 实现，且基本行为正确。

use class_const::*;

#[test]
fn smoke_calculator_initial() {
    let calc = calculator_new();
    assert_eq!(calc.get_value(), 0, "初始值应为 0");
    assert_eq!(calc.get_history_count(), 0, "初始历史计数应为 0");
}

#[test]
fn smoke_calculator_add() {
    let mut calc = calculator_new();
    calc.add(10);
    assert_eq!(calc.get_value(), 10, "add(10) 后应为 10");
    calc.add(5);
    assert_eq!(calc.get_value(), 15, "add(5) 后应为 15");
}

#[test]
fn smoke_calculator_subtract() {
    let mut calc = calculator_new();
    calc.add(20);
    calc.subtract(3);
    assert_eq!(calc.get_value(), 17, "20 - 3 = 17");
}

#[test]
fn smoke_calculator_clear() {
    let mut calc = calculator_new();
    calc.add(10);
    calc.add(5);
    calc.subtract(3);
    calc.clear();
    assert_eq!(calc.get_value(), 0, "clear 后值应为 0");
    assert_eq!(calc.get_history_count(), 0, "clear 后历史计数应为 0");
}
