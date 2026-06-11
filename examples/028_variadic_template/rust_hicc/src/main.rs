use variadic_template::*;

fn main() {
    println!("=== 028_variadic_template - 可变参数模板 ===\n");

    // 可变参数模板的 FFI 挑战
    // C++ 可变参数: template<typename... Args> int sum(Args... args)
    // FFI 无法直接传递...args

    // 解决方案：导出固定参数版本的函数

    let result0 = sum_zero();
    println!("Result: sum() = {}", result0);

    let result1 = sum_1(1);
    println!("Result: sum(1) = {}", result1);

    let result2 = sum_2(1, 2);
    println!("Result: sum(1, 2) = {}", result2);

    let result3 = sum_3(1, 2, 3);
    println!("Result: sum(1, 2, 3) = {}", result3);

    let result4 = sum_4(1, 2, 3, 4);
    println!("Result: sum(1, 2, 3, 4) = {}", result4);

    let result5 = sum_5(1, 2, 3, 4, 5);
    println!("Result: sum(1, 2, 3, 4, 5) = {}", result5);

    println!();

    let r2 = sum_double_2(1.5, 2.5);
    println!("Result: sum(1.5, 2.5) = {}", r2);

    let r3 = sum_double_3(1.1, 2.2, 3.3);
    println!("Result: sum(1.1, 2.2, 3.3) = {}", r3);

    println!("\nRust FFI: 可变参数模板的 FFI 挑战与解决方案");
    println!("挑战: C++ 可变参数模板(...Args) 无法直接映射到 FFI");
    println!("解决方案: 导出固定参数版本的函数");
    println!("每个参数数量 = 一个独立的函数");
}
