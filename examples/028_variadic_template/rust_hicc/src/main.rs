use variadic_template::*;

fn main() {
    println!("=== 028_variadic_template - 可变参数模板 ===\n");

    println!("sum_i32_0() = {}", sum_i32_0());
    println!("sum_i32_2(10, 20) = {}", sum_i32_2(10, 20));
    println!("sum_i32_3(1, 2, 3) = {}", sum_i32_3(1, 2, 3));
    println!("sum_i32_5(1, 2, 3, 4, 5) = {}", sum_i32_5(1, 2, 3, 4, 5));

    println!();

    println!("sum_f64_2(1.5, 2.5) = {}", sum_f64_2(1.5, 2.5));
    println!("sum_f64_3(1.5, 2.5, 3.0) = {}", sum_f64_3(1.5, 2.5, 3.0));

    println!("\nRust FFI: 可变参数模板按「实参个数 + 类型」逐一实例化");
    println!("每个组合（sum<int,int>、sum<double,double,double> …）是一个独立实例");
    println!("anchor() = {}", variadic_template_anchor());
}
