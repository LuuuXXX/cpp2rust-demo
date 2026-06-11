//! 004_inline_functions 冒烟测试
//!
//! 内联函数与普通函数在 FFI 中调用方式一致；验证返回值正确。

use inline_functions::*;

#[test]
fn smoke_min_max() {
    assert_eq!(min(10, 20), 10, "min 应返回较小值");
    assert_eq!(max(10, 20), 20, "max 应返回较大值");
}

#[test]
fn smoke_min_max_v2() {
    assert_eq!(min_v2(10, 20), 10, "min_v2 应返回较小值");
    assert_eq!(max_v2(10, 20), 20, "max_v2 应返回较大值");
}
