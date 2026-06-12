use exception_basic::*;

fn main() {
    println!("=== 042_exception_basic - Exception Handling ===\n");

    let mut calc = calculator_new();

    // Test division - normal case
    println!("--- Division Tests ---");
    let result = calc.divide(10, 2);
    println!("10 / 2 = {}", result);
    check_exception(&mut calc, "10 / 2");

    // Test division by zero
    println!("\nTesting division by zero:");
    let result = calc.divide(10, 0);
    println!("10 / 0 = {} (returns 0, check exception)", result);
    check_exception(&mut calc, "10 / 0");

    // Clear exception and test division again
    println!("\nAfter clearing exception:");
    calc.clear_exception();
    let result = calc.divide(20, 4);
    println!("20 / 4 = {}", result);
    check_exception(&mut calc, "20 / 4");

    // Test string to int
    println!("\n--- String to Int Tests ---");
    let result = calc.string_to_int("123\0".as_ptr() as *const i8);
    println!("string_to_int(\"123\") = {}", result);
    check_exception(&mut calc, "string_to_int(\"123\")");

    let result = calc.string_to_int("abc\0".as_ptr() as *const i8);
    println!("string_to_int(\"abc\") = {} (returns 0, check exception)", result);
    check_exception(&mut calc, "string_to_int(\"abc\")");

    println!("\n--- Summary ---");
    println!("1. C++ exceptions CANNOT propagate across FFI boundary");
    println!("2. Common FFI pattern: set error code, return error value");
    println!("3. Check exception/error state after each call");
    println!("4. Clear exception state before next operation");
    println!("5. Never throw in FFI boundary - use error codes instead");
}
