use string_basic::*;

fn main() {
    use std::ffi::CString;
    use std::ffi::CStr;

    println!("=== 036_string_basic - std::string ===\n");

    // Create string
    println!("--- Creation Demo ---");
    let mut s = unsafe { string_new_from(CString::new("Hello").unwrap().as_ptr()) };
    let c_str = unsafe { CStr::from_ptr(s.c_str()) };
    println!("Created: {:?}", c_str);
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
    let c_str = unsafe { CStr::from_ptr(s.c_str()) };
    println!("After append: {:?}", c_str);

    // Case conversion
    println!("\n--- Case Conversion Demo ---");
    let mut s = unsafe { string_new_from(CString::new("Hello World").unwrap().as_ptr()) };
    s.to_upper();
    let c_str = unsafe { CStr::from_ptr(s.c_str()) };
    println!("To upper: {:?}", c_str);

    s.to_lower();
    let c_str = unsafe { CStr::from_ptr(s.c_str()) };
    println!("To lower: {:?}", c_str);

    println!("\nRust FFI: std::string 映射");
    println!("1. C++ 字符串映射为 opaque 指针");
    println!("2. 字符串内容通过 c_str() 获取");
    println!("3. 修改操作直接在原字符串上进行");
    println!("4. CString 用于 Rust 到 C 的转换");
}

