use functional_bind::*;
use std::ffi::CString;

#[test]
fn smoke_adder() {
    let mut adder = adder_new(100);
    assert_eq!(adder.add(50), 150, "adder(100).add(50) should return 150");
    assert_eq!(adder.add(30), 130, "adder(100).add(30) should return 130");
}

#[test]
fn smoke_adder_zero() {
    let mut adder = adder_new(0);
    assert_eq!(adder.add(42), 42, "adder(0).add(42) should return 42");
}

#[test]
fn smoke_multiplier() {
    let mut multiplier = multiplier_new(7);
    assert_eq!(multiplier.multiply(6), 42, "multiplier(7).multiply(6) should return 42");
    assert_eq!(multiplier.multiply(11), 77, "multiplier(7).multiply(11) should return 77");
}

#[test]
fn smoke_string_processor_count_char() {
    let mut processor = string_processor_new();
    let target = CString::new("hello world!").unwrap();
    processor.set_target(target.as_ptr());
    assert_eq!(processor.count_char('l' as i8), 3, "'hello world!' has 3 'l's");
    assert_eq!(processor.count_char('o' as i8), 2, "'hello world!' has 2 'o's");
    assert_eq!(processor.count_char('h' as i8), 1, "'hello world!' has 1 'h'");
}
