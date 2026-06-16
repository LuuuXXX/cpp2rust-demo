//! 036_string_basic: std::string 基本操作（命名空间类直接持有字符串）。
//!
//! `MyString` 直接持有 `std::string`，演示 length/empty/append/at/c_str/compare/to_upper/find
//! 等基本操作。hicc 直出无需 extern-C 不透明指针 + `*_delete`，析构由 Rust `Drop` 自动完成。

hicc::cpp! {
    #include "string_basic.h"
}

hicc::import_class! {
    #[cpp(class = "string_basic_ns::MyString")]
    pub class MyString {
        #[cpp(method = "int length() const")]
        pub fn length(&self) -> i32;

        #[cpp(method = "int empty() const")]
        pub fn empty(&self) -> i32;

        #[cpp(method = "void append(const char* s)")]
        pub fn append(&mut self, s: *const i8);

        #[cpp(method = "char at(int i) const")]
        pub fn at(&self, i: i32) -> i8;

        #[cpp(method = "const char* c_str() const")]
        pub fn c_str(&self) -> *const i8;

        #[cpp(method = "int compare(const char* other) const")]
        pub fn compare(&self, other: *const i8) -> i32;

        #[cpp(method = "void to_upper()")]
        pub fn to_upper(&mut self);

        #[cpp(method = "int find(const char* sub) const")]
        pub fn find(&self, sub: *const i8) -> i32;

        pub fn new(s: *const i8) -> Self { my_string_new(s) }
    }
}

hicc::import_lib! {
    #![link_name = "string_basic"]

    #[cpp(func = "std::unique_ptr<string_basic_ns::MyString> hicc::make_unique<string_basic_ns::MyString, const char*>(const char*&&)")]
    pub fn my_string_new(s: *const i8) -> MyString;
}
