hicc::cpp! {
    #include <stddef.h>
    #include <iostream>
    #include <stdexcept>
    #include <cstring>

    #include "exception_basic.h"
}

hicc::import_class! {
    #[cpp(class = "Calculator", destroy = "calculator_delete")]
    pub class Calculator {
        #[cpp(method = "void clear_exception()")]
        pub fn clear_exception(&mut self);

        #[cpp(method = "int get_exception()")]
        pub fn get_exception(&mut self) -> i32;

        #[cpp(method = "int divide(int a, int b)")]
        pub fn divide(&mut self, a: i32, b: i32) -> i32;

        #[cpp(method = "int string_to_int(const char* str)")]
        pub fn string_to_int(&mut self, str: *const i8) -> i32;
    }
}

hicc::import_lib! {
    #![link_name = "exception_basic"]

    class Calculator;

    #[cpp(func = "Calculator* calculator_new()")]
    pub fn calculator_new() -> Calculator;
}

pub fn check_exception(calc: &mut Calculator, op: &str) {
    let ex = calc.get_exception();
    if ex != 0 {
        let name = match ex {
            1 => "Invalid argument exception",
            2 => "Out of range exception",
            3 => "Runtime error exception",
            _ => "Unknown exception",
        };
        println!("  {}: {}", op, name);
        calc.clear_exception();
    } else {
        println!("  {}: No exception", op);
    }
}
