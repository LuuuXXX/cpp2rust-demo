//! 002_function_overload 冒烟测试
//!
//! 验证生成的 Rust FFI 绑定可编译、可链接 C++ 实现，且基本行为正确。

use function_overload::*;

#[test]
fn smoke_add_int() {
    assert_eq!(add_int(1, 2), 3, "add_int(1, 2) 应返回 3");
    assert_eq!(add_int(-5, 10), 5, "add_int(-5, 10) 应返回 5");
}

#[test]
fn smoke_add_double() {
    assert!((add_double(1.5, 2.5) - 4.0).abs() < 1e-10, "add_double(1.5, 2.5) 应返回 4.0");
}

#[test]
fn smoke_add_strings() {
    let a = b"Hello\0".as_ptr() as *const i8;
    let b = b" World\0".as_ptr() as *const i8;
    let ptr = unsafe { add_strings(a, b) };
    let s = unsafe { std::ffi::CStr::from_ptr(ptr) }.to_str().unwrap();
    assert_eq!(s, "Hello World", "add_strings 应拼接字符串");
}

#[test]
fn smoke_sum3() {
    assert_eq!(sum3(1, 2, 3), 6, "sum3(1, 2, 3) 应返回 6");
    assert_eq!(sum3(10, 20, 30), 60, "sum3(10, 20, 30) 应返回 60");
}
