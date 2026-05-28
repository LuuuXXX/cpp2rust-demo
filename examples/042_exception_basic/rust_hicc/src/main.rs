hicc::cpp! {
    #include <stddef.h>
    #include <iostream>
    #include <stdexcept>
    #include <cstring>

    class ExceptionInfo {
    public:
        int code;
        char message[256];
    public:
        ExceptionInfo() : code(0) {
    message[0] = '\0';
}
        void clear() {
    code = 0;
    message[0] = '\0';
}
        void set(int c, const char* msg) {
    code = c;
    strncpy(message, msg, 255);
    message[255] = '\0';
}
    };

    class CalculatorImpl {
    public:
        ExceptionInfo last_exception;
    public:
        CalculatorImpl() {}
        ~CalculatorImpl() {}
        void clear_exception() {
    last_exception.clear();
}
        int get_exception() {
    return last_exception.code;
}
        int divide(int a, int b) {
    if (b == 0) {
        last_exception.set(3, "Division by zero");
        throw std::runtime_error("Division by zero");
    }
    return a / b;
}
        int safe_get(int* arr, int size, int index) {
    if (index < 0 || index >= size) {
        last_exception.set(2, "Index out of range");
        throw std::out_of_range("Index out of range");
    }
    return arr[index];
}
        int string_to_int(const char* str) {
    if (!str || *str == '\0') {
        last_exception.set(1, "Empty string");
        throw std::invalid_argument("Empty string");
    }
    char* end;
    int result = std::strtol(str, &end, 10);
    if (*end != '\0') {
        last_exception.set(1, "Invalid number format");
        throw std::invalid_argument("Invalid number format");
    }
    return result;
}
    };

    struct Calculator {
    public:
        CalculatorImpl* impl;
        Calculator() : impl(new CalculatorImpl()) {}
        ~Calculator() { delete impl; }
        void clear_exception() { impl->clear_exception(); }
        int get_exception() { return impl->get_exception(); }
        int divide(int a, int b) {
    try { return impl->divide(a, b); } catch (...) { return 0; }
}
        int string_to_int(const char* str) {
    try { return impl->string_to_int(str); } catch (...) { return 0; }
}
    };

    Calculator* calculator_new() {
        return new Calculator();
    }

    void calculator_delete(Calculator* self) {
        delete self;
    }
}

hicc::import_class! {
    #[cpp(class = "Calculator")]
    class Calculator {
        #[cpp(method = "void clear_exception()")]
        fn clear_exception(&mut self);

        #[cpp(method = "int get_exception()")]
        fn get_exception(&mut self) -> i32;

        #[cpp(method = "int divide(int, int)")]
        fn divide(&mut self, a: i32, b: i32) -> i32;

        #[cpp(method = "int string_to_int(const char*)")]
        fn string_to_int(&mut self, str: *const i8) -> i32;
    }
}

hicc::import_lib! {
    #![link_name = "exception_basic"]

    class Calculator;

    #[cpp(func = "Calculator* calculator_new()")]
    fn calculator_new() -> *mut Calculator;

    #[cpp(func = "void calculator_delete(Calculator* self)")]
    unsafe fn calculator_delete(self_: *mut Calculator);
}

fn check_exception(calc: &mut Calculator, operation: &str) {
    let code = calc.get_exception();
    match code {
        0 => println!("  {}: No exception", operation),
        1 => println!("  {}: Invalid argument exception", operation),
        2 => println!("  {}: Out of range exception", operation),
        3 => println!("  {}: Runtime error exception", operation),
        _ => println!("  {}: Unknown exception code: {}", operation, code),
    }
}

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

    unsafe {
        calculator_delete(&mut calc);
    }

    println!("\n--- Summary ---");
    println!("1. C++ exceptions CANNOT propagate across FFI boundary");
    println!("2. Common FFI pattern: set error code, return error value");
    println!("3. Check exception/error state after each call");
    println!("4. Clear exception state before next operation");
    println!("5. Never throw in FFI boundary - use error codes instead");
}



