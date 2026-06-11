use friend_function::*;

fn main() {
    println!("=== Friend Function FFI ===\n");
    println!("Friend functions in C++ can access private members of a class\n");

    let a = myclass_new(10);
    let b = myclass_new(20);
    use hicc::AbiClass;

    println!("Created MyClass objects:");
    println!("  a.value = {}", a.get_value());
    println!("  b.value = {}", b.get_value());
    println!();

    // Friend functions: can access private members
    println!("Friend function operations:");
    let sum = friend_function_get_sum(&a.as_ptr(), &b.as_ptr());
    println!("  Sum: {}", sum);

    let product = friend_function_get_product(&a.as_ptr(), &b.as_ptr());
    println!("  Product: {}", product);

    let cmp = friend_function_compare(&a.as_ptr(), &b.as_ptr());
    println!("  Compare: {}", cmp);

    println!();
    println!("Rust FFI: Friend functions are just regular functions");
    println!("In C FFI, we can access struct members directly");
    println!("The 'friend' relationship is a C++ access control concept");

}
