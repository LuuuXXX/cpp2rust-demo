//! 037_array_basic 冒烟测试
//!
//! 验证生成的 Rust FFI 绑定可编译、可链接 C++ 实现，且固定数组操作行为正确。

use array_basic::*;

#[test]
fn smoke_int_array_size_and_zero_init() {
    let a = IntArray::new();
    assert_eq!(a.size(), 8);
    assert_eq!(a.sum(), 0);
    assert_eq!(a.min(), 0);
    assert_eq!(a.max(), 0);
}

#[test]
fn smoke_int_array_set_get_sum() {
    let mut a = IntArray::new();
    for i in 0..a.size() {
        a.set(i, i + 1);
    }
    assert_eq!(a.get(0), 1);
    assert_eq!(a.get(7), 8);
    assert_eq!(a.sum(), 36);
}

#[test]
fn smoke_int_array_fill_min_max() {
    let mut a = IntArray::new();
    a.fill(5);
    assert_eq!(a.sum(), 40);
    assert_eq!(a.min(), 5);
    assert_eq!(a.max(), 5);
    a.set(3, -9);
    a.set(6, 42);
    assert_eq!(a.min(), -9);
    assert_eq!(a.max(), 42);
}

#[test]
fn smoke_int_array_oob_is_safe() {
    let mut a = IntArray::new();
    a.fill(3);
    a.set(-1, 99);
    a.set(8, 99);
    assert_eq!(a.get(-1), 0);
    assert_eq!(a.get(8), 0);
    assert_eq!(a.sum(), 24);
}

#[test]
fn smoke_int_array_per_object_state() {
    let mut a = IntArray::new();
    let mut b = IntArray::new();
    a.fill(1);
    b.fill(2);
    a.set(0, 10);
    assert_eq!(a.sum(), 17);
    assert_eq!(b.sum(), 16);
}
