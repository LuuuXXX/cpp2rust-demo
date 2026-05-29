hicc::cpp! {
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

