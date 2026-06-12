//! 004_inline_functions 冒烟测试
//!
//! 验证生成的 Rust FFI 绑定可编译、可链接 C++ 实现，且基本行为正确。

use inline_functions::*;

#[test]
fn smoke_min() {
    assert_eq!(min(10, 20), 10, "min(10, 20) 应返回 10");
    assert_eq!(min(20, 10), 10, "min(20, 10) 应返回 10");
    assert_eq!(min(-5, 5), -5, "min(-5, 5) 应返回 -5");
}

#[test]
fn smoke_max() {
    assert_eq!(max(10, 20), 20, "max(10, 20) 应返回 20");
    assert_eq!(max(20, 10), 20, "max(20, 10) 应返回 20");
}

#[test]
fn smoke_min_v2_matches_min() {
    assert_eq!(min_v2(10, 20), min(10, 20), "min_v2 应与 min 结果一致");
}

#[test]
fn smoke_max_v2_matches_max() {
    assert_eq!(max_v2(10, 20), max(10, 20), "max_v2 应与 max 结果一致");
}

#[test]
fn smoke_inline_equal_values() {
    assert_eq!(min(5, 5), 5, "min(5, 5) 应返回 5");
    assert_eq!(max(5, 5), 5, "max(5, 5) 应返回 5");
}
