use functional_bind::*;
use std::ffi::CString;

fn main() {
    println!("=== 041_functional_bind - std::bind（hicc 直出）===\n");

    let adder = Adder::new(10);
    println!("adder.add(5)={}", adder.add(5));

    let multiplier = Multiplier::new(3);
    println!("multiplier.multiply(4)={}", multiplier.multiply(4));

    let mut processor = StringProcessor::new();
    let target = CString::new("banana").expect("CString::new failed");
    processor.set_target(target.as_ptr());
    println!("count('a')={}", processor.count_char('a' as i8));

    println!("\nRust FFI: hicc 绑定内部持有 std::bind(std::function) 的类，状态在 C++ 侧保留");
}
