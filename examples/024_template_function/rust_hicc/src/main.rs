hicc::cpp! {
    #include "template_function.h"
}

hicc::import_lib! {
    #![link_name = "template_function"]

    #[cpp(func = "void swap_int(int*, int*)")]
    unsafe fn swap_int(a: *mut i32, b: *mut i32);

    #[cpp(func = "void swap_double(double*, double*)")]
    unsafe fn swap_double(a: *mut f64, b: *mut f64);

    #[cpp(func = "void swap_char(char*, char*)")]
    unsafe fn swap_char(a: *mut i8, b: *mut i8);

    #[cpp(func = "void swap_int_array(int*, int, int)")]
    unsafe fn swap_int_array(arr: *mut i32, i: i32, j: i32);

    #[cpp(func = "int get_int_array(int*, int)")]
    unsafe fn get_int_array(arr: *mut i32, idx: i32) -> i32;

    #[cpp(func = "void set_int_array(int*, int, int)")]
    unsafe fn set_int_array(arr: *mut i32, idx: i32, value: i32);
}

fn main() {
    println!("=== 024_template_function - 函数模板 ===\n");

    // swap int
    let mut a = 10i32;
    let mut b = 20i32;
    println!("Before swap: a = {}, b = {}", a, b);
    swap_int(&mut a, &mut b);
    println!("After swap: a = {}, b = {}", a, b);

    println!();

    // swap double
    let mut x = 3.14f64;
    let mut y = 2.71f64;
    println!("Before swap: x = {}, y = {}", x, y);
    swap_double(&mut x, &mut y);
    println!("After swap: x = {}, y = {}", x, y);

    println!();

    // swap char
    let mut c1 = b'A' as i8;
    let mut c2 = b'B' as i8;
    println!("Before swap: c1 = {}, c2 = {}", c1 as u8 as char, c2 as u8 as char);
    swap_char(&mut c1, &mut c2);
    println!("After swap: c1 = {}, c2 = {}", c1 as u8 as char, c2 as u8 as char);

    println!();

    // swap in array
    let mut arr = [1i32, 2, 3, 4, 5];
    println!("Array before swap(0,4): {:?}", arr);
    swap_int_array(arr.as_mut_ptr(), 0, 4);
    println!("Array after swap(0,4): {:?}", arr);

    println!("\nRust FFI: 模板必须在 C++ 侧实例化");
    println!("每个模板实例 = 一个独立的 C 函数");
    println!("swap_int, swap_double, swap_char 是三个不同的函数");
}

