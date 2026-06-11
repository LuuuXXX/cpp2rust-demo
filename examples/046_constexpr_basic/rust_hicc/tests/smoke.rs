//! 046_constexpr_basic 冒烟测试
//!
//! 验证编译期 fibonacci、曼哈顿距离以及数组求和/求最大值等 constexpr 函数。

use constexpr_basic::*;

#[test]
fn smoke_fibonacci_and_manhattan() {
    assert_eq!(get_fibonacci_10(), 55);
    assert_eq!(manhattan_distance(3, 4), 7);
    assert_eq!(manhattan_distance(-3, -4), 7);
    assert_eq!(manhattan_distance(10, -5), 15);
}

#[test]
fn smoke_array_operations() {
    let arr = [1, 5, 3, 9, 2, 8, 4, 7, 6, 0];
    let size = get_array_size();
    assert_eq!(size, 10);
    assert_eq!(constexpr_sum_array(arr.as_ptr(), size), 45);
    assert_eq!(constexpr_find_max(arr.as_ptr(), size), 9);
}
