hicc::cpp! {
    #include <cstddef>

    #include "constexpr_basic.h"

    int get_fibonacci_10() {
        return example::fibonacci<10>();
    }

    int manhattan_distance(int x, int y) {
        example::ConstexprPoint p(x, y);
        return p.manhattan_distance();
    }

    int constexpr_sum_array(const int* arr, int size) {
        int sum = 0;
        for (int i = 0; i < size; ++i) sum += arr[i];
        return sum;
    }

    int constexpr_find_max(const int* arr, int size) {
        int max_val = arr[0];
        for (int i = 1; i < size; ++i) if (arr[i] > max_val) max_val = arr[i];
        return max_val;
    }

    int get_array_size() {
        return 10;
    }
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

pub const FIB_RUST: i32 = 55;
