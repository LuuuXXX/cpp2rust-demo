//! 041_functional_bind 冒烟测试
//!
//! 验证生成的 Rust FFI 绑定可编译、可链接 C++ 实现，且基本行为正确。

use functional_bind::*;

#[test]
fn smoke_adder() {
    let mut adder = adder_new(100);
    assert_eq!(adder.add(50), 150, "adder(100).add(50) 应返回 150");
    assert_eq!(adder.add(30), 130, "adder(100).add(30) 应返回 130");
}

#[test]
fn smoke_adder_zero() {
    let mut adder = adder_new(0);
    assert_eq!(adder.add(42), 42, "adder(0).add(42) 应返回 42");
}

#[test]
fn smoke_multiplier() {
    let mut multiplier = multiplier_new(7);
    assert_eq!(multiplier.multiply(6), 42, "multiplier(7).multiply(6) 应返回 42");
    assert_eq!(multiplier.multiply(11), 77, "multiplier(7).multiply(11) 应返回 77");
}

#[test]
fn smoke_string_processor_count_char() {
    use std::ffi::CString;
    let mut processor = string_processor_new();
    let target = CString::new("hello world!").unwrap();
    processor.set_target(target.as_ptr());
    assert_eq!(processor.count_char('l' as i8), 3, "'hello world!' 中 'l' 出现 3 次");
    assert_eq!(processor.count_char('o' as i8), 2, "'hello world!' 中 'o' 出现 2 次");
    assert_eq!(processor.count_char('h' as i8), 1, "'hello world!' 中 'h' 出现 1 次");
}

#[test]
fn smoke_add_five_impl() {
    let result = add_five_impl(3, 5);
    assert_eq!(result, 8, "add_five_impl(3, 5) 应返回 8");
}

#[test]
fn smoke_add_ten_impl() {
    let result = add_ten_impl(2, 10);
    assert_eq!(result, 12, "add_ten_impl(2, 10) 应返回 12");
}

#[test]
fn smoke_add_five() {
    let result = add_five(10);
    assert_eq!(result, 15, "add_five(10) 应返回 15");
}

#[test]
fn smoke_add_ten() {
    let result = add_ten(10);
    assert_eq!(result, 20, "add_ten(10) 应返回 20");
}
