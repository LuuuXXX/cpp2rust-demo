//! 032_placement_new 冒烟测试
//!
//! 验证生成的 Rust FFI 绑定可编译、可链接 C++ 实现，且 placement new 行为正确。

use placement_new::*;

#[test]
fn smoke_buffer_capacity() {
    let buf = Buffer::new(64);
    assert_eq!(buf.capacity(), 64);
}

#[test]
fn smoke_buffer_construct_and_read() {
    let mut buf = Buffer::new(64);
    assert_eq!(buf.construct_at(0, 42), 42);
    assert_eq!(buf.value_at(0), 42, "应能读回 placement new 构造的值");
    assert_eq!(buf.size(), 4);
}

#[test]
fn smoke_buffer_out_of_range() {
    let mut buf = Buffer::new(4);
    assert_eq!(buf.construct_at(8, 1), -1, "越界 offset 应返回 -1");
}

#[test]
fn smoke_object_array_emplace() {
    let mut arr = ObjectArray::new(3);
    assert_eq!(arr.count(), 3);
    assert_eq!(arr.element_size(), 4);
    for i in 0..3 {
        arr.emplace(i, (i + 1) * 10);
    }
    assert_eq!(arr.at(0), 10);
    assert_eq!(arr.at(1), 20);
    assert_eq!(arr.at(2), 30);
}

#[test]
fn smoke_object_array_out_of_range() {
    let mut arr = ObjectArray::new(2);
    assert_eq!(arr.emplace(5, 1), -1);
    assert_eq!(arr.at(5), -1);
}
