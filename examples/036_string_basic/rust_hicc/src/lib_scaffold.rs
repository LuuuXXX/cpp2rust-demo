hicc::cpp! {
    #include <iostream>
    #include <string>
    #include <cstring>
    #include <algorithm>
    #include <cctype>

    #include "string_basic.h"
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

    #[cpp(func = "std::unique_ptr<String> hicc::make_unique<String>()")]
    pub fn string_new() -> String;

    #[cpp(func = "std::unique_ptr<String> std::make_unique<String>(const char*)")]
    pub unsafe fn string_new_with_str(str: *const i8) -> String;

    #[cpp(func = "std::unique_ptr<String> std::make_unique<String>(const char*, size_t)")]
    pub unsafe fn string_new_2(str: *const i8, len: usize) -> String;
}
