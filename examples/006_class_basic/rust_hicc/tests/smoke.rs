//! 006_class_basic 冒烟测试
//!
//! 验证生成的 Rust FFI 绑定可编译、可链接 C++ 实现，且基本行为正确。

use class_basic::*;

#[test]
fn smoke_counter_initial() {
    let counter = counter_new();
    assert_eq!(counter.get(), 0, "初始值应为 0");
}

#[test]
fn smoke_counter_increment() {
    let mut counter = counter_new();
    counter.increment();
    assert_eq!(counter.get(), 1, "increment 一次后应为 1");
    counter.increment();
    counter.increment();
    assert_eq!(counter.get(), 3, "increment 三次后应为 3");
}

#[test]
fn smoke_counter_decrement() {
    let mut counter = counter_new();
    counter.increment();
    counter.increment();
    counter.increment();
    counter.increment();
    counter.increment();
    counter.decrement();
    assert_eq!(counter.get(), 4, "5 次 increment + 1 次 decrement = 4");
}
