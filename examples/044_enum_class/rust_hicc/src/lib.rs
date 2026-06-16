//! 044_enum_class: enum class 强类型枚举（命名空间类直接持有枚举）。
//!
//! `OperationResult` 直接持有 `enum class` 成员，演示通过底层整数 set/get 与 bit flags
//! 等基本操作。hicc 直出无需 extern-C 不透明指针 + `*_delete`，析构由 Rust `Drop` 自动完成。

hicc::cpp! {
    #include "enum_class.h"
}

hicc::import_class! {
    #[cpp(class = "enum_class_ns::OperationResult")]
    pub class OperationResult {
        #[cpp(method = "void set_error(int code)")]
        pub fn set_error(&mut self, code: i32);

        #[cpp(method = "int get_error() const")]
        pub fn get_error(&self) -> i32;

        #[cpp(method = "void set_state(unsigned char s)")]
        pub fn set_state(&mut self, s: u8);

        #[cpp(method = "unsigned char get_state() const")]
        pub fn get_state(&self) -> u8;

        #[cpp(method = "void set_flags(unsigned int f)")]
        pub fn set_flags(&mut self, f: u32);

        #[cpp(method = "unsigned int get_flags() const")]
        pub fn get_flags(&self) -> u32;

        pub fn new() -> Self { operation_result_new() }
    }
}

hicc::import_lib! {
    #![link_name = "enum_class"]

    #[cpp(func = "std::unique_ptr<enum_class_ns::OperationResult> hicc::make_unique<enum_class_ns::OperationResult>()")]
    pub fn operation_result_new() -> OperationResult;

    #[cpp(func = "unsigned int enum_class_ns::combine_flags(unsigned int, unsigned int)")]
    pub fn combine_flags(f1: u32, f2: u32) -> u32;

    #[cpp(func = "int enum_class_ns::has_flag(unsigned int, unsigned int)")]
    pub fn has_flag(flags: u32, flag: u32) -> i32;
}
