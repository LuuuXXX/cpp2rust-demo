//! 040_std_function: std::function 回调（命名空间类内部持有 std::function）。
//!
//! `Callback` 按 kind 选择 double/triple/negate 的 lambda，`Pipeline` 按顺序持有并运行多个回调。
//! hicc 直出无需把函数指针跨 FFI 传递，回调在 C++ 侧内部持有；析构由 Rust `Drop` 自动完成。

hicc::cpp! {
    #include "std_function.h"
}

hicc::import_class! {
    #[cpp(class = "std_function_ns::Callback")]
    pub class Callback {
        #[cpp(method = "int invoke(int v) const")]
        pub fn invoke(&self, v: i32) -> i32;

        pub fn new(kind: i32) -> Self { callback_new(kind) }
    }
}

hicc::import_class! {
    #[cpp(class = "std_function_ns::Pipeline")]
    pub class Pipeline {
        #[cpp(method = "void add(int kind)")]
        pub fn add(&mut self, kind: i32);

        #[cpp(method = "int run(int v) const")]
        pub fn run(&self, v: i32) -> i32;

        #[cpp(method = "int size() const")]
        pub fn size(&self) -> i32;

        pub fn new() -> Self { pipeline_new() }
    }
}

hicc::import_lib! {
    #![link_name = "std_function"]

    #[cpp(func = "std::unique_ptr<std_function_ns::Callback> hicc::make_unique<std_function_ns::Callback, int>(int&&)")]
    pub fn callback_new(kind: i32) -> Callback;

    #[cpp(func = "std::unique_ptr<std_function_ns::Pipeline> hicc::make_unique<std_function_ns::Pipeline>()")]
    pub fn pipeline_new() -> Pipeline;
}
