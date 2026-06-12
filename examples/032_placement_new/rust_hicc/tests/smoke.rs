//! 032_placement_new 冒烟测试
//!
//! 验证生成的 Rust FFI 绑定可编译、可链接 C++ 实现，且基本行为正确。

use placement_new::*;

#[test]
fn smoke_buffer_new() {
    let buffer = buffer_new(1024);
    assert_eq!(buffer.capacity(), 1024, "Buffer 容量应为 1024");
}

#[test]
fn smoke_buffer_data() {
    let mut buffer = buffer_new(512);
    let data_ptr = buffer.data();
    assert!(!data_ptr.is_null(), "data() 不应返回空指针");
}

#[test]
fn smoke_buffer_size() {
    let buffer = buffer_new(256);
    let size = buffer.size();
    // 初始 size 应为 0（未在 buffer 中构造任何对象）
    assert_eq!(size, 0, "初始 Buffer 的 size 应为 0");
}

#[test]
fn smoke_buffer_construct() {
    let mut buffer = buffer_new(1024);
    let ptr = buffer.construct(0);
    assert!(!ptr.is_null(), "construct 应返回非空指针");
    let size = buffer.size();
    assert!(size > 0, "construct 后 size 应大于 0");
}

#[test]
fn smoke_vector_buffer_new() {
    let vec_buffer = vector_buffer_new(10);
    let elem_size = vec_buffer.element_size();
    assert!(elem_size > 0, "element_size 应大于 0");
}

#[test]
fn smoke_vector_buffer_data() {
    let mut vec_buffer = vector_buffer_new(10);
    let data_ptr = vec_buffer.data();
    assert!(!data_ptr.is_null(), "VectorBuffer data() 不应返回空指针");
}

#[test]
fn smoke_vector_buffer_destroy_all() {
    let mut vec_buffer = vector_buffer_new(10);
    vec_buffer.destroy_all();
    // destroy_all 不崩溃即可
}
