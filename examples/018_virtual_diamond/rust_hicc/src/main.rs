use hicc::AbiClass;
use virtual_diamond::*;

fn main() {
    println!("=== Diamond Inheritance FFI with hicc ===\n");
    println!("Diamond inheritance structure:");
    println!("       A");
    println!("      / \\");
    println!("     B   C");
    println!("      \\ /");
    println!("       D");
    println!();
    println!("Virtual inheritance ensures only ONE A subobject in D\n");

    let mut d = d_new(1, 2, 3, 4);

    println!("Values:");
    println!("  A value (via B): {}", d_get_a_value(&d.as_mut_ptr()));
    println!("  B value: {}", d.get_b_value());
    println!("  C value: {}", d.get_c_value());
    println!("  D value: {}", d.get_d_value());

    println!();
    d.compute();

    println!("\nRust FFI: Diamond inheritance works correctly with hicc!");
}

