use template_function::*;

fn main() {
    println!("=== 024_template_function - 函数模板 ===\n");

    // do_swap<int> 实例化
    let mut a = 10i32;
    let mut b = 20i32;
    println!("Before swap: a = {}, b = {}", a, b);
    unsafe {
        swap_i32(&mut a, &mut b);
    }
    println!("After swap:  a = {}, b = {}", a, b);

    println!();

    // do_swap<double> 实例化
    let mut x = 3.14f64;
    let mut y = 2.71f64;
    println!("Before swap: x = {}, y = {}", x, y);
    unsafe {
        swap_f64(&mut x, &mut y);
    }
    println!("After swap:  x = {}, y = {}", x, y);

    println!();

    // max_value<T> 实例化
    println!("max_i32(3, 7) = {}", max_i32(3, 7));
    println!("max_f64(2.5, 1.5) = {}", max_f64(2.5, 1.5));

    println!("\nRust FFI: 模板必须在 C++ 侧按具体类型实例化");
    println!("每个实例化（do_swap<int>、do_swap<double> …）是一个独立的具体函数");
    println!("anchor() = {}", template_function_anchor());
}
