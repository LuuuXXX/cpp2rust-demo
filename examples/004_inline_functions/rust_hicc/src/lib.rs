hicc::cpp! {
    #include "inline_functions.h"
}

hicc::import_lib! {
    #![link_name = "inline_functions"]

    #[cpp(func = "int min(int, int)")]
    pub fn min(a: i32, b: i32) -> i32;

    #[cpp(func = "int max(int, int)")]
    pub fn max(a: i32, b: i32) -> i32;

    #[cpp(func = "int min_v2(int, int)")]
    pub fn min_v2(a: i32, b: i32) -> i32;

    #[cpp(func = "int max_v2(int, int)")]
    pub fn max_v2(a: i32, b: i32) -> i32;
}
