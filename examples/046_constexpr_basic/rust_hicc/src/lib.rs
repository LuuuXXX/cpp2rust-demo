hicc::cpp! {
    #include "constexpr_basic.h"
}

hicc::import_lib! {
    #![link_name = "constexpr_basic"]

    #[cpp(func = "int get_fibonacci_10()")]
    pub fn get_fibonacci_10() -> i32;

    #[cpp(func = "int manhattan_distance(int, int)")]
    pub fn manhattan_distance(x: i32, y: i32) -> i32;

    #[cpp(func = "int constexpr_sum_array(const int*, int)")]
    pub fn constexpr_sum_array(arr: *const i32, size: i32) -> i32;

    #[cpp(func = "int constexpr_find_max(const int*, int)")]
    pub fn constexpr_find_max(arr: *const i32, size: i32) -> i32;

    #[cpp(func = "int get_array_size()")]
    pub fn get_array_size() -> i32;
}
