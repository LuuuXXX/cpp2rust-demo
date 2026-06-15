use hicc::AbiClass;
use friend_function::*;

fn main() {
    println!("=== Friend Function FFI ===\n");
    println!("Friend functions in C++ can access private members of a class\n");

    let a = myclass_new_with_v(10);
    let b = myclass_new_with_v(20);

    println!("Created MyClass objects:");
    println!("  a.value = {}", a.get_value());
    println!("  b.value = {}", b.get_value());
    println!();

    println!("Friend function operations:");
    let sum = friend_function_get_sum(&a.as_ptr(), &b.as_ptr());
    println!("  Sum: {}", sum);

    let product = friend_function_get_product(&a.as_ptr(), &b.as_ptr());
    println!("  Product: {}", product);

    let cmp = friend_function_compare(&a.as_ptr(), &b.as_ptr());
    println!("  Compare: {}", cmp);

    println!();
    println!("Rust FFI: Friend functions are just regular functions");
    println!("The 'friend' relationship is a C++ access control concept");
}
