use string_basic::*;

fn decode_cstr(ptr: *const i8) -> String {
    unsafe { std::ffi::CStr::from_ptr(ptr).to_string_lossy().into_owned() }
}

fn main() {
    use std::ffi::CString;

    println!("=== 036_string_basic - std::string ===\n");

    // Create string
    println!("--- Creation Demo ---");
    let mut s = unsafe { string_new_from(CString::new("Hello").unwrap().as_ptr()) };
    println!("Created: {}", decode_cstr(s.c_str()));
    println!("Size: {}, Length: {}", s.size(), s.length());
    println!("Empty: {}", s.empty());

    // Comparison
    println!("\n--- Comparison Demo ---");
    let cmp = s.compare(CString::new("Hello").unwrap().as_ptr());
    println!("Compare with 'Hello': {}", cmp);

    let eq = s.equals(CString::new("Hello").unwrap().as_ptr());
    println!("Equals 'Hello': {}", eq);

    // Concatenation
    println!("\n--- Concatenation Demo ---");
    s.append(CString::new(", World!").unwrap().as_ptr());
    println!("After append: {}", decode_cstr(s.c_str()));

    // Case conversion
    println!("\n--- Case Conversion Demo ---");
    let mut s = unsafe { string_new_from(CString::new("Hello World").unwrap().as_ptr()) };
    s.to_upper();
    println!("To upper: {}", decode_cstr(s.c_str()));

    s.to_lower();
    println!("To lower: {}", decode_cstr(s.c_str()));

    println!("\nRust FFI: std::string 映射");
    println!("1. C++ 字符串映射为 opaque 指针");
    println!("2. 字符串内容通过 c_str() 获取");
    println!("3. 修改操作直接在原字符串上进行");
    println!("4. CString 用于 Rust 到 C 的转换");
}
