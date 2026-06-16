use summary::*;

fn main() {
    println!("=== 048_summary - 示例系列汇总（hicc 直出）===\n");

    let mut counter = Counter::new();
    println!("initial={}", counter.get());
    counter.increment();
    counter.increment();
    counter.increment();
    println!("after increment x3={}", counter.get());
    counter.decrement();
    println!("after decrement={}", counter.get());
    counter.reset();
    println!("after reset={}", counter.get());

    println!("safe_add(2,3)={}", safe_add(2, 3));
    println!("max_size()={}", max_size());

    println!("\nRust FFI: hicc 直接绑定命名空间类与自由函数，无需 extern-C shim");
}
