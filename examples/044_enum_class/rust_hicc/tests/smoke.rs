//! 044_enum_class 冒烟测试
//!
//! 验证生成的 Rust FFI 绑定可编译、可链接 C++ 实现，且基本行为正确。

use enum_class::*;

#[test]
fn smoke_operation_result_error() {
    let result = unsafe { operation_result_new() };
    unsafe { operation_result_set_error(result, ERROR_INVALID_INPUT) };
    assert_eq!(unsafe { operation_result_get_error(result) }, ERROR_INVALID_INPUT, "设置错误码应为 ERROR_INVALID_INPUT");

    unsafe { operation_result_set_error(result, ERROR_NOT_FOUND) };
    assert_eq!(unsafe { operation_result_get_error(result) }, ERROR_NOT_FOUND, "设置错误码应为 ERROR_NOT_FOUND");
}

#[test]
fn smoke_operation_result_state() {
    let result = unsafe { operation_result_new() };
    unsafe { operation_result_set_state(result, STATE_RUNNING) };
    assert_eq!(unsafe { operation_result_get_state(result) }, STATE_RUNNING, "设置状态应为 STATE_RUNNING");

    unsafe { operation_result_set_state(result, STATE_PAUSED) };
    assert_eq!(unsafe { operation_result_get_state(result) }, STATE_PAUSED, "设置状态应为 STATE_PAUSED");
}

#[test]
fn smoke_flags() {
    let result = unsafe { operation_result_new() };
    unsafe { operation_result_set_flags(result, FLAG_READ | FLAG_WRITE) };
    let flags = unsafe { operation_result_get_flags(result) };
    assert_eq!(flags, FLAG_READ | FLAG_WRITE, "设置标志位应为 READ|WRITE");
    assert!(unsafe { has_flag(flags, FLAG_READ) } != 0, "应包含 FLAG_READ");
    assert!(unsafe { has_flag(flags, FLAG_WRITE) } != 0, "应包含 FLAG_WRITE");
    assert!(unsafe { has_flag(flags, FLAG_EXECUTE) } == 0, "不应包含 FLAG_EXECUTE");
}

#[test]
fn smoke_combine_flags() {
    let combined = unsafe { combine_flags(FLAG_READ, FLAG_EXECUTE) };
    assert_eq!(combined, FLAG_READ | FLAG_EXECUTE, "combine_flags 应合并两个标志");
}

#[test]
fn smoke_error_code_name() {
    assert_eq!(error_code_name(0), "None");
    assert_eq!(error_code_name(1), "InvalidInput");
    assert_eq!(error_code_name(3), "NotFound");
    assert_eq!(error_code_name(99), "Unknown");
}

#[test]
fn smoke_state_name() {
    assert_eq!(state_name(0), "Idle");
    assert_eq!(state_name(1), "Running");
    assert_eq!(state_name(2), "Paused");
    assert_eq!(state_name(3), "Stopped");
    assert_eq!(state_name(99), "Unknown");
}
