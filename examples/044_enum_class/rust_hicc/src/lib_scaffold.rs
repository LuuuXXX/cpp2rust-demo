hicc::cpp! {
    #include <cstddef>
    #include <cstdint>
    #include <iostream>

    #include "enum_class.h"

    using OperationResult = example::OperationResult;
}

hicc::import_class! {
    #[cpp(class = "OperationResult")]
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
    }
}

hicc::import_lib! {
    #![link_name = "enum_class"]

    class OperationResult;

    #[cpp(func = "std::unique_ptr<OperationResult> hicc::make_unique<OperationResult>()")]
    pub fn operation_result_new() -> OperationResult;
}
