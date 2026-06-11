//! 010_class_static 冒烟测试
//!
//! 静态成员（实例计数）是进程级全局状态，故用单一测试串行验证，
//! 避免并行测试间相互干扰。

use class_static::*;

#[test]
fn smoke_static_instance_count_lifecycle() {
    // 复位到已知初值，排除其它对象的影响。
    counter_reset_instance_count();
    assert_eq!(counter_get_instance_count(), 0, "复位后实例计数应为 0");

    let mut c1 = unsafe { counter_new().into_unique() };
    let c2 = unsafe { counter_new().into_unique() };
    assert_eq!(counter_get_instance_count(), 2, "创建 2 个后实例计数应为 2");

    // 实例方法：自增后读取。
    c1.increment();
    c1.increment();
    assert_eq!(c1.get_value(), 2, "c1 自增 2 次后值应为 2");

    drop(c1);
    assert_eq!(counter_get_instance_count(), 1, "销毁 c1 后实例计数应为 1");
    drop(c2);
    assert_eq!(counter_get_instance_count(), 0, "全部销毁后实例计数应为 0");
}
