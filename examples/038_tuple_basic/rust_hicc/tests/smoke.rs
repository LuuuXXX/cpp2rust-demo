//! 038_tuple_basic 冒烟测试
//!
//! 验证生成的 Rust FFI 绑定可编译、可链接 C++ 实现，且基本行为正确。

use tuple_basic::*;
use std::ffi::{CStr, CString};

#[test]
fn smoke_tuple2_new() {
    let second = CString::new("hello").unwrap();
    let tuple = unsafe { tuple2_new(42, second.as_ptr()) };
    assert_eq!(tuple.get_first(), 42, "Tuple2 first 应为 42");
    let second_str = unsafe { CStr::from_ptr(tuple.get_second()) };
    assert_eq!(second_str.to_str().unwrap(), "hello", "Tuple2 second 应为 hello");
}

#[test]
fn smoke_tuple3_new() {
    let third = CString::new("world").unwrap();
    let tuple = unsafe { tuple3_new(100, 3.14, third.as_ptr()) };
    assert_eq!(tuple.get_first(), 100, "Tuple3 first 应为 100");
    assert!((tuple.get_second() - 3.14).abs() < 1e-9, "Tuple3 second 应约为 3.14");
    let third_str = unsafe { CStr::from_ptr(tuple.get_third()) };
    assert_eq!(third_str.to_str().unwrap(), "world", "Tuple3 third 应为 world");
}

#[test]
fn smoke_tuple4_new() {
    let third = CString::new("tuple").unwrap();
    let tuple = unsafe { tuple4_new(1, 2.71, third.as_ptr(), 4) };
    assert_eq!(tuple.get_first(), 1);
    assert!((tuple.get_second() - 2.71).abs() < 1e-9);
    let third_str = unsafe { CStr::from_ptr(tuple.get_third()) };
    assert_eq!(third_str.to_str().unwrap(), "tuple");
    assert_eq!(tuple.get_fourth(), 4);
}
