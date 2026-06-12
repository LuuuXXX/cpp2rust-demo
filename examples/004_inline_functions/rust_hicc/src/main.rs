use inline_functions::*;

fn main() {
    let a = 10;
    let b = 20;
    println!("min({}, {}) = {}", a, b, min(a, b));
    println!("max({}, {}) = {}", a, b, max(a, b));

    println!("min_v2({}, {}) = {}", a, b, min_v2(a, b));
    println!("max_v2({}, {}) = {}", a, b, max_v2(a, b));

    println!("\nRust FFI: Inline and normal functions work the same way!");
}
