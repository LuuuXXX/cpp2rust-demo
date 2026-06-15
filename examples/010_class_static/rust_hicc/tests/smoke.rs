//! 010_class_static 冒烟测试：静态计数随实例构造/Drop 维护。
//!
//! 静态计数是跨实例共享的全局状态，多个测试并行会相互干扰，故用进程内
//! 互斥锁串行化访问该计数器（对标含全局状态的最佳实践）。

use class_static::*;
use std::sync::Mutex;

static COUNTER_LOCK: Mutex<()> = Mutex::new(());

#[test]
fn instance_value_increment() {
    let _guard = COUNTER_LOCK.lock().unwrap();
    let mut c = Counter::new();
    assert_eq!(c.value(), 0);
    c.increment();
    c.increment();
    c.increment();
    assert_eq!(c.value(), 3);
}

#[test]
fn static_count_tracks_lifetime() {
    let _guard = COUNTER_LOCK.lock().unwrap();
    counter_reset_instance_count();
    assert_eq!(counter_instance_count(), 0);

    let c1 = Counter::new();
    assert_eq!(counter_instance_count(), 1);

    let c2 = Counter::new();
    assert_eq!(counter_instance_count(), 2);

    drop(c1);
    assert_eq!(counter_instance_count(), 1, "drop 一个实例后计数应减一");

    drop(c2);
    assert_eq!(counter_instance_count(), 0, "全部 drop 后计数应为 0");
}
