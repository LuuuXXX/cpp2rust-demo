hicc::cpp! {
    #include "function_overload.h"
}

hicc::import_lib! {
    #![link_name = "function_overload"]

    #[cpp(func = "int function_overload_ns::add_int(int, int)")]
    pub fn add_int(a: i32, b: i32) -> i32;

    #[cpp(func = "double function_overload_ns::add_double(double, double)")]
    pub fn add_double(a: f64, b: f64) -> f64;

    #[cpp(func = "const char* function_overload_ns::add_strings(const char*, const char*)")]
    pub unsafe fn add_strings(a: *const i8, b: *const i8) -> *const i8;

    #[cpp(func = "int function_overload_ns::sum3(int, int, int)")]
    pub fn sum3(a: i32, b: i32, c: i32) -> i32;
}
