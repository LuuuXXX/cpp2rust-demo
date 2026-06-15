use functional_bind::*;
use std::ffi::CString;

fn main() {
    println!("=== 041_functional_bind - std::bind ===\n");

    println!("--- Adder Demo ---");
    let mut adder = adder_new(100);
    println!("Result of adder.add(50): {}", adder.add(50));
    println!("Result of adder.add(30): {}", adder.add(30));

    println!("\n--- Multiplier Demo ---");
    let mut multiplier = multiplier_new(7);
    println!("multiply(6) = {}", multiplier.multiply(6));
    println!("multiply(11) = {}", multiplier.multiply(11));

    println!("\n--- StringProcessor Demo ---");
    let mut processor = string_processor_new();
    processor.set_target(CString::new("hello world!").unwrap().as_ptr());

    println!("Count of 'l': {}", processor.count_char('l' as i8));
    println!("Count of 'o': {}", processor.count_char('o' as i8));
    println!("Count of 'h': {}", processor.count_char('h' as i8));

    println!("\n--- Summary ---");
    println!("1. std::bind creates partially-applied function objects");
    println!("2. Can bind functions, member functions, and argument values");
    println!("3. Passed across FFI via opaque pointer");
    println!("4. _1, _2 placeholders represent unbound parameter positions");
}
