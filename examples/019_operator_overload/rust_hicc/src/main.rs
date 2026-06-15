use operator_overload::*;

fn main() {
    println!("=== Operator Overload FFI ===\n");
    println!("C++ operator overloading becomes named method calls in FFI\n");

    let a = number_new_with_v(10);
    let b = number_new_with_v(3);

    println!("Created numbers: a = {}, b = {}", a.get_value(), b.get_value());
    println!();

    println!("Direct mode methods:");
    println!("  a.getValue() = {}", a.get_value());
    println!("  b.getValue() = {}", b.get_value());

    let cmp = a.compare(&b);
    println!("  a.compare(&b) = {} (positive = a > b)", cmp);

    println!();
    println!("Manual arithmetic using getValue():");
    let va = a.get_value();
    let vb = b.get_value();
    println!("  a + b = {}", va + vb);
    println!("  a - b = {}", va - vb);
    println!("  a * b = {}", va * vb);
    println!("  a / b = {}", va / vb);
    println!("  -a   = {}", -va);

    println!();
    println!("Rust FFI: 运算符重载在 Direct 模式中只有 getValue + compare");
    println!("算术运算需在 Rust 侧用 getValue() 手动实现");
}
