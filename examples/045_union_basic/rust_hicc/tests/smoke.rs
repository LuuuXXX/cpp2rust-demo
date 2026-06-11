//! 045_union_basic 冒烟测试
//!
//! 验证 Variant 在 INT/FLOAT/STRING 三种类型下的存取，以及 union 内存重叠读写。

use union_basic::*;
use hicc::AbiClass;

#[test]
fn smoke_variant_int() {
    let v = variant_new_int(42);
    assert_eq!(v.get_type(), 0);
    assert_eq!(v.get_int(), 42);
}

#[test]
fn smoke_variant_float() {
    let v = variant_new_float(3.14);
    assert_eq!(v.get_type(), 1);
    assert!((v.get_float() - 3.14).abs() < 1e-5);
}

#[test]
fn smoke_variant_string() {
    let s = std::ffi::CString::new("Hello, Union!").unwrap();
    let v = unsafe { variant_new_string(s.as_ptr()) };
    assert_eq!(v.get_type(), 2);
    let out = unsafe { std::ffi::CStr::from_ptr(v.get_string()).to_str().unwrap() };
    assert_eq!(out, "Hello, Union!");
}

#[test]
fn smoke_union_int_float_overlay() {
    let mut u = union_new();
    unsafe { union_set_int(&u.as_mut_ptr(), 0x41414141); }
    assert_eq!(union_get_int(&u.as_mut_ptr()) as u32, 0x41414141);
    // 同一内存按 float 读取的位模式应保持一致
    let f = union_get_float(&u.as_mut_ptr());
    assert_eq!(f.to_bits(), 0x41414141);
}
