hicc::cpp! {
    #include "inline_functions.h"
}

hicc::import_lib! {
    #![link_name = "inline_functions"]

    #[cpp(func = "int min(int, int)")]
    fn min(a: i32, b: i32) -> i32;

    #[cpp(func = "int max(int, int)")]
    fn max(a: i32, b: i32) -> i32;

    #[cpp(func = "int min_v2(int, int)")]
    fn min_v2(a: i32, b: i32) -> i32;

    #[cpp(func = "int max_v2(int, int)")]
    fn max_v2(a: i32, b: i32) -> i32;
}

fn main() {
    // 内联函数在 Rust 端直接调用
    let a = 10;
    let b = 20;
    println!("min({}, {}) = {}", a, b, min_val(a, b));
    println!("max({}, {}) = {}", a, b, max_val(a, b));

    // 普通函数版本
    println!("min_v2({}, {}) = {}", a, b, min_v2(a, b));
    println!("max_v2({}, {}) = {}", a, b, max_v2(a, b));

    println!("\nRust FFI: Inline and normal functions work the same way!");
}



