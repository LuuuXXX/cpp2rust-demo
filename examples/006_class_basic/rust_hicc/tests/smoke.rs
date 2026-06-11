//! 006_class_basic 冒烟测试
//!
//! 验证 opaque 指针 + import_class! 模式下构造、读取与状态修改往返正确。

use class_basic::*;

#[test]
fn smoke_counter_initial_zero() {
    let counter = counter_new();
    assert_eq!(counter.get(), 0, "新建 Counter 初始值应为 0");
}

#[test]
fn smoke_counter_increment_decrement() {
    let mut counter = counter_new();
    counter.increment();
    counter.increment();
    counter.increment();
    assert_eq!(counter.get(), 3, "3 次 increment 后应为 3");
    counter.decrement();
    assert_eq!(counter.get(), 2, "再 1 次 decrement 后应为 2");
}
