hicc::cpp! {
    #include <iostream>
    #include <cstdarg>

    #include "variadic_template.h"
}

hicc::import_lib! {
    #![link_name = "variadic_template"]

    #[cpp(func = "int sum_zero()")]
    pub fn sum_zero() -> i32;

    #[cpp(func = "int sum_1(int)")]
    pub fn sum_1(a: i32) -> i32;

    #[cpp(func = "int sum_2(int, int)")]
    pub fn sum_2(a: i32, b: i32) -> i32;

    #[cpp(func = "int sum_3(int, int, int)")]
    pub fn sum_3(a: i32, b: i32, c: i32) -> i32;

    #[cpp(func = "int sum_4(int, int, int, int)")]
    pub fn sum_4(a: i32, b: i32, c: i32, d: i32) -> i32;

    #[cpp(func = "int sum_5(int, int, int, int, int)")]
    pub fn sum_5(a: i32, b: i32, c: i32, d: i32, e: i32) -> i32;

    #[cpp(func = "double sum_double_2(double, double)")]
    pub fn sum_double_2(a: f64, b: f64) -> f64;

    #[cpp(func = "double sum_double_3(double, double, double)")]
    pub fn sum_double_3(a: f64, b: f64, c: f64) -> f64;

    #[cpp(func = "double sum_double_4(double, double, double, double)")]
    pub fn sum_double_4(a: f64, b: f64, c: f64, d: f64) -> f64;

    #[cpp(func = "const char* sum_getFormat(int)")]
    pub unsafe fn sum_get_format(count: i32) -> *const i8;
}
