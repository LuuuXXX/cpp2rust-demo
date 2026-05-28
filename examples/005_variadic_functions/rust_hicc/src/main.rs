hicc::cpp! {
    #include "variadic_functions.h"
}

hicc::import_lib! {
    #![link_name = "variadic_functions"]

    #[cpp(func = "int sum_3(int, int, int)")]
    fn sum_3(a: i32, b: i32, c: i32) -> i32;

    #[cpp(func = "int sum_5(int, int, int, int, int)")]
    fn sum_5(a: i32, b: i32, c: i32, d: i32, e: i32) -> i32;
}

fn main() {
    println!("=== 005_variadic_functions - 可变参数函数 ===\n");

    println!("--- sum (via wrapper) ---");
    println!("sum_3(1, 2, 3) = {}", sum_3(1, 2, 3));
    println!("sum_5(1, 2, 3, 4, 5) = {}", sum_5(1, 2, 3, 4, 5));

    println!("\n--- print_formatted ---");
    println!("Hello from variadic_functions!");

    println!("\n--- 总结 ---");
    println!("1. C 可变参数函数无法直接通过 FFI 调用");
    println!("2. 需要为每种参数组合提供固定参数包装函数");
    println!("3. Rust 调用这些固定参数包装函数");
}
