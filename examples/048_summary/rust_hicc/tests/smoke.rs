//! 048_summary 冒烟测试
//!
//! 验证生成的 Rust FFI 绑定可编译、可链接 C++ 实现，且基本行为正确。

use summary::*;

#[test]
fn smoke_counter_new() {
    let counter = counter_new();
    assert_eq!(counter.get(), 0, "Counter 初始值应为 0");
}

#[test]
fn smoke_counter_increment() {
    let mut counter = counter_new();
    counter.increment();
    assert_eq!(counter.get(), 1, "increment 一次后值应为 1");
    counter.increment();
    assert_eq!(counter.get(), 2, "increment 两次后值应为 2");
}

#[test]
fn smoke_counter_decrement() {
    let mut counter = counter_new();
    counter.increment();
    counter.increment();
    counter.increment();
    counter.decrement();
    assert_eq!(counter.get(), 2, "3 次 increment + 1 次 decrement 后值应为 2");
}

#[test]
fn smoke_safe_add() {
    assert_eq!(safe_add(10, 20), 30, "safe_add(10, 20) 应返回 30");
    assert_eq!(safe_add(-5, 5), 0, "safe_add(-5, 5) 应返回 0");
    assert_eq!(safe_add(0, 0), 0, "safe_add(0, 0) 应返回 0");
}

#[test]
fn smoke_get_max_size() {
    let max_size = get_max_size();
    assert!(max_size > 0, "get_max_size() 应返回正数");
}
