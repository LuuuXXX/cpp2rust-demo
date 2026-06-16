use std_function::*;

fn main() {
    println!("=== 040_std_function - std::function（hicc 直出）===\n");

    let dbl = Callback::new(0);
    let tri = Callback::new(1);
    let neg = Callback::new(2);
    println!("double(5)={}", dbl.invoke(5));
    println!("triple(5)={}", tri.invoke(5));
    println!("negate(5)={}", neg.invoke(5));

    println!();

    let mut p = Pipeline::new();
    p.add(0);
    p.add(1);
    println!("pipeline size={} run(2)={}", p.size(), p.run(2));

    println!("\nRust FFI: hicc 绑定内部持有 std::function 的类，回调状态在 C++ 侧保留");
}
