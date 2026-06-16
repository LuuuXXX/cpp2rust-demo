use array_basic::*;

fn main() {
    println!("=== 037_array_basic - std::array（hicc 直出）===\n");

    let mut a = IntArray::new();
    println!("size={} sum={}", a.size(), a.sum());
    for i in 0..a.size() {
        a.set(i, i * 10);
    }
    println!("after set sum={} min={} max={}", a.sum(), a.min(), a.max());
    a.set(2, 999);
    println!("get(2)={} get(99)={}", a.get(2), a.get(99));
    a.fill(7);
    println!("after fill sum={} min={} max={}", a.sum(), a.min(), a.max());

    println!("\nRust FFI: hicc 直接绑定持有 std::array 的类，析构由 Rust Drop 自动完成");
}
