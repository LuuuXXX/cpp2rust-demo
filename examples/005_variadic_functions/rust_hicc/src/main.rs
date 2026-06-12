use variadic_functions::*;

fn main() {
    println!("=== 005_variadic_functions - 可变参数函数 ===\n");

    println!("--- sum (via wrapper) ---");
    println!("sum_3(1, 2, 3) = {}", sum_3(1, 2, 3));
    println!("sum_5(1, 2, 3, 4, 5) = {}", sum_5(1, 2, 3, 4, 5));

    println!("\nRust FFI: C 可变参数函数通过固定参数包装函数调用");
}
