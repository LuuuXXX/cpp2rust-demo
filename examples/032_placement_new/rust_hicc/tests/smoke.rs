//! 032_placement_new 冒烟测试
//!
//! placement new 在预分配缓冲中构造对象；验证容量、初始大小与构造后大小。

use placement_new::*;

#[test]
fn smoke_buffer_capacity_and_size() {
    let mut buffer = buffer_new(1024);
    assert_eq!(buffer.capacity(), 1024, "缓冲容量应为构造参数 1024");
    assert_eq!(buffer.size(), 0, "未构造对象时已用大小应为 0");

    // 在偏移 0 处构造一个对象后，已用大小应增长。
    let _p = buffer.construct(0);
    assert!(buffer.size() > 0, "construct 后已用大小应大于 0");
}

#[test]
fn smoke_vector_buffer_element_size() {
    let vec_buffer = vector_buffer_new(10);
    assert!(vec_buffer.element_size() > 0, "VectorBuffer 元素大小应大于 0");
}
