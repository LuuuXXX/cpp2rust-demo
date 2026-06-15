use enum_class::*;

fn main() {
    println!("=== 044_enum_class - Strongly-typed Enums ===\n");

    let mut result = operation_result_new();

    println!("--- ErrorCode Demo ---");
    result.set_error(ERROR_INVALID_INPUT);
    println!("Error: {} (code={})", error_code_name(result.get_error()), result.get_error());

    result.set_error(ERROR_NOT_FOUND);
    println!("Error: {} (code={})", error_code_name(result.get_error()), result.get_error());

    println!("\n--- State Demo ---");
    result.set_state(STATE_RUNNING);
    println!("State: {} (value={})", state_name(result.get_state()), result.get_state());

    result.set_state(STATE_PAUSED);
    println!("State: {} (value={})", state_name(result.get_state()), result.get_state());

    println!("\n--- Flags Demo ---");
    result.set_flags(FLAG_READ | FLAG_WRITE);
    let flags = result.get_flags();
    println!("Flags: {:03b} (read={}, write={}, execute={})",
        flags,
        has_flag(flags, FLAG_READ) != 0,
        has_flag(flags, FLAG_WRITE) != 0,
        has_flag(flags, FLAG_EXECUTE) != 0
    );

    let combined = combine_flags(FLAG_READ, FLAG_EXECUTE);
    result.set_flags(combined);
    println!("Combined flags: {:03b}", result.get_flags());

    println!("\n--- Summary ---");
    println!("1. enum class is strongly typed, no implicit conversion to int");
    println!("2. Can specify underlying type: enum class Foo : int");
    println!("3. FFI passes enum values as integers");
    println!("4. Rust side defines corresponding constants to simulate enums");
    println!("5. Strongly-typed enums are safer, avoiding enum value confusion");
}
