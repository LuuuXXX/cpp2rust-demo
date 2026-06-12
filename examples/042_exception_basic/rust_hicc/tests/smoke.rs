//! 042_exception_basic 冒烟测试
//!
//! 验证生成的 Rust FFI 绑定可编译、可链接 C++ 实现，且基本行为正确。

use exception_basic::*;

#[test]
fn smoke_calculator_divide_normal() {
    let mut calc = calculator_new();
    let result = calc.divide(10, 2);
    assert_eq!(result, 5, "10 / 2 应返回 5");
    let ex = calc.get_exception();
    assert_eq!(ex, 0, "正常除法不应设置异常码");
}

#[test]
fn smoke_calculator_divide_by_zero() {
    let mut calc = calculator_new();
    let result = calc.divide(10, 0);
    assert_eq!(result, 0, "除以零应返回 0");
    let ex = calc.get_exception();
    assert_eq!(ex, 3, "除以零应设置异常码 3 (EXCEPTION_RUNTIME_ERROR)");
}

#[test]
fn smoke_calculator_clear_exception() {
    let mut calc = calculator_new();
    // 触发异常
    calc.divide(10, 0);
    assert_eq!(calc.get_exception(), 3, "异常应已设置");
    // 清除异常
    calc.clear_exception();
    assert_eq!(calc.get_exception(), 0, "清除后异常码应为 0");
    // 再次正常除法
    let result = calc.divide(20, 4);
    assert_eq!(result, 5, "20 / 4 应返回 5");
    assert_eq!(calc.get_exception(), 0, "正常除法后不应有异常");
}

#[test]
fn smoke_calculator_string_to_int_valid() {
    let mut calc = calculator_new();
    let result = calc.string_to_int("123\0".as_ptr() as *const i8);
    assert_eq!(result, 123, "string_to_int(\"123\") 应返回 123");
    assert_eq!(calc.get_exception(), 0, "合法字符串转换不应设置异常");
}

#[test]
fn smoke_calculator_string_to_int_invalid() {
    let mut calc = calculator_new();
    let result = calc.string_to_int("abc\0".as_ptr() as *const i8);
    assert_eq!(result, 0, "非法字符串转换应返回 0");
    let ex = calc.get_exception();
    assert_eq!(ex, 1, "非法字符串应设置异常码 1 (EXCEPTION_INVALID_ARGUMENT)");
}

#[test]
fn smoke_calculator_divide_after_recovery() {
    let mut calc = calculator_new();
    // 正常路径
    assert_eq!(calc.divide(100, 10), 10);
    assert_eq!(calc.get_exception(), 0);
    // 错误路径
    assert_eq!(calc.divide(1, 0), 0);
    assert_eq!(calc.get_exception(), 3);
    calc.clear_exception();
    // 恢复正常
    assert_eq!(calc.divide(50, 5), 10);
    assert_eq!(calc.get_exception(), 0);
}

#[test]
fn smoke_check_exception_helper() {
    let mut calc = calculator_new();
    calc.divide(10, 0);
    // check_exception 应打印错误信息并清除异常
    check_exception(&mut calc, "测试除零");
    // 清除后异常码应为 0
    assert_eq!(calc.get_exception(), 0, "check_exception 应已清除异常");
}
