//! 024_template_function 冒烟测试
//!
//! 验证生成的 Rust FFI 绑定可编译、可链接 C++ 模板实例化，且基本行为正确。

use template_function::*;

#[test]
fn smoke_swap_i32() {
    let mut a = 10i32;
    let mut b = 20i32;
    unsafe {
        swap_i32(&mut a, &mut b);
    }
    assert_eq!(a, 20, "swap 后 a 应等于原 b");
    assert_eq!(b, 10, "swap 后 b 应等于原 a");
}

#[test]
fn smoke_swap_f64() {
    let mut x = 3.14f64;
    let mut y = 2.71f64;
    unsafe {
        swap_f64(&mut x, &mut y);
    }
    assert!((x - 2.71).abs() < 1e-10, "swap 后 x 应等于原 y");
    assert!((y - 3.14).abs() < 1e-10, "swap 后 y 应等于原 x");
}

#[test]
fn smoke_max_value() {
    assert_eq!(max_i32(3, 7), 7);
    assert_eq!(max_i32(42, 1), 42);
    assert!((max_f64(2.5, 1.5) - 2.5).abs() < 1e-10);
}

#[test]
fn smoke_anchor() {
    assert_eq!(template_function_anchor(), 0);
}
