hicc::cpp! {
    #include <stddef.h>
    #include <iostream>
    #include <string>
    #include <cstring>
    #include <algorithm>
    #include <cctype>

    #include "string_basic.h"
}

hicc::import_class! {
    #[cpp(class = "String", destroy = "string_delete")]
    pub class String {
        #[cpp(method = "const char* c_str() const")]
        fn c_str(&self) -> *const i8;

        #[cpp(method = "size_t size() const")]
        fn size(&self) -> usize;

        #[cpp(method = "size_t length() const")]
        fn length(&self) -> usize;

        #[cpp(method = "bool empty() const")]
        fn empty(&self) -> bool;

        #[cpp(method = "int compare(const char* str) const")]
        fn compare(&self, str: *const i8) -> i32;

        #[cpp(method = "bool equals(const char* str) const")]
        fn equals(&self, str: *const i8) -> bool;

        #[cpp(method = "void append(const char* str)")]
        fn append(&mut self, str: *const i8);

        #[cpp(method = "void to_upper()")]
        fn to_upper(&mut self);

        #[cpp(method = "void to_lower()")]
        fn to_lower(&mut self);
    }
}

hicc::import_lib! {
    #![link_name = "string_basic"]

    class String;

    #[cpp(func = "String* string_new()")]
    fn string_new() -> String;

    #[cpp(func = "String* string_new_from(const char*)")]
    unsafe fn string_new_from(str: *const i8) -> String;

    #[cpp(func = "String* string_new_from_len(const char*, size_t)")]
    unsafe fn string_new_from_len(str: *const i8, len: usize) -> String;
}

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

