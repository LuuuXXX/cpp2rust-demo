//! 026_template_specialization 冒烟测试
//!
//! 验证生成的 Rust FFI 绑定可编译、可链接 C++ 实现，且基本行为正确。

use template_specialization::*;

#[test]
fn smoke_int_holder_get() {
    let ih = intholder_new(42);
    assert_eq!(ih.get(), 42, "IntHolder::get() 应返回构造时传入的值");
}

#[test]
fn smoke_int_holder_describe() {
    let ih = intholder_new(10);
    let desc = unsafe { std::ffi::CStr::from_ptr(ih.describe()) };
    let s = desc.to_string_lossy();
    assert!(!s.is_empty(), "describe() 不应返回空字符串");
}

#[test]
fn smoke_double_holder_get() {
    let dh = doubleholder_new(3.14);
    assert!((dh.get() - 3.14).abs() < 1e-10, "DoubleHolder::get() 应返回构造时传入的值");
}

#[test]
fn smoke_double_holder_describe() {
    let dh = doubleholder_new(1.5);
    let desc = unsafe { std::ffi::CStr::from_ptr(dh.describe()) };
    let s = desc.to_string_lossy();
    assert!(!s.is_empty(), "describe() 不应返回空字符串");
}

#[test]
fn smoke_string_holder_get() {
    let s = std::ffi::CString::new("hello").expect("CString::new failed");
    let sh = unsafe { stringholder_new(s.as_ptr()) };
    let result = unsafe { std::ffi::CStr::from_ptr(sh.get()) };
    assert_eq!(result.to_str().unwrap(), "hello", "StringHolder::get() 应返回构造时传入的字符串");
}
