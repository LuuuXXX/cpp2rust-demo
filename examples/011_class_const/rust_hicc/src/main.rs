use class_const::*;

fn main() {
    let mut calc = calculator_new();

    println!("Initial value: {}", calc.get_value());
    println!("History count: {}", calc.get_history_count());

    calc.add(10);
    println!("After add(10): {}", calc.get_value());

    calc.add(5);
    println!("After add(5): {}", calc.get_value());

    calc.subtract(3);
    println!("After subtract(3): {}", calc.get_value());

    println!("History count: {}", calc.get_history_count());

    calc.clear();
    println!("After clear: {}", calc.get_value());
    println!("History count: {}", calc.get_history_count());

    println!("\nRust FFI: const member functions work!");
}
