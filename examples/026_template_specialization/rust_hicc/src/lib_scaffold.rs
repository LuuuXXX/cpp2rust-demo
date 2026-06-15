hicc::cpp! {
    #include <iostream>
    #include <cstring>
    #include <cstdlib>
    #include <cstdio>

    #include "template_specialization.h"

    std::unique_ptr<IntHolder> _cpp2rust_make_unique_int_holder_with_value(int value) { return std::make_unique<IntHolder>(value); }
    std::unique_ptr<DoubleHolder> _cpp2rust_make_unique_double_holder_with_value(double value) { return std::make_unique<DoubleHolder>(value); }
    std::unique_ptr<StringHolder> _cpp2rust_make_unique_string_holder_with_value(const char* value) { return std::make_unique<StringHolder>(value); }
}

hicc::import_class! {
    #[cpp(class = "IntHolder")]
    pub class IntHolder {
        #[cpp(method = "int get() const")]
        pub fn get(&self) -> i32;

        #[cpp(method = "const char* describe() const")]
        pub fn describe(&self) -> *const i8;
    }
}

hicc::import_class! {
    #[cpp(class = "DoubleHolder")]
    pub class DoubleHolder {
        #[cpp(method = "double get() const")]
        pub fn get(&self) -> f64;

        #[cpp(method = "const char* describe() const")]
        pub fn describe(&self) -> *const i8;
    }
}

hicc::import_class! {
    #[cpp(class = "StringHolder")]
    pub class StringHolder {
        #[cpp(method = "const char* get() const")]
        pub fn get(&self) -> *const i8;

        #[cpp(method = "const char* describe() const")]
        pub fn describe(&self) -> *const i8;
    }
}

hicc::import_lib! {
    #![link_name = "template_specialization"]

    class IntHolder;
    class DoubleHolder;
    class StringHolder;

    #[cpp(func = "std::unique_ptr<IntHolder> _cpp2rust_make_unique_int_holder_with_value(int)")]
    pub fn int_holder_new_with_value(value: i32) -> IntHolder;

    #[cpp(func = "std::unique_ptr<DoubleHolder> _cpp2rust_make_unique_double_holder_with_value(double)")]
    pub fn double_holder_new_with_value(value: f64) -> DoubleHolder;

    #[cpp(func = "std::unique_ptr<StringHolder> _cpp2rust_make_unique_string_holder_with_value(const char*)")]
    pub unsafe fn string_holder_new_with_value(value: *const i8) -> StringHolder;
}
