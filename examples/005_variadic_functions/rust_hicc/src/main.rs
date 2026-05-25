#[link(name = "variadic_functions")]
extern "C" {
    fn sum_3(a: i32, b: i32, c: i32) -> i32;
    fn sum_5(a: i32, b: i32, c: i32, d: i32, e: i32) -> i32;
}

fn main() {
    // sum(3, 1, 2, 3) = 6
    let result = unsafe { sum_3(1, 2, 3) };
    println!("sum_3(1, 2, 3) = {}", result);

    // sum(5, 10, 20, 30, 40, 50) = 150
    let result = unsafe { sum_5(10, 20, 30, 40, 50) };
    println!("sum_5(10, 20, 30, 40, 50) = {}", result);

    println!("\nRust FFI: Variadic functions handled via wrapper!");
    println!("Note: C variadic functions (va_list) require wrapper functions for FFI");
}
