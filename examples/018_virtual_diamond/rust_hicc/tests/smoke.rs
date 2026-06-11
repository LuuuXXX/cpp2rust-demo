//! 018_virtual_diamond 冒烟测试
//!
//! 验证生成的 Rust FFI 绑定可编译、可链接 C++ 实现，且基本行为正确。

use hicc::AbiClass;
use virtual_diamond::*;

#[test]
fn smoke_d_values() {
    let mut d = d_new(1, 2, 3, 4);
    assert_eq!(d_get_a_value(&d.as_mut_ptr()), 1, "getAValue() 应返回 a=1");
    assert_eq!(d.get_b_value(), 2, "getBValue() 应返回 b=2");
    assert_eq!(d.get_c_value(), 3, "getCValue() 应返回 c=3");
    assert_eq!(d.get_d_value(), 4, "getDValue() 应返回 d=4");
}

#[test]
fn smoke_d_compute() {
    let d = d_new(1, 2, 3, 4);
    // compute() 只输出信息，不做断言，仅验证调用不崩溃
    d.compute();
}
