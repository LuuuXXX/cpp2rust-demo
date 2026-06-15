hicc::cpp! {
    #include <iostream>
    #include <string>
    #include <cstring>
    #include <algorithm>
    #include <cctype>

    #include "string_basic.h"

    std::unique_ptr<String> _cpp2rust_make_unique_string_0() { return std::make_unique<String>(); }
    std::unique_ptr<String> _cpp2rust_make_unique_string_with_str(const char* str) { return std::make_unique<String>(str); }
    std::unique_ptr<String> _cpp2rust_make_unique_string_2(const char* str, size_t len) { return std::make_unique<String>(str, len); }
}

hicc::import_class! {
    #[cpp(class = "String")]
    pub class String {
        #[cpp(method = "const char* c_str() const")]
        pub fn c_str(&self) -> *const i8;

        #[cpp(method = "size_t size() const")]
        pub fn size(&self) -> usize;

        #[cpp(method = "size_t length() const")]
        pub fn length(&self) -> usize;

        #[cpp(method = "bool empty() const")]
        pub fn empty(&self) -> bool;

        #[cpp(method = "int compare(const char* str) const")]
        pub fn compare(&self, str: *const i8) -> i32;

        #[cpp(method = "bool equals(const char* str) const")]
        pub fn equals(&self, str: *const i8) -> bool;

        #[cpp(method = "void append(const char* str)")]
        pub fn append(&mut self, str: *const i8);

        #[cpp(method = "void to_upper()")]
        pub fn to_upper(&mut self);

        #[cpp(method = "void to_lower()")]
        pub fn to_lower(&mut self);
    }
}

hicc::import_lib! {
    #![link_name = "string_basic"]

    class String;

    #[cpp(func = "std::unique_ptr<String> _cpp2rust_make_unique_string_0()")]
    pub fn string_new() -> String;

    #[cpp(func = "std::unique_ptr<String> _cpp2rust_make_unique_string_with_str(const char*)")]
    pub unsafe fn string_new_from(str: *const i8) -> String;

    #[cpp(func = "std::unique_ptr<String> _cpp2rust_make_unique_string_2(const char*, size_t)")]
    pub unsafe fn string_new_from_len(str: *const i8, len: usize) -> String;
}
