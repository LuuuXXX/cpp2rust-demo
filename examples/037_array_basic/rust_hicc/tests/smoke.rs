//! 037_array_basic 冒烟测试
//!
//! 验证生成的 Rust FFI 绑定可编译、可链接 C++ 实现，且基本行为正确。

use array_basic::*;

#[test]
fn smoke_int_array5_new() {
    let arr = int_array5_new();
    assert_eq!(arr.size(), 5, "IntArray5 固定大小应为 5");
    assert!(!arr.empty(), "固定大小数组 empty 应为 false");
}

#[test]
fn smoke_int_array5_set_get() {
    let mut arr = int_array5_new();
    for i in 0..5usize {
        arr.set(i, (i * 10) as i32);
    }
    for i in 0..5usize {
        assert_eq!(arr.get(i), (i * 10) as i32, "get({}) 值应匹配", i);
    }
}

#[test]
fn smoke_int_array5_at() {
    let mut arr = int_array5_new();
    arr.set(2, 42);
    assert_eq!(arr.at(2), 42, "at(2) 应等于 set(2, 42)");
}

#[test]
fn smoke_int_array5_new_from() {
    let values = [1i32, 2, 3, 4, 5];
    let arr = int_array5_new_from(values.as_ptr());
    assert_eq!(arr.size(), 5);
    assert_eq!(arr.get(0), 1, "第一个元素应为 1");
    assert_eq!(arr.get(4), 5, "最后一个元素应为 5");
}

#[test]
fn smoke_double_array3_type_available() {
    let arr = double_array3_new();
    assert_eq!(arr.size(), 3, "DoubleArray3 固定大小应为 3");
}

#[test]
fn smoke_string_array4_type_available() {
    let arr = string_array4_new();
    assert_eq!(arr.size(), 4, "StringArray4 固定大小应为 4");
}
