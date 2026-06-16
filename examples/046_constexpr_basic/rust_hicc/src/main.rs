use constexpr_basic::*;

fn main() {
    println!("=== 046_constexpr_basic - constexpr（hicc 直出）===\n");

    let p = ConstexprPoint::new(3, 4);
    println!(
        "p x={} y={} manhattan={}",
        p.x(),
        p.y(),
        p.manhattan_distance()
    );

    let neg = ConstexprPoint::new(-2, -5);
    println!("neg manhattan={}", neg.manhattan_distance());
    println!("fibonacci<10>()={} array_size={}", fibonacci_10(), array_size());

    println!("\nRust FFI: hicc 直接绑定 constexpr 类与命名空间自由函数，析构由 Rust Drop 自动完成");
}
