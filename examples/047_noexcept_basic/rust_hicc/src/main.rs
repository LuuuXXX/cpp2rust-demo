use noexcept_basic::*;

fn main() {
    println!("=== 047_noexcept_basic - noexcept（hicc 直出）===\n");

    println!("noexcept_add(2,3)={}", noexcept_add(2, 3));
    println!("noexcept_multiply(4,5)={}", noexcept_multiply(4, 5));
    println!(
        "conditional_abs(-7)={} conditional_abs(7)={}",
        conditional_abs(-7),
        conditional_abs(7)
    );
    println!(
        "safe_divide(10,2)={} safe_divide(10,0)={}",
        safe_divide(10, 2),
        safe_divide(10, 0)
    );

    let mover = NoexceptMover::new(42);
    println!("mover value={}", mover.get_value());

    println!("\nRust FFI: hicc 直接绑定 noexcept 命名空间函数与 move-only 类，析构由 Rust Drop 自动完成");
}
