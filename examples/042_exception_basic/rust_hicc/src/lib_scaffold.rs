hicc::cpp! {
    #include <iostream>
    #include <stdexcept>
    #include <cstring>

    #include "exception_basic.h"
}

hicc::import_class! {
    #[cpp(class = "ExceptionInfo")]
    pub class ExceptionInfo {
        #[cpp(method = "void clear()")]
        pub fn clear(&mut self);

        #[cpp(method = "void set(int c, const char* msg)")]
        pub fn set(&mut self, c: i32, msg: *const i8);
    }
}

hicc::import_class! {
    #[cpp(class = "CalculatorImpl")]
    pub class CalculatorImpl {
        #[cpp(method = "void clear_exception()")]
        pub fn clear_exception(&mut self);

        #[cpp(method = "int get_exception()")]
        pub fn get_exception(&mut self) -> i32;

        #[cpp(method = "int divide(int a, int b)")]
        pub fn divide(&mut self, a: i32, b: i32) -> i32;

        #[cpp(method = "int safe_get(int* arr, int size, int index)")]
        pub fn safe_get(&mut self, arr: *mut i32, size: i32, index: i32) -> i32;

        #[cpp(method = "int string_to_int(const char* str)")]
        pub fn string_to_int(&mut self, str: *const i8) -> i32;
    }
}

hicc::import_class! {
    #[cpp(class = "Calculator")]
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

    class ExceptionInfo;
    class CalculatorImpl;
    class Calculator;

    #[cpp(func = "std::unique_ptr<ExceptionInfo> hicc::make_unique<ExceptionInfo>()")]
    pub fn exception_info_new() -> ExceptionInfo;

    #[cpp(func = "std::unique_ptr<CalculatorImpl> hicc::make_unique<CalculatorImpl>()")]
    pub fn calculator_impl_new() -> CalculatorImpl;

    #[cpp(func = "std::unique_ptr<Calculator> hicc::make_unique<Calculator>()")]
    pub fn calculator_new() -> Calculator;
}
