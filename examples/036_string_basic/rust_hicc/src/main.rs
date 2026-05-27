hicc::cpp! {
    #include <stddef.h>
    #include <iostream>
    #include <string>
    #include <cstring>
    #include <algorithm>
    #include <cctype>

    class StringImpl {
    public:
        std::string data;
    public:
        StringImpl(const char* str, size_t len) : data(str ? std::string(str, len) : "") {
}
        StringImpl(const char* str) = default;
        StringImpl(const char* str, size_t len) = default;
        ~StringImpl() {
    data.clear();
}
    };

    struct String {
    public:
        StringImpl* impl;
        String(const char* str, size_t len) : impl(new StringImpl(str, len)) {
}
        String(const char* str) = default;
        String(const char* str, size_t len) = default;
        ~String() {
    delete impl;
    impl = nullptr;
}
    };

    String* string_new() {
        return new String();
    }

    String* string_new_from(const char* str) {
        return new String(str);
    }

    String* string_new_from_len(const char* str, size_t len) {
        return new String(str, len);
    }

    void string_delete(String* self) {
        delete self;
    }
}

hicc::import_lib! {
    #![link_name = "string_basic"]

    class String;

    #[cpp(func = "String* string_new()")]
    fn string_new() -> *mut String;

    #[cpp(func = "String* string_new_from(const char*)")]
    unsafe fn string_new_from(str: *const i8) -> *mut String;

    #[cpp(func = "String* string_new_from_len(const char*, size_t)")]
    unsafe fn string_new_from_len(str: *const i8, len: usize) -> *mut String;

    #[cpp(func = "void string_delete(String* self)")]
    unsafe fn string_delete(self_: *mut String);
}

fn main() {
    use std::ffi::CString;
    use std::ffi::CStr;

    println!("=== 036_string_basic - std::string ===\n");

    // Create string
    println!("--- Creation Demo ---");
    let mut s = string_new_from(CString::new("Hello").unwrap().as_ptr());
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
    let mut s = string_new_from(CString::new("Hello World").unwrap().as_ptr());
    s.to_upper();
    let c_str = unsafe { CStr::from_ptr(s.c_str()) };
    println!("To upper: {:?}", c_str);

    s.to_lower();
    let c_str = unsafe { CStr::from_ptr(s.c_str()) };
    println!("To lower: {:?}", c_str);

    unsafe { string_delete(&s); }

    println!("\nRust FFI: std::string 映射");
    println!("1. C++ 字符串映射为 opaque 指针");
    println!("2. 字符串内容通过 c_str() 获取");
    println!("3. 修改操作直接在原字符串上进行");
    println!("4. CString 用于 Rust 到 C 的转换");
}

