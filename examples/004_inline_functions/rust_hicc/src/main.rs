use inline_functions::*;

fn main() {
    // 内联函数在 Rust 端直接调用
    let a = 10;
    let b = 20;
    println!("min({}, {}) = {}", a, b, min(a, b));
    println!("max({}, {}) = {}", a, b, max(a, b));

    // 普通函数版本
    println!("min_v2({}, {}) = {}", a, b, min_v2(a, b));
    println!("max_v2({}, {}) = {}", a, b, max_v2(a, b));

    println!("\nRust FFI: Inline and normal functions work the same way!");
}
