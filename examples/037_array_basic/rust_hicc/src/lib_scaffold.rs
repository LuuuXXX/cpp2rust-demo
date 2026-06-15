hicc::cpp! {
    #include <iostream>
    #include <array>
    #include <string>
    #include <cstring>

    #include "array_basic.h"

    std::unique_ptr<IntArray5> _cpp2rust_make_unique_int_array5_with_values(const int* values) { return std::make_unique<IntArray5>(values); }
    std::unique_ptr<DoubleArray3> _cpp2rust_make_unique_double_array3_with_values(const double* values) { return std::make_unique<DoubleArray3>(values); }
}

hicc::import_class! {
    #[cpp(class = "IntArray5")]
    pub class IntArray5 {
        #[cpp(method = "size_t size() const")]
        pub fn size(&self) -> usize;

        #[cpp(method = "bool empty() const")]
        pub fn empty(&self) -> bool;

        #[cpp(method = "void set(size_t i, int val)")]
        pub fn set(&mut self, i: usize, val: i32);

        #[cpp(method = "int get(size_t i) const")]
        pub fn get(&self, i: usize) -> i32;

        #[cpp(method = "int at(size_t i) const")]
        pub fn at(&self, i: usize) -> i32;

        #[cpp(method = "int* data()")]
        pub fn data(&mut self) -> *mut i32;
    }
}

hicc::import_class! {
    #[cpp(class = "DoubleArray3")]
    pub class DoubleArray3 {
        #[cpp(method = "size_t size() const")]
        pub fn size(&self) -> usize;
    }
}

hicc::import_class! {
    #[cpp(class = "StringArray4")]
    pub class StringArray4 {
        #[cpp(method = "size_t size() const")]
        pub fn size(&self) -> usize;
    }
}

hicc::import_lib! {
    #![link_name = "array_basic"]

    class IntArray5;
    class DoubleArray3;
    class StringArray4;

    #[cpp(func = "std::unique_ptr<IntArray5> hicc::make_unique<IntArray5>()")]
    pub fn int_array5_new() -> IntArray5;

    #[cpp(func = "std::unique_ptr<IntArray5> _cpp2rust_make_unique_int_array5_with_values(const int*)")]
    pub fn int_array5_new_with_values(values: *const i32) -> IntArray5;

    #[cpp(func = "std::unique_ptr<DoubleArray3> hicc::make_unique<DoubleArray3>()")]
    pub fn double_array3_new() -> DoubleArray3;

    #[cpp(func = "std::unique_ptr<DoubleArray3> _cpp2rust_make_unique_double_array3_with_values(const double*)")]
    pub fn double_array3_new_with_values(values: *const f64) -> DoubleArray3;

    #[cpp(func = "std::unique_ptr<StringArray4> hicc::make_unique<StringArray4>()")]
    pub fn string_array4_new() -> StringArray4;
}
