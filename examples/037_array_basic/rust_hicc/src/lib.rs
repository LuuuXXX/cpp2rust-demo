hicc::cpp! {
    #include <iostream>
    #include <array>
    #include <string>
    #include <cstring>

    #include "array_basic.h"

    extern "C" {
        IntArray5* hicc_int_array5_new() { return new IntArray5(); }
        IntArray5* hicc_int_array5_new_from(const int* values) { return new IntArray5(values); }
        DoubleArray3* hicc_double_array3_new() { return new DoubleArray3(); }
        DoubleArray3* hicc_double_array3_new_from(const double* values) { return new DoubleArray3(values); }
        StringArray4* hicc_string_array4_new() { return new StringArray4(); }
    }
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

    #[cpp(func = "IntArray5* hicc_int_array5_new()")]
    pub fn int_array5_new() -> IntArray5;

    #[cpp(func = "IntArray5* hicc_int_array5_new_from(const int*)")]
    pub unsafe fn int_array5_new_from(values: *const i32) -> IntArray5;

    #[cpp(func = "DoubleArray3* hicc_double_array3_new()")]
    pub fn double_array3_new() -> DoubleArray3;

    #[cpp(func = "DoubleArray3* hicc_double_array3_new_from(const double*)")]
    pub unsafe fn double_array3_new_from(values: *const f64) -> DoubleArray3;

    #[cpp(func = "StringArray4* hicc_string_array4_new()")]
    pub fn string_array4_new() -> StringArray4;
}
