//! 036_string_basic 冒烟测试
//!
//! 验证生成的 Rust FFI 绑定可编译、可链接 C++ 实现，且字符串操作行为正确。

use string_basic::*;
use std::ffi::{CStr, CString};

fn my_string(s: &str) -> MyString {
    let cs = CString::new(s).expect("CString::new failed");
    MyString::new(cs.as_ptr())
}

fn to_string(s: &MyString) -> String {
    unsafe { CStr::from_ptr(s.c_str()).to_string_lossy().into_owned() }
}

#[test]
fn smoke_length_and_empty() {
    let s = my_string("hello");
    assert_eq!(s.length(), 5);
    assert_eq!(s.empty(), 0);

    let empty = my_string("");
    assert_eq!(empty.length(), 0);
    assert_eq!(empty.empty(), 1);
}

#[test]
fn smoke_append_and_c_str() {
    let mut s = my_string("hello");
    let suffix = CString::new(", world").expect("CString::new failed");
    s.append(suffix.as_ptr());
    assert_eq!(s.length(), 12);
    assert_eq!(to_string(&s), "hello, world");
}

#[test]
fn smoke_at_bounds() {
    let s = my_string("abc");
    assert_eq!(s.at(0), b'a' as i8);
    assert_eq!(s.at(2), b'c' as i8);
    assert_eq!(s.at(-1), 0);
    assert_eq!(s.at(3), 0);
}

#[test]
fn smoke_compare() {
    let s = my_string("hello");
    let same = CString::new("hello").expect("CString::new failed");
    let later = CString::new("world").expect("CString::new failed");
    assert_eq!(s.compare(same.as_ptr()), 0);
    assert!(s.compare(later.as_ptr()) < 0);
}

#[test]
fn smoke_to_upper() {
    let mut s = my_string("Hello, Rust!");
    s.to_upper();
    assert_eq!(to_string(&s), "HELLO, RUST!");
}

#[test]
fn smoke_find() {
    let s = my_string("hello, world");
    let world = CString::new("world").expect("CString::new failed");
    let missing = CString::new("missing").expect("CString::new failed");
    assert_eq!(s.find(world.as_ptr()), 7);
    assert_eq!(s.find(missing.as_ptr()), -1);
}
