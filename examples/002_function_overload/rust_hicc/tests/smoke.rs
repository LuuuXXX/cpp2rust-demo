//! 002_function_overload 冒烟测试
//!
//! 验证重载在 FFI 中被映射为命名函数后行为正确。

use function_overload::*;

#[test]
fn smoke_add_int() {
    assert_eq!(add_int(1, 2), 3, "add_int 应返回整数和");
}

#[test]
fn smoke_add_double() {
    assert!((add_double(1.5, 2.5) - 4.0).abs() < 1e-12, "add_double 应返回浮点和");
}

#[test]
fn smoke_sum3() {
    assert_eq!(sum3(1, 2, 3), 6, "sum3 应返回三数之和");
}

#[test]
fn smoke_add_strings() {
    let result = unsafe {
        let a = b"Hello\0".as_ptr() as *const i8;
        let b = b" World\0".as_ptr() as *const i8;
        let ptr = add_strings(a, b);
        std::ffi::CStr::from_ptr(ptr).to_string_lossy().into_owned()
    };
    assert_eq!(result, "Hello World", "add_strings 应拼接两个字符串");
}
