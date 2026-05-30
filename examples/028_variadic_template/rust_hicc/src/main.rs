hicc::cpp! {
    #include <iostream>
    #include <cstdarg>

    #include "variadic_template.h"
}

hicc::import_lib! {
    #![link_name = "variadic_template"]

    #[cpp(func = "int sum_zero()")]
    fn sum_zero() -> i32;

    #[cpp(func = "int sum_1(int)")]
    fn sum_1(a: i32) -> i32;

    #[cpp(func = "int sum_2(int, int)")]
    fn sum_2(a: i32, b: i32) -> i32;

    #[cpp(func = "int sum_3(int, int, int)")]
    fn sum_3(a: i32, b: i32, c: i32) -> i32;

    #[cpp(func = "int sum_4(int, int, int, int)")]
    fn sum_4(a: i32, b: i32, c: i32, d: i32) -> i32;

    #[cpp(func = "int sum_5(int, int, int, int, int)")]
    fn sum_5(a: i32, b: i32, c: i32, d: i32, e: i32) -> i32;

    #[cpp(func = "double sum_double_2(double, double)")]
    fn sum_double_2(a: f64, b: f64) -> f64;

    #[cpp(func = "double sum_double_3(double, double, double)")]
    fn sum_double_3(a: f64, b: f64, c: f64) -> f64;

    #[cpp(func = "double sum_double_4(double, double, double, double)")]
    fn sum_double_4(a: f64, b: f64, c: f64, d: f64) -> f64;

    #[cpp(func = "const char* sum_getFormat(int)")]
    unsafe fn sum_get_format(count: i32) -> *const i8;
}

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


