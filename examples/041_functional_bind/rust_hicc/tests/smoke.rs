//! 041_functional_bind 冒烟测试
//!
//! std::bind 风格的部分应用经 opaque 对象暴露；验证绑定后的调用结果。

use functional_bind::*;

#[test]
fn smoke_adder_binds_base() {
    let mut adder = adder_new(100);
    assert_eq!(adder.add(50), 150, "绑定基数 100 后 add(50) 应为 150");
    assert_eq!(adder.add(30), 130, "绑定基数 100 后 add(30) 应为 130");
}

#[test]
fn smoke_multiplier_binds_factor() {
    let mut multiplier = multiplier_new(7);
    assert_eq!(multiplier.multiply(6), 42, "绑定乘数 7 后 multiply(6) 应为 42");
    assert_eq!(multiplier.multiply(11), 77, "绑定乘数 7 后 multiply(11) 应为 77");
}

#[test]
fn smoke_string_processor_count_char() {
    let mut processor = string_processor_new();
    let target = std::ffi::CString::new("hello world!").unwrap();
    processor.set_target(target.as_ptr());
    assert_eq!(processor.count_char(b'l' as i8), 3, "'l' 出现 3 次");
    assert_eq!(processor.count_char(b'o' as i8), 2, "'o' 出现 2 次");
    assert_eq!(processor.count_char(b'h' as i8), 1, "'h' 出现 1 次");
}

#[test]
fn smoke_bound_free_functions() {
    assert_eq!(add_five(3), 8, "add_five(3) 应为 8");
    assert_eq!(add_ten(3), 13, "add_ten(3) 应为 13");
}
