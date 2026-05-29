hicc::cpp! {
    #include "function_overload.h"
}

hicc::import_lib! {
    #![link_name = "function_overload"]

    #[cpp(func = "int add_int(int, int)")]
    fn add_int(a: i32, b: i32) -> i32;

    #[cpp(func = "double add_double(double, double)")]
    fn add_double(a: f64, b: f64) -> f64;

    #[cpp(func = "const char* add_strings(const char*, const char*)")]
    unsafe fn add_strings(a: *const i8, b: *const i8) -> *const i8;

    #[cpp(func = "int sum3(int, int, int)")]
    fn sum3(a: i32, b: i32, c: i32) -> i32;
}

fn main() {
    use std::ffi::CStr;
    let sum = add_int(1, 2);
    println!("add_int result: {}", sum);

    let sum = add_double(1.5, 2.5);
    println!("add_double result: {}", sum);

    let result = unsafe {
        let a = b"Hello\0".as_ptr() as *const i8;
        let b = b" World\0".as_ptr() as *const i8;
        let ptr = add_strings(a, b);
        CStr::from_ptr(ptr).to_string_lossy().into_owned()
    };
    println!("add_strings result: {}", result);

    let sum = sum3(1, 2, 3);
    println!("sum3 result: {}", sum);

    println!("\nRust FFI: All overloads called successfully!");
}



