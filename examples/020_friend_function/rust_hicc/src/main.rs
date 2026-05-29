hicc::cpp! {
    #include <iostream>

    #include "friend_function.h"
}

hicc::import_class! {
    #[cpp(class = "MyClass", destroy = "myclass_delete")]
    class MyClass {
        #[cpp(method = "int getValue() const")]
        fn get_value(&self) -> i32;

        #[cpp(method = "void setValue(int v)")]
        fn set_value(&mut self, v: i32);
    }
}

hicc::import_lib! {
    #![link_name = "friend_function"]

    class MyClass;

    #[cpp(func = "MyClass* myclass_new(int)")]
    fn myclass_new(secret_value: i32) -> MyClass;

    #[cpp(func = "int friend_function_getSum(const MyClass* a, const MyClass* b)")]
    fn friend_function_get_sum(a: *const MyClass, b: *const MyClass) -> i32;

    #[cpp(func = "int friend_function_getProduct(const MyClass* a, const MyClass* b)")]
    fn friend_function_get_product(a: *const MyClass, b: *const MyClass) -> i32;

    #[cpp(func = "int friend_function_compare(const MyClass* a, const MyClass* b)")]
    fn friend_function_compare(a: *const MyClass, b: *const MyClass) -> i32;
}

fn main() {
    println!("=== Friend Function FFI ===\n");
    println!("Friend functions in C++ can access private members of a class\n");

    let a = myclass_new(10);
    let b = myclass_new(20);

    println!("Created MyClass objects:");
    println!("  a.value = {}", a.get_value());
    println!("  b.value = {}", b.get_value());
    println!();

    // Friend functions: can access private members
    println!("Friend function operations:");
    let sum = friend_function_get_sum(&a.as_ref().as_ptr(), &b.as_ref().as_ptr());
    println!("  Sum: {}", sum);

    let product = friend_function_get_product(&a.as_ref().as_ptr(), &b.as_ref().as_ptr());
    println!("  Product: {}", product);

    let cmp = friend_function_compare(&a.as_ref().as_ptr(), &b.as_ref().as_ptr());
    println!("  Compare: {}", cmp);

    println!();
    println!("Rust FFI: Friend functions are just regular functions");
    println!("In C FFI, we can access struct members directly");
    println!("The 'friend' relationship is a C++ access control concept");

}

