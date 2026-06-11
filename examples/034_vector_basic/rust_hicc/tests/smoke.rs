//! 034_vector_basic 冒烟测试
//!
//! 验证生成的 Rust FFI 绑定可编译、可链接 C++ 实现，且基本行为正确。

use vector_basic::*;

#[test]
fn smoke_int_vector_new() {
    let vec = int_vector_new();
    assert!(vec.empty(), "新建 IntVector 应为空");
    assert_eq!(vec.size(), 0, "新建 IntVector size 应为 0");
}

#[test]
fn smoke_int_vector_push_get() {
    let mut vec = int_vector_new();
    vec.push_back(10);
    vec.push_back(20);
    vec.push_back(30);
    assert_eq!(vec.size(), 3, "push_back 3 次后 size 应为 3");
    assert_eq!(vec.get(0), 10, "get(0) 应等于第一个元素");
    assert_eq!(vec.get(1), 20, "get(1) 应等于第二个元素");
    assert_eq!(vec.get(2), 30, "get(2) 应等于第三个元素");
}

#[test]
fn smoke_int_vector_set() {
    let mut vec = int_vector_new();
    vec.push_back(100);
    vec.set(0, 999);
    assert_eq!(vec.get(0), 999, "set(0, 999) 后 get(0) 应为 999");
}

#[test]
fn smoke_int_vector_clear() {
    let mut vec = int_vector_new();
    vec.push_back(1);
    vec.push_back(2);
    assert!(!vec.empty(), "push_back 后不应为空");
    vec.clear();
    assert!(vec.empty(), "clear 后应为空");
    assert_eq!(vec.size(), 0, "clear 后 size 应为 0");
}

#[test]
fn smoke_string_vector_type_available() {
    // 验证 StringVector 类型可用（编译期类型检查）
    let _ = string_vector_new();
}
