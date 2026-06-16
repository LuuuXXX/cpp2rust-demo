//! 048_summary 冒烟测试
//!
//! 验证生成的 Rust FFI 绑定可编译、可链接 C++ 实现，且对象状态与自由函数行为正确。

use summary::*;

#[test]
fn smoke_counter_state_is_per_object() {
    let mut counter = Counter::new();
    assert_eq!(counter.get(), 0);

    counter.increment();
    counter.increment();
    counter.increment();
    assert_eq!(counter.get(), 3);

    counter.decrement();
    assert_eq!(counter.get(), 2);

    counter.reset();
    assert_eq!(counter.get(), 0);
}

#[test]
fn smoke_free_functions() {
    assert_eq!(safe_add(2, 3), 5);
    assert_eq!(max_size(), 1024);
}
