//! 036_string_basic 冒烟测试
//!
//! 验证生成的 Rust FFI 绑定可编译、可链接 C++ 实现，且基本行为正确。

use string_basic::*;
use std::ffi::{CStr, CString};

#[test]
fn smoke_string_new_empty() {
    let s = string_new();
    assert_eq!(s.size(), 0, "新建空字符串 size 应为 0");
    assert!(s.empty(), "新建空字符串应为 empty");
}

#[test]
fn smoke_string_new_from() {
    let cs = CString::new("hello").unwrap();
    let s = unsafe { string_new_from(cs.as_ptr()) };
    assert_eq!(s.length(), 5, "length 应等于字节数");
    assert!(!s.empty(), "非空字符串 empty 应为 false");
    let content = unsafe { CStr::from_ptr(s.c_str()) };
    assert_eq!(content.to_str().unwrap(), "hello", "c_str 内容应匹配");
}

#[test]
fn smoke_string_equals_compare() {
    let cs = CString::new("hello").unwrap();
    let s = unsafe { string_new_from(cs.as_ptr()) };
    let eq_str = CString::new("hello").unwrap();
    assert!(s.equals(eq_str.as_ptr()), "equals 相同字符串应为 true");
    let ne_str = CString::new("world").unwrap();
    assert!(!s.equals(ne_str.as_ptr()), "equals 不同字符串应为 false");
    let cmp_eq = CString::new("hello").unwrap();
    assert_eq!(s.compare(cmp_eq.as_ptr()), 0, "compare 相同字符串应为 0");
}

#[test]
fn smoke_string_append() {
    let cs = CString::new("hello").unwrap();
    let mut s = unsafe { string_new_from(cs.as_ptr()) };
    let suffix = CString::new(", world").unwrap();
    s.append(suffix.as_ptr());
    assert_eq!(s.length(), 12, "append 后 length 应增加");
    let content = unsafe { CStr::from_ptr(s.c_str()) };
    assert_eq!(content.to_str().unwrap(), "hello, world");
}

#[test]
fn smoke_string_new_from_len() {
    let cs = CString::new("hello world").unwrap();
    // 仅取前 5 个字符
    let s = unsafe { string_new_from_len(cs.as_ptr(), 5) };
    assert_eq!(s.length(), 5, "string_new_from_len 应截取指定长度");
    let content = unsafe { CStr::from_ptr(s.c_str()) };
    assert_eq!(content.to_str().unwrap(), "hello");
}
