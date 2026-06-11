use functional_bind::*;

fn main() {
    use std::ffi::CString;

    println!("=== 041_functional_bind - std::bind 绑定 ===\n");

    // Adder example - bound base value
    println!("--- Adder Demo (绑定基础值) ---");
    let mut adder = adder_new(100);
    println!("Result of adder.add(50): {}", adder.add(50));
    println!("Result of adder.add(30): {}", adder.add(30));

    // Multiplier example - bound multiplier
    println!("\n--- Multiplier Demo (绑定乘数) ---");
    let mut multiplier = multiplier_new(7);
    println!("multiply(6) = {}", multiplier.multiply(6));
    println!("multiply(11) = {}", multiplier.multiply(11));

    // StringProcessor example - bound member function and argument
    println!("\n--- StringProcessor Demo (成员函数绑定) ---");
    let mut processor = string_processor_new();
    processor.set_target(CString::new("hello world!").unwrap().as_ptr());

    println!("Count of 'l': {}", processor.count_char('l' as i8));
    println!("Count of 'o': {}", processor.count_char('o' as i8));
    println!("Count of 'h': {}", processor.count_char('h' as i8));

    println!("\n--- 总结 ---");
    println!("1. std::bind 创建部分应用的函数对象");
    println!("2. 可以绑定函数、成员函数、参数值");
    println!("3. 通过 opaque pointer 在 FFI 间传递绑定后的函数");
    println!("4. _1, _2 等占位符表示未绑定的参数位置");
}
