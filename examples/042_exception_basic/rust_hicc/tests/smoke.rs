//! 042_exception_basic 冒烟测试
//!
//! C++ 异常不能跨 FFI 传播；采用「错误码 + 返回兜底值」模式。验证正常与异常路径。

use exception_basic::*;

#[test]
fn smoke_divide_normal() {
    let mut calc = calculator_new();
    assert_eq!(calc.divide(10, 2), 5, "10 / 2 应为 5");
    assert_eq!(calc.get_exception(), 0, "正常路径不应设置异常码");
}

#[test]
fn smoke_divide_by_zero_sets_exception() {
    let mut calc = calculator_new();
    let _ = calc.divide(10, 0);
    assert_ne!(calc.get_exception(), 0, "除零应设置非零异常码");
    // 清理后可继续正常运算。
    calc.clear_exception();
    assert_eq!(calc.divide(20, 4), 5, "清理异常后 20 / 4 应为 5");
    assert_eq!(calc.get_exception(), 0, "清理后异常码应为 0");
}

#[test]
fn smoke_string_to_int() {
    let mut calc = calculator_new();
    let ok = calc.string_to_int(b"123\0".as_ptr() as *const i8);
    assert_eq!(ok, 123, "合法字符串应转换为整数 123");
    calc.clear_exception();

    let _ = calc.string_to_int(b"abc\0".as_ptr() as *const i8);
    assert_ne!(calc.get_exception(), 0, "非法字符串应设置非零异常码");
}
