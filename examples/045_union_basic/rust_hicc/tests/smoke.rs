//! 045_union_basic 冒烟测试
//!
//! 验证生成的 Rust FFI 绑定可编译、可链接 C++ 实现，且 union 操作行为正确。

use std::ffi::{CStr, CString};
use union_basic::*;

#[test]
fn smoke_variant_int() {
    let mut v = Variant::new();
    v.set_int(42);
    assert_eq!(v.get_type(), 0);
    assert_eq!(v.get_int(), 42);
}

#[test]
fn smoke_variant_float() {
    let mut v = Variant::new();
    v.set_float(2.5);
    assert_eq!(v.get_type(), 1);
    assert!((v.get_float() - 2.5).abs() < 1e-6);
}

#[test]
fn smoke_variant_string() {
    let mut v = Variant::new();
    let hi = CString::new("hi").expect("CString::new failed");
    v.set_string(hi.as_ptr());
    assert_eq!(v.get_type(), 2);
    let s = unsafe { CStr::from_ptr(v.get_string()).to_string_lossy().into_owned() };
    assert_eq!(s, "hi");
}

#[test]
fn smoke_int_float_union_int() {
    let mut u = IntFloatUnion::new();
    u.set_int(7);
    assert_eq!(u.get_int(), 7);
}

#[test]
fn smoke_int_float_union_float() {
    let mut u = IntFloatUnion::new();
    u.set_float(1.5);
    assert!((u.get_float() - 1.5).abs() < 1e-6);
}
