hicc::cpp! {
    #include "constexpr_basic.h"
}

hicc::import_lib! {
    #![link_name = "constexpr_basic"]

    #[cpp(func = "int get_fibonacci_10()")]
    fn get_fibonacci_10() -> i32;

    #[cpp(func = "int manhattan_distance(int, int)")]
    fn manhattan_distance(x: i32, y: i32) -> i32;

    #[cpp(func = "int constexpr_sum_array(const int*, int)")]
    fn constexpr_sum_array(arr: *const i32, size: i32) -> i32;

    #[cpp(func = "int constexpr_find_max(const int*, int)")]
    fn constexpr_find_max(arr: *const i32, size: i32) -> i32;

    #[cpp(func = "int get_array_size()")]
    fn get_array_size() -> i32;
}

fn main() {
    println!("=== 046_constexpr_basic - constexpr ===\n");

    // Compile-time computed fibonacci number
    println!("--- Compile-time Fibonacci ---");
    let fib_10 = get_fibonacci_10();
    println!("fibonacci<10>() = {} (computed at compile time)", fib_10);
    println!("Rust equivalent: fib(10) = {} (also compile time)", FIB_RUST);

    // Runtime manhattan distance
    println!("\n--- Runtime Manhattan Distance ---");
    println!("manhattan_distance(3, 4) = {}", manhattan_distance(3, 4));
    println!("manhattan_distance(-3, -4) = {}", manhattan_distance(-3, -4));
    println!("manhattan_distance(10, -5) = {}", manhattan_distance(10, -5));

    // Array operations
    println!("\n--- Array Operations ---");
    let arr = [1, 5, 3, 9, 2, 8, 4, 7, 6, 0];
    let size = get_array_size();
    println!("Array: {:?}", &arr[..size as usize]);

    let sum = constexpr_sum_array(arr.as_ptr(), size);
    println!("Sum: {}", sum);

    let max = constexpr_find_max(arr.as_ptr(), size);
    println!("Max: {}", max);

    println!("\n--- Summary ---");
    println!("1. constexpr specifies expression computed at compile time");
    println!("2. constexpr functions must satisfy compile-time evaluation conditions");
    println!("3. constexpr variables have determined values at compile time");
    println!("4. FFI constexpr values passed via preprocessor macros");
    println!("5. Rust const fn can achieve similar functionality");
}



