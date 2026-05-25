hicc::cpp! {
    #include <string>
    #include <algorithm>
    #include <cctype>

    class String {
    public:
        std::string data;
        String() : data() {}
        explicit String(const char* str) : data(str ? str : "") {}
        explicit String(const char* str, unsigned long len) : data(str ? std::string(str, len) : "") {}
        ~String() { data.clear(); }
        unsigned long size() const { return data.size(); }
        unsigned long length() const { return data.length(); }
        bool empty() const { return data.empty(); }
        const char* c_str() const { return data.c_str(); }
        int compare(const char* other) const { return other ? data.compare(other) : 1; }
        bool equals(const char* other) const { return other ? data == other : data.empty(); }
        void append(const char* other) { if (other) data += other; }
        void clear() { data.clear(); }
        void to_upper() {
            std::transform(data.begin(), data.end(), data.begin(), ::toupper);
        }
        void to_lower() {
            std::transform(data.begin(), data.end(), data.begin(), ::tolower);
        }
    };

    String* string_new() { return new String(); }
    String* string_new_from(const char* str) { return new String(str); }
    void string_delete(String* self) { delete self; }
}

hicc::import_class! {
    #[cpp(class = "String")]
    class String {
        #[cpp(method = "unsigned long size() const")]
        fn size(&self) -> u64;

        #[cpp(method = "unsigned long length() const")]
        fn length(&self) -> u64;

        #[cpp(method = "bool empty() const")]
        fn empty(&self) -> bool;

        #[cpp(method = "const char* c_str() const")]
        fn c_str(&self) -> *const i8;

        #[cpp(method = "int compare(const char*) const")]
        fn compare(&self, other: *const i8) -> i32;

        #[cpp(method = "bool equals(const char*) const")]
        fn equals(&self, other: *const i8) -> bool;

        #[cpp(method = "void append(const char*)")]
        fn append(&mut self, other: *const i8);

        #[cpp(method = "void clear()")]
        fn clear(&mut self);

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
    fn string_new() -> *mut String;

    #[cpp(func = "String* string_new_from(const char* str)")]
    fn string_new_from(str: *const i8) -> *mut String;

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
