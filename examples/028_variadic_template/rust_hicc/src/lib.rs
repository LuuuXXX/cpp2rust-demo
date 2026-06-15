hicc::cpp! {
    #include <iostream>
    #include <cstdarg>

    #include "variadic_template.h"
}

hicc::import_lib! {
    #![link_name = "variadic_template"]

    class SumCalculator;

    #[cpp(func = "int SumCalculator::calculate_zero()")]
    pub fn sum_calculator_calculate_zero() -> i32;

    #[cpp(func = "int SumCalculator::calculate_1(int)")]
    pub fn sum_calculator_calculate_1(a: i32) -> i32;

    #[cpp(func = "int SumCalculator::calculate_2(int, int)")]
    pub fn sum_calculator_calculate_2(a: i32, b: i32) -> i32;

    #[cpp(func = "int SumCalculator::calculate_3(int, int, int)")]
    pub fn sum_calculator_calculate_3(a: i32, b: i32, c: i32) -> i32;

    #[cpp(func = "int SumCalculator::calculate_4(int, int, int, int)")]
    pub fn sum_calculator_calculate_4(a: i32, b: i32, c: i32, d: i32) -> i32;

    #[cpp(func = "int SumCalculator::calculate_5(int, int, int, int, int)")]
    pub fn sum_calculator_calculate_5(a: i32, b: i32, c: i32, d: i32, e: i32) -> i32;

    #[cpp(func = "double SumCalculator::calculate_double_2(double, double)")]
    pub fn sum_calculator_calculate_double_2(a: f64, b: f64) -> f64;

    #[cpp(func = "double SumCalculator::calculate_double_3(double, double, double)")]
    pub fn sum_calculator_calculate_double_3(a: f64, b: f64, c: f64) -> f64;

    #[cpp(func = "double SumCalculator::calculate_double_4(double, double, double, double)")]
    pub fn sum_calculator_calculate_double_4(a: f64, b: f64, c: f64, d: f64) -> f64;

    #[cpp(func = "const char* SumCalculator::get_format(int)")]
    pub fn sum_calculator_get_format(count: i32) -> *const i8;
}
