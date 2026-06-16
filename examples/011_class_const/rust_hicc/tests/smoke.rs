//! 011_class_const 冒烟测试：const / 非 const 成员函数。

use class_const::*;

#[test]
fn const_methods_read_state() {
    let calc = Calculator::new();
    // const 方法以 &self 读取初始状态。
    assert_eq!(calc.value(), 0);
    assert_eq!(calc.history_count(), 0);
}

#[test]
fn mutating_methods_update_state() {
    let mut calc = Calculator::new();
    calc.add(10);
    calc.add(5);
    calc.subtract(3);
    assert_eq!(calc.value(), 12);
    assert_eq!(calc.history_count(), 3, "每次 add/subtract 记录一条历史");

    calc.clear();
    assert_eq!(calc.value(), 0);
    assert_eq!(calc.history_count(), 0);
}
