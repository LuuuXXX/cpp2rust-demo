use operator_overload::*;
use hicc::AbiClass;

fn main() {
    println!("=== Operator Overload FFI ===\n");
    println!("C++ operator overloading becomes named method calls in FFI\n");

    let a = number_new(10);
    let b = number_new(3);

    println!("Created numbers: a = {}, b = {}", number_getValue(&a.as_ptr()), number_getValue(&b.as_ptr()));
    println!();

    // Addition: a + b
    let sum = number_add(&a.as_ptr(), &b.as_ptr());
    println!("Result of a + b = {}", number_getValue(&sum.as_ptr()));

    // Subtraction: a - b
    let diff = number_sub(&a.as_ptr(), &b.as_ptr());
    println!("Result of a - b = {}", number_getValue(&diff.as_ptr()));

    // Multiplication: a * b
    let prod = number_mul(&a.as_ptr(), &b.as_ptr());
    println!("Result of a * b = {}", number_getValue(&prod.as_ptr()));

    // Division: a / b
    let quot = number_div(&a.as_ptr(), &b.as_ptr());
    println!("Result of a / b = {}", number_getValue(&quot.as_ptr()));

    println!();

    // Unary operators
    println!("Unary operators:");
    let neg = number_negate(&a.as_ptr());
    println!("Negation of a = {}", number_getValue(&neg.as_ptr()));

    // Comparison
    let cmp = number_compare(&a.as_ptr(), &b.as_ptr());
    println!("a compared to b = {}", cmp);

    println!();
    println!("Rust FFI: Operators become named methods");
    println!("a + b -> number_add(a, b)");
    println!("a - b -> number_sub(a, b)");
    println!("a * b -> number_mul(a, b)");

}
