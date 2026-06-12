hicc::cpp! {
    #include "enum_class.h"
}

hicc::import_lib! {
    #![link_name = "enum_class"]

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

pub enum OperationResult {}

extern "C" {
    pub fn operation_result_new() -> *mut OperationResult;
    pub fn operation_result_set_error(p: *mut OperationResult, error_code: i32);
    pub fn operation_result_get_error(p: *mut OperationResult) -> i32;
    pub fn operation_result_set_state(p: *mut OperationResult, state: u8);
    pub fn operation_result_get_state(p: *mut OperationResult) -> u8;
    pub fn operation_result_set_flags(p: *mut OperationResult, flags: u32);
    pub fn operation_result_get_flags(p: *mut OperationResult) -> u32;
}

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
