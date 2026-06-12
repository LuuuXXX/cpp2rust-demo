//! 046_constexpr_basic 冒烟测试
//!
//! 验证生成的 Rust FFI 绑定可编译、可链接 C++ 实现，且基本行为正确。

use constexpr_basic::*;

#[test]
fn smoke_fibonacci_10() {
    let fib = get_fibonacci_10();
    assert_eq!(fib, 55, "fibonacci<10> 应返回 55");
    assert_eq!(fib, FIB_RUST, "C++ constexpr 值应与 Rust const 值一致");
}

#[test]
fn smoke_manhattan_distance() {
    assert_eq!(manhattan_distance(3, 4), 7, "manhattan_distance(3, 4) 应返回 7");
    assert_eq!(manhattan_distance(-3, -4), 7, "manhattan_distance(-3, -4) 应返回 7");
    assert_eq!(manhattan_distance(10, -5), 15, "manhattan_distance(10, -5) 应返回 15");
    assert_eq!(manhattan_distance(0, 0), 0, "manhattan_distance(0, 0) 应返回 0");
}

#[test]
fn smoke_array_operations() {
    let arr = [1, 5, 3, 9, 2, 8, 4, 7, 6, 0];
    let size = get_array_size();
    assert_eq!(size, 10, "get_array_size() 应返回 10");

    let sum = constexpr_sum_array(arr.as_ptr(), size);
    assert_eq!(sum, 45, "数组 [1..9,0] 的和应为 45");

    let max = constexpr_find_max(arr.as_ptr(), size);
    assert_eq!(max, 9, "数组最大值应为 9");
}
