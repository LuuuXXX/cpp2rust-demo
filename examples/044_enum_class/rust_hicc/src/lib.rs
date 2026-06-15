hicc::cpp! {
    #include <cstddef>
    #include <cstdint>
    #include <iostream>

    #include "enum_class.h"

    using OperationResult = example::OperationResult;

    unsigned int combine_flags(unsigned int f1, unsigned int f2) {
        return f1 | f2;
    }

    int has_flag(unsigned int flags, unsigned int flag) {
        return (flags & flag) != 0 ? 1 : 0;
    }
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

    #[cpp(func = "unsigned int combine_flags(unsigned int, unsigned int)")]
    pub fn combine_flags(f1: u32, f2: u32) -> u32;

    #[cpp(func = "int has_flag(unsigned int, unsigned int)")]
    pub fn has_flag(flags: u32, flag: u32) -> i32;
}

pub const ERROR_INVALID_INPUT: i32 = 1;
pub const ERROR_NOT_FOUND: i32 = 3;
pub const STATE_RUNNING: u8 = 1;
pub const STATE_PAUSED: u8 = 2;
pub const FLAG_READ: u32 = 1;
pub const FLAG_WRITE: u32 = 2;
pub const FLAG_EXECUTE: u32 = 4;

pub fn error_code_name(code: i32) -> &'static str {
    match code {
        0 => "None",
        1 => "InvalidInput",
        2 => "OutOfMemory",
        3 => "NotFound",
        4 => "PermissionDenied",
        _ => "Unknown",
    }
}

pub fn state_name(s: u8) -> &'static str {
    match s {
        0 => "Idle",
        1 => "Running",
        2 => "Paused",
        3 => "Stopped",
        _ => "Unknown",
    }
}
