//! 048_summary: 示例系列汇总（hicc 直出，去 shim）。
//!
//! `Counter` 直接持有对象状态，演示类方法、命名空间自由函数与 `make_unique` 工厂。
//! hicc 直出无需 extern-C 不透明指针 + `*_delete`，析构由 Rust `Drop` 自动完成。

hicc::cpp! {
    #include "summary.h"
}

hicc::import_class! {
    #[cpp(class = "summary_ns::Counter")]
    pub class Counter {
        #[cpp(method = "void increment()")]
        pub fn increment(&mut self);

        #[cpp(method = "void decrement()")]
        pub fn decrement(&mut self);

        #[cpp(method = "int get() const")]
        pub fn get(&self) -> i32;

        #[cpp(method = "void reset()")]
        pub fn reset(&mut self);

        pub fn new() -> Self { counter_new() }
    }
}

hicc::import_lib! {
    #![link_name = "summary"]

    #[cpp(func = "std::unique_ptr<summary_ns::Counter> hicc::make_unique<summary_ns::Counter>()")]
    pub fn counter_new() -> Counter;

    #[cpp(func = "int summary_ns::safe_add(int, int)")]
    pub fn safe_add(a: i32, b: i32) -> i32;

    #[cpp(func = "int summary_ns::max_size()")]
    pub fn max_size() -> i32;
}
