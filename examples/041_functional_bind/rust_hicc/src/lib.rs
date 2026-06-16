//! 041_functional_bind: std::bind（命名空间类内部持有由 std::bind 构造的 std::function）。
//!
//! `Adder` / `Multiplier` 在 C++ 构造函数中用 `std::bind` 绑定基础参数，`StringProcessor`
//! 持有字符串状态并统计字符。hicc 直出无需 extern-C shim，析构由 Rust `Drop` 自动完成。

hicc::cpp! {
    #include "functional_bind.h"
}

hicc::import_class! {
    #[cpp(class = "functional_bind_ns::Adder")]
    pub class Adder {
        #[cpp(method = "int add(int value) const")]
        pub fn add(&self, value: i32) -> i32;

        pub fn new(base: i32) -> Self { adder_new(base) }
    }
}

hicc::import_class! {
    #[cpp(class = "functional_bind_ns::Multiplier")]
    pub class Multiplier {
        #[cpp(method = "int multiply(int value) const")]
        pub fn multiply(&self, value: i32) -> i32;

        pub fn new(factor: i32) -> Self { multiplier_new(factor) }
    }
}

hicc::import_class! {
    #[cpp(class = "functional_bind_ns::StringProcessor")]
    pub class StringProcessor {
        #[cpp(method = "void set_target(const char* t)")]
        pub fn set_target(&mut self, t: *const i8);

        #[cpp(method = "int count_char(char ch) const")]
        pub fn count_char(&self, ch: i8) -> i32;

        pub fn new() -> Self { string_processor_new() }
    }
}

hicc::import_lib! {
    #![link_name = "functional_bind"]

    #[cpp(func = "std::unique_ptr<functional_bind_ns::Adder> hicc::make_unique<functional_bind_ns::Adder, int>(int&&)")]
    pub fn adder_new(base: i32) -> Adder;

    #[cpp(func = "std::unique_ptr<functional_bind_ns::Multiplier> hicc::make_unique<functional_bind_ns::Multiplier, int>(int&&)")]
    pub fn multiplier_new(factor: i32) -> Multiplier;

    #[cpp(func = "std::unique_ptr<functional_bind_ns::StringProcessor> hicc::make_unique<functional_bind_ns::StringProcessor>()")]
    pub fn string_processor_new() -> StringProcessor;
}
