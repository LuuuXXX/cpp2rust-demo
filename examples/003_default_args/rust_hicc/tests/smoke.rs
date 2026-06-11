//! 003_default_args 冒烟测试
//!
//! C++ 默认参数在 FFI 中需显式传入；验证显式调用返回正确次数。

use default_args::*;

#[test]
fn smoke_greet_returns_times() {
    let name = b"World\0".as_ptr() as *const i8;
    let times = unsafe { greet(name, 3) };
    assert_eq!(times, 3, "greet 应返回打印次数 times");
}

#[test]
fn smoke_greet_default_simulated() {
    // Rust 侧模拟默认参数 times = 1。
    let name = b"World\0".as_ptr() as *const i8;
    let times = unsafe { greet(name, 1) };
    assert_eq!(times, 1, "默认参数 times = 1 时应返回 1");
}
