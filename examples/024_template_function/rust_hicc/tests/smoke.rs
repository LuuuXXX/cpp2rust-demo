//! 024_template_function 冒烟测试
//!
//! 验证生成的 Rust FFI 绑定可编译、可链接 C++ 实现，且基本行为正确。

use template_function::*;

#[test]
fn smoke_swap_int() {
    let mut a = 10i32;
    let mut b = 20i32;
    unsafe { swap_int(&mut a, &mut b) };
    assert_eq!(a, 20, "swap 后 a 应等于原 b");
    assert_eq!(b, 10, "swap 后 b 应等于原 a");
}

#[test]
fn smoke_swap_double() {
    let mut x = 3.14f64;
    let mut y = 2.71f64;
    unsafe { swap_double(&mut x, &mut y) };
    assert!((x - 2.71).abs() < 1e-10, "swap 后 x 应等于原 y");
    assert!((y - 3.14).abs() < 1e-10, "swap 后 y 应等于原 x");
}

#[test]
fn smoke_swap_char() {
    let mut c1 = b'A';
    let mut c2 = b'B';
    unsafe { swap_char(&mut c1, &mut c2) };
    assert_eq!(c1, b'B');
    assert_eq!(c2, b'A');
}

#[test]
fn smoke_array_get_set() {
    let mut arr = [10i32, 20, 30];
    unsafe { set_int_array(arr.as_mut_ptr(), 1, 99) };
    let v = unsafe { get_int_array(arr.as_mut_ptr(), 1) };
    assert_eq!(v, 99, "set 后 get 应返回相同值");
}

#[test]
fn smoke_swap_int_array() {
    let mut arr = [1i32, 2, 3, 4, 5];
    unsafe { swap_int_array(arr.as_mut_ptr(), 0, 4) };
    assert_eq!(arr[0], 5);
    assert_eq!(arr[4], 1);
}
