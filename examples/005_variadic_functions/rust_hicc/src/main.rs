hicc::cpp! {
    #include "variadic_functions.h"
}

hicc::import_lib! {
    #![link_name = "variadic_functions"]

    #[cpp(func = "int sum(int)")]
    fn sum(count: i32) -> i32;

    #[cpp(func = "int print_formatted(const char*)")]
    unsafe fn print_formatted(format: *const i8) -> i32;

    #[cpp(func = "int sum_3(int, int, int)")]
    fn sum_3(a: i32, b: i32, c: i32) -> i32;

    #[cpp(func = "int sum_5(int, int, int, int, int)")]
    fn sum_5(a: i32, b: i32, c: i32, d: i32, e: i32) -> i32;
}
