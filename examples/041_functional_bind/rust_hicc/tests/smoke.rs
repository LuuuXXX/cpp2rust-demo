//! 041_functional_bind 冒烟测试
//!
//! 验证生成的 Rust FFI 绑定可编译、可链接 C++ 实现，且 std::bind/状态行为正确。

use functional_bind::*;

#[test]
fn smoke_adder() {
    let adder = Adder::new(10);
    assert_eq!(adder.add(5), 15);
}

#[test]
fn smoke_multiplier() {
    let multiplier = Multiplier::new(3);
    assert_eq!(multiplier.multiply(4), 12);
}

#[test]
fn smoke_string_processor_count_char() {
    use std::ffi::CString;
    let mut processor = StringProcessor::new();
    let target = CString::new("banana").expect("CString::new failed");
    processor.set_target(target.as_ptr());
    assert_eq!(processor.count_char('a' as i8), 3);
}
