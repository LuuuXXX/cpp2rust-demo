//! 042_exception_basic: C++ 异常在方法边界内部捕获（hicc 直出）。
//!
//! `Calculator` 的方法内部使用真实 `throw` / `catch`，并把异常转换为对象内错误码。
//! hicc 直出无需 extern-C 不透明指针 + `*_delete`，析构由 Rust `Drop` 自动完成。

hicc::cpp! {
    #include "exception_basic.h"
}

hicc::import_class! {
    #[cpp(class = "exception_basic_ns::Calculator")]
    pub class Calculator {
        #[cpp(method = "int last_error() const")]
        pub fn last_error(&self) -> i32;

        #[cpp(method = "void clear_error()")]
        pub fn clear_error(&mut self);

        #[cpp(method = "int has_error() const")]
        pub fn has_error(&self) -> i32;

        #[cpp(method = "int divide(int a, int b)")]
        pub fn divide(&mut self, a: i32, b: i32) -> i32;

        #[cpp(method = "int parse_int(const char* s)")]
        pub fn parse_int(&mut self, s: *const i8) -> i32;

        pub fn new() -> Self { calculator_new() }
    }
}

hicc::import_lib! {
    #![link_name = "exception_basic"]

    #[cpp(func = "std::unique_ptr<exception_basic_ns::Calculator> hicc::make_unique<exception_basic_ns::Calculator>()")]
    pub fn calculator_new() -> Calculator;
}
