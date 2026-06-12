use function_overload::*;

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
