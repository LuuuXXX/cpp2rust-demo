//! 011_class_const 冒烟测试
//!
//! 验证 const 成员函数（getter）与可变成员函数（add/subtract/clear）行为。

use class_const::*;

#[test]
fn smoke_calculator_add_subtract() {
    let mut calc = calculator_new();
    assert_eq!(calc.get_value(), 0, "初始值应为 0");
    calc.add(10);
    assert_eq!(calc.get_value(), 10, "add(10) 后应为 10");
    calc.add(5);
    assert_eq!(calc.get_value(), 15, "再 add(5) 后应为 15");
    calc.subtract(3);
    assert_eq!(calc.get_value(), 12, "subtract(3) 后应为 12");
}

#[test]
fn smoke_calculator_history_and_clear() {
    let mut calc = calculator_new();
    calc.add(10);
    calc.add(5);
    calc.subtract(3);
    assert_eq!(calc.get_history_count(), 3, "三次操作后历史计数应为 3");
    calc.clear();
    assert_eq!(calc.get_value(), 0, "clear 后值应为 0");
    assert_eq!(calc.get_history_count(), 0, "clear 后历史计数应为 0");
}
