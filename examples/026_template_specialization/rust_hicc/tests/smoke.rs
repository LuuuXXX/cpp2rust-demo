//! 026_template_specialization 冒烟测试
//!
//! 验证生成的 Rust FFI 绑定可编译、可链接 C++ 实现，且基本行为正确。

use template_specialization::*;

#[test]
fn smoke_int_holder_get() {
    let ih = IntHolder::new(42);
    assert_eq!(ih.get(), 42, "IntHolder::get() 应返回构造时传入的值");
}

#[test]
fn smoke_int_holder_describe() {
    let ih = IntHolder::new(10);
    let desc = unsafe { std::ffi::CStr::from_ptr(ih.describe()) };
    let s = desc.to_string_lossy();
    assert!(s.contains("generic"), "通用模板 describe() 应标注 generic");
}

#[test]
fn smoke_double_holder_get() {
    let dh = DoubleHolder::new(3.14);
    assert!((dh.get() - 3.14).abs() < 1e-10, "DoubleHolder::get() 应返回构造时传入的值");
}

#[test]
fn smoke_double_holder_describe() {
    let dh = DoubleHolder::new(1.5);
    let desc = unsafe { std::ffi::CStr::from_ptr(dh.describe()) };
    let s = desc.to_string_lossy();
    assert!(s.contains("generic"), "通用模板 describe() 应标注 generic");
}

#[test]
fn smoke_string_holder_get() {
    let s = std::ffi::CString::new("hello").expect("CString::new failed");
    let sh = StringHolder::new(s.as_ptr());
    let result = unsafe { std::ffi::CStr::from_ptr(sh.get()) };
    assert_eq!(result.to_str().unwrap(), "hello", "StringHolder::get() 应返回构造时传入的字符串");
}

#[test]
fn smoke_string_holder_specialized_describe() {
    let s = std::ffi::CString::new("hi").expect("CString::new failed");
    let sh = StringHolder::new(s.as_ptr());
    let desc = unsafe { std::ffi::CStr::from_ptr(sh.describe()) };
    let d = desc.to_string_lossy();
    assert!(d.contains("std::string"), "特化版本 describe() 应标注 std::string");
    assert!(d.contains("length=2"), "特化版本 describe() 应含长度信息");
}
