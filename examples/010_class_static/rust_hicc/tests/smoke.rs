//! 010_class_static 冒烟测试
//!
//! 验证生成的 Rust FFI 绑定可编译、可链接 C++ 实现，且基本行为正确。

use class_static::*;
use hicc::AbiClass;

#[test]
fn smoke_static_instance_count() {
    counter_reset_instance_count();
    assert_eq!(counter_get_instance_count(), 0, "reset 后计数应为 0");
}

#[test]
fn smoke_static_instance_tracking() {
    counter_reset_instance_count();

    let c1 = unsafe { counter_new().into_unique() };
    assert_eq!(counter_get_instance_count(), 1, "创建 1 个实例后计数应为 1");

    let _c2 = unsafe { counter_new().into_unique() };
    assert_eq!(counter_get_instance_count(), 2, "创建 2 个实例后计数应为 2");

    drop(c1);
    assert_eq!(counter_get_instance_count(), 1, "drop 1 个后计数应为 1");

    counter_reset_instance_count();
    assert_eq!(counter_get_instance_count(), 0, "reset 后计数应为 0");
}

#[test]
fn smoke_counter_value() {
    let mut c = unsafe { counter_new().into_unique() };
    assert_eq!(c.get_value(), 0, "初始值应为 0");
    c.increment();
    c.increment();
    c.increment();
    assert_eq!(c.get_value(), 3, "increment 3 次后应为 3");
}
