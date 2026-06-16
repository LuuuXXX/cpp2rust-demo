//! 042_exception_basic 冒烟测试
//!
//! 验证生成的 Rust FFI 绑定可编译、可链接 C++ 实现，且异常状态为对象内状态。

use exception_basic::*;
use std::ffi::CString;

#[test]
fn smoke_calculator_error_state() {
    let mut calc = Calculator::new();

    assert_eq!(calc.divide(10, 2), 5);
    assert_eq!(calc.has_error(), 0);

    assert_eq!(calc.divide(1, 0), 0);
    assert_eq!(calc.last_error(), 3);
    assert_eq!(calc.has_error(), 1);
    calc.clear_error();
    assert_eq!(calc.has_error(), 0);

    let ok = CString::new("123").expect("CString::new failed");
    assert_eq!(calc.parse_int(ok.as_ptr()), 123);
    assert_eq!(calc.has_error(), 0);

    let invalid = CString::new("abc").expect("CString::new failed");
    assert_eq!(calc.parse_int(invalid.as_ptr()), 0);
    assert_eq!(calc.last_error(), 1);

    let huge = CString::new("99999999999999999999").expect("CString::new failed");
    assert_eq!(calc.parse_int(huge.as_ptr()), 0);
    assert_eq!(calc.last_error(), 2);
}
