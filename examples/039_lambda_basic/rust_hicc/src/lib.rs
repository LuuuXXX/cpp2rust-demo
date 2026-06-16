//! 039_lambda_basic: lambda 表达式（命名空间类内部持有由 lambda 构造的 std::function）。
//!
//! `Operation` 按 kind 选择 add/multiply/max 的 lambda，`Accumulator` 是捕获状态的闭包。
//! hicc 直出无需把函数指针跨 FFI 传递，lambda 在 C++ 侧内部持有；析构由 Rust `Drop` 自动完成。

hicc::cpp! {
    #include "lambda_basic.h"
}

hicc::import_class! {
    #[cpp(class = "lambda_basic_ns::Operation")]
    pub class Operation {
        #[cpp(method = "int apply(int a, int b) const")]
        pub fn apply(&self, a: i32, b: i32) -> i32;

        pub fn new(kind: i32) -> Self { operation_new(kind) }
    }
}

hicc::import_class! {
    #[cpp(class = "lambda_basic_ns::Accumulator")]
    pub class Accumulator {
        #[cpp(method = "int apply(int delta)")]
        pub fn apply(&mut self, delta: i32) -> i32;

        #[cpp(method = "int value() const")]
        pub fn value(&self) -> i32;

        pub fn new(initial: i32) -> Self { accumulator_new(initial) }
    }
}

hicc::import_lib! {
    #![link_name = "lambda_basic"]

    #[cpp(func = "std::unique_ptr<lambda_basic_ns::Operation> hicc::make_unique<lambda_basic_ns::Operation, int>(int&&)")]
    pub fn operation_new(kind: i32) -> Operation;

    #[cpp(func = "std::unique_ptr<lambda_basic_ns::Accumulator> hicc::make_unique<lambda_basic_ns::Accumulator, int>(int&&)")]
    pub fn accumulator_new(initial: i32) -> Accumulator;
}
