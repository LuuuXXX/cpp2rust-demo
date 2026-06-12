//! 003_default_args 冒烟测试
//!
//! 验证生成的 Rust FFI 绑定可编译、可链接 C++ 实现，且基本行为正确。

use default_args::*;

#[test]
fn smoke_greet_once() {
    let name = b"World\0".as_ptr() as *const i8;
    let result = unsafe { greet(name, 1) };
    assert_eq!(result, 1, "greet(\"World\", 1) 应返回调用次数 1");
}

#[test]
fn smoke_greet_multiple() {
    let name = b"Rust\0".as_ptr() as *const i8;
    let result = unsafe { greet(name, 3) };
    assert_eq!(result, 3, "greet(\"Rust\", 3) 应返回调用次数 3");
}

#[test]
fn smoke_greet_zero_times() {
    let name = b"Test\0".as_ptr() as *const i8;
    let result = unsafe { greet(name, 0) };
    assert_eq!(result, 0, "greet(\"Test\", 0) 应返回 0");
}
