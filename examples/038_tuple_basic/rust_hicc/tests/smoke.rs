//! 038_tuple_basic 冒烟测试
//!
//! 验证生成的 Rust FFI 绑定可编译、可链接 C++ 实现，且 tuple 元素访问行为正确。

use tuple_basic::*;

#[test]
fn smoke_record_new_and_getters() {
    let name = std::ffi::CString::new("alice").expect("CString::new failed");
    let record = Record::new(42, 98.5, name.as_ptr());
    assert_eq!(record.id(), 42);
    assert!((record.score() - 98.5).abs() < 1e-9);
    let record_name = unsafe { std::ffi::CStr::from_ptr(record.name()).to_string_lossy().into_owned() };
    assert_eq!(record_name, "alice");
}

#[test]
fn smoke_record_set_id() {
    let name = std::ffi::CString::new("bob").expect("CString::new failed");
    let mut record = Record::new(1, 2.5, name.as_ptr());
    record.set_id(7);
    assert_eq!(record.id(), 7);
    assert!((record.score() - 2.5).abs() < 1e-9);
}

#[test]
fn smoke_record_set_score() {
    let name = std::ffi::CString::new("carol").expect("CString::new failed");
    let mut record = Record::new(3, 4.5, name.as_ptr());
    record.set_score(9.75);
    assert_eq!(record.id(), 3);
    assert!((record.score() - 9.75).abs() < 1e-9);
}

#[test]
fn smoke_record_per_object_state() {
    let alice = std::ffi::CString::new("alice").expect("CString::new failed");
    let bob = std::ffi::CString::new("bob").expect("CString::new failed");
    let mut first = Record::new(1, 10.0, alice.as_ptr());
    let second = Record::new(2, 20.0, bob.as_ptr());

    first.set_id(11);
    first.set_score(99.0);

    assert_eq!(first.id(), 11);
    assert!((first.score() - 99.0).abs() < 1e-9);
    assert_eq!(second.id(), 2);
    assert!((second.score() - 20.0).abs() < 1e-9);

    let first_name = unsafe { std::ffi::CStr::from_ptr(first.name()).to_string_lossy().into_owned() };
    let second_name = unsafe { std::ffi::CStr::from_ptr(second.name()).to_string_lossy().into_owned() };
    assert_eq!(first_name, "alice");
    assert_eq!(second_name, "bob");
}
