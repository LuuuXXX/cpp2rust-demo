//! 045_union_basic 冒烟测试
//!
//! 验证生成的 Rust FFI 绑定可编译、可链接 C++ 实现，且基本行为正确。

use union_basic::*;
use hicc::AbiClass;

#[test]
fn smoke_variant_int() {
    let v = variant_new_int(42);
    assert_eq!(v.get_type(), 0, "Variant INT 类型码应为 0");
    assert_eq!(v.get_int(), 42, "Variant int 值应为 42");
}

#[test]
fn smoke_variant_float() {
    let v = variant_new_float(3.14);
    assert_eq!(v.get_type(), 1, "Variant FLOAT 类型码应为 1");
    assert!((v.get_float() - 3.14f32).abs() < 0.01, "Variant float 值应接近 3.14");
}

#[test]
fn smoke_variant_string() {
    let v = unsafe { variant_new_string("hello\0".as_ptr() as *const i8) };
    assert_eq!(v.get_type(), 2, "Variant STRING 类型码应为 2");
    let s = unsafe { std::ffi::CStr::from_ptr(v.get_string()) };
    assert_eq!(s.to_str().unwrap(), "hello", "Variant string 值应为 hello");
}

#[test]
fn smoke_union_memory_overlay() {
    let mut u = union_new();
    unsafe { union_set_int(&u.as_mut_ptr(), 0x41414141) };
    let int_val = union_get_int(&u.as_mut_ptr());
    assert_eq!(int_val, 0x41414141i32, "读取 int 值应与写入一致");
    let float_bits = union_get_float(&u.as_mut_ptr()).to_bits();
    assert_eq!(float_bits, 0x41414141u32, "float 的位表示应与 int 相同（union 共享内存）");
}

#[test]
fn smoke_variant_type_name() {
    assert_eq!(variant_type_name(0), "INT");
    assert_eq!(variant_type_name(1), "FLOAT");
    assert_eq!(variant_type_name(2), "STRING");
    assert_eq!(variant_type_name(99), "Unknown");
}
