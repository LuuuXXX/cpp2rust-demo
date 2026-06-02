hicc::cpp! {
    #include "enum_class.h"
}

hicc::import_lib! {
    #![link_name = "enum_class"]

    #[cpp(func = "unsigned int combine_flags(unsigned int, unsigned int)")]
    fn combine_flags(f1: u32, f2: u32) -> u32;

    #[cpp(func = "int has_flag(unsigned int, unsigned int)")]
    fn has_flag(flags: u32, flag: u32) -> i32;
}

const ERROR_INVALID_INPUT: i32 = 1;
const ERROR_NOT_FOUND: i32 = 3;
const STATE_RUNNING: u8 = 1;
const STATE_PAUSED: u8 = 2;
const FLAG_READ: u32 = 1;
const FLAG_WRITE: u32 = 2;
const FLAG_EXECUTE: u32 = 4;

enum OperationResult {}

extern "C" {
    fn operation_result_new() -> *mut OperationResult;
    fn operation_result_set_error(p: *mut OperationResult, error_code: i32);
    fn operation_result_get_error(p: *mut OperationResult) -> i32;
    fn operation_result_set_state(p: *mut OperationResult, state: u8);
    fn operation_result_get_state(p: *mut OperationResult) -> u8;
    fn operation_result_set_flags(p: *mut OperationResult, flags: u32);
    fn operation_result_get_flags(p: *mut OperationResult) -> u32;
}

fn error_code_name(code: i32) -> &'static str {
    match code {
        0 => "None",
        1 => "InvalidInput",
        2 => "OutOfMemory",
        3 => "NotFound",
        4 => "PermissionDenied",
        _ => "Unknown",
    }
}

fn state_name(s: u8) -> &'static str {
    match s {
        0 => "Idle",
        1 => "Running",
        2 => "Paused",
        3 => "Stopped",
        _ => "Unknown",
    }
}

fn main() {
    println!("=== 044_enum_class - 强类型枚举 ===\n");

    let result = unsafe { operation_result_new() };

    // ErrorCode example
    println!("--- ErrorCode Demo ---");
    unsafe { operation_result_set_error(result, ERROR_INVALID_INPUT) };
    println!("Error: {} (code={})", error_code_name(unsafe { operation_result_get_error(result) }), unsafe { operation_result_get_error(result) });

    unsafe { operation_result_set_error(result, ERROR_NOT_FOUND) };
    println!("Error: {} (code={})", error_code_name(unsafe { operation_result_get_error(result) }), unsafe { operation_result_get_error(result) });

    // State example
    println!("\n--- State Demo ---");
    unsafe { operation_result_set_state(result, STATE_RUNNING) };
    println!("State: {} (value={})", state_name(unsafe { operation_result_get_state(result) }), unsafe { operation_result_get_state(result) });

    unsafe { operation_result_set_state(result, STATE_PAUSED) };
    println!("State: {} (value={})", state_name(unsafe { operation_result_get_state(result) }), unsafe { operation_result_get_state(result) });

    // Flags example
    println!("\n--- Flags Demo ---");
    unsafe { operation_result_set_flags(result, FLAG_READ | FLAG_WRITE) };
    let flags = unsafe { operation_result_get_flags(result) };
    println!("Flags: {:03b} (read={}, write={}, execute={})",
        flags,
        unsafe { has_flag(flags, FLAG_READ) } != 0,
        unsafe { has_flag(flags, FLAG_WRITE) } != 0,
        unsafe { has_flag(flags, FLAG_EXECUTE) } != 0
    );

    let combined = unsafe { combine_flags(FLAG_READ, FLAG_EXECUTE) };
    unsafe { operation_result_set_flags(result, combined) };
    println!("Combined flags: {:03b}", unsafe { operation_result_get_flags(result) });

    println!("\n--- 总结 ---");
    println!("1. enum class 是强类型，不会隐式转换为 int");
    println!("2. 可以指定底层类型：enum class Foo : int");
    println!("3. FFI 传递枚举值作为整数");
    println!("4. Rust 端定义相应常量来模拟枚举");
    println!("5. 强类型枚举更安全，避免枚举值混淆");
}

