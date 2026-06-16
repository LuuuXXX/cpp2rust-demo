use lambda_basic::*;

fn main() {
    println!("=== 039_lambda_basic - lambda 表达式（hicc 直出）===\n");

    let add = Operation::new(0);
    let mul = Operation::new(1);
    let mx = Operation::new(2);
    println!("add(3,4)={}", add.apply(3, 4));
    println!("mul(3,4)={}", mul.apply(3, 4));
    println!("max(3,4)={}", mx.apply(3, 4));

    println!();

    let mut acc = Accumulator::new(10);
    let a = acc.apply(5);
    let b = acc.apply(3);
    println!("acc.apply(5)={} apply(3)={} value={}", a, b, acc.value());

    println!("\nRust FFI: hicc 绑定内部持有 lambda(std::function) 的类，闭包状态在 C++ 侧保留");
}
