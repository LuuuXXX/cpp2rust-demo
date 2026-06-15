use inheritance_multiple::*;

fn main() {
    let derived = derived_new_3(10, 20, 30);

    println!("Base1 value: {}", derived.get_value1());
    println!("Base2 value: {}", derived.get_value2());
    println!("Derived value: {}", derived.get_derived_value());

    derived.compute();

    println!("\nRust FFI: Multiple inheritance with hicc pattern");
}
