use explicit_ctor::*;

fn main() {
    println!("=== 021_explicit_ctor - explicit 构造函数 ===\n");
    println!("C++ explicit 关键字防止隐式类型转换\n");

    // Implicit constructor
    let w1 = widget_new(42);
    println!("Created with implicit ctor: value = {}", w1.get_value());

    println!();

    // Explicit constructor - must be called explicitly
    let w2 = widget_from_int(100);
    println!("Created with explicit int ctor: value = {}", w2.get_value());

    let w3 = widget_from_double(3.14);
    println!("Created with explicit double ctor: value = {}", w3.get_value());

    println!("\nRust FFI: explicit 不影响 FFI - 只是禁止隐式转换");
    println!("在 FFI 中，所有构造函数都是显式调用的");
}
