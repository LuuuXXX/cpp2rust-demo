//! 034_vector_basic 冒烟测试
//!
//! 验证生成的 Rust FFI 绑定可编译、可链接 C++ 实现，且容器操作行为正确。

use vector_basic::*;

#[test]
fn smoke_int_vector_push_and_sum() {
    let mut v = IntVector::new();
    assert_eq!(v.empty(), 1);
    for i in 0..5 {
        v.push_back(i * 10);
    }
    assert_eq!(v.size(), 5);
    assert_eq!(v.sum(), 0 + 10 + 20 + 30 + 40);
    assert_eq!(v.empty(), 0);
}

#[test]
fn smoke_int_vector_get_set() {
    let mut v = IntVector::new();
    v.push_back(1);
    v.push_back(2);
    v.push_back(3);
    assert_eq!(v.get(1), 2);
    v.set(1, 999);
    assert_eq!(v.get(1), 999);
}

#[test]
fn smoke_int_vector_pop_clear() {
    let mut v = IntVector::new();
    v.push_back(1);
    v.push_back(2);
    v.pop_back();
    assert_eq!(v.size(), 1);
    v.clear();
    assert_eq!(v.size(), 0);
}

#[test]
fn smoke_int_vector_reserve_capacity() {
    let mut v = IntVector::new();
    v.reserve(8);
    assert!(v.capacity() >= 8, "reserve 后容量应至少为 8");
}

#[test]
fn smoke_string_vector() {
    let mut sv = StringVector::new();
    for s in ["alpha", "beta"] {
        let cs = std::ffi::CString::new(s).expect("CString::new failed");
        sv.push_back(cs.as_ptr());
    }
    assert_eq!(sv.size(), 2);
    let g0 = unsafe { std::ffi::CStr::from_ptr(sv.get(0)).to_string_lossy().into_owned() };
    assert_eq!(g0, "alpha");
}
