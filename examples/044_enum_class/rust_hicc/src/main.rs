// 044_enum_class - 强类型枚举
// 使用 raw extern "C" 模式

hicc::cpp! {
    #include <cstddef>
    #include <cstdint>
    #include <iostream>

    class example_OperationResult {
        ErrorCode error_;
        State state_;
        Flags flags_;
    public:
        OperationResult() = default;
        ~OperationResult() = default;
        void set_error(int code) {}
        int get_error() const {}
        void set_state(unsigned char s) {}
        unsigned char get_state() const {}
        void set_flags(unsigned int f) {}
        unsigned int get_flags() const {}
    };

    example::OperationResult* operation_result_new() {
        return new example::OperationResult();
    }

    void operation_result_delete(example::OperationResult* p) {
        delete p;
    }

    void operation_result_set_error(example::OperationResult* p, int error_code) {
        if (p) p->set_error(error_code);
    }

    int operation_result_get_error(example::OperationResult* p) {
        if (p) return p->get_error();
        return 0;
    }

    void operation_result_set_state(example::OperationResult* p, unsigned char state) {
        if (p) p->set_state(state);
    }

    unsigned char operation_result_get_state(example::OperationResult* p) {
        if (p) return p->get_state();
        return 0;
    }

    void operation_result_set_flags(example::OperationResult* p, unsigned int flags) {
        if (p) p->set_flags(flags);
    }

    unsigned int operation_result_get_flags(example::OperationResult* p) {
        if (p) return p->get_flags();
        return 0;
    }

    unsigned int combine_flags(unsigned int f1, unsigned int f2) {
        return f1 | f2;
    }

    int has_flag(unsigned int flags, unsigned int flag) {
        return (flags & flag) == flag;
    }
}

hicc::import_class! {
    #[cpp(class = "example_OperationResult")]
    class example_OperationResult {
        #[cpp(method = "void set_error(int code)")]
        fn set_error(&mut self, code: i32);

        #[cpp(method = "int get_error() const")]
        fn get_error(&self) -> i32;

        #[cpp(method = "void set_state(unsigned char s)")]
        fn set_state(&mut self, s: u8);

        #[cpp(method = "unsigned char get_state() const")]
        fn get_state(&self) -> u8;

        #[cpp(method = "void set_flags(unsigned int f)")]
        fn set_flags(&mut self, f: u32);

        #[cpp(method = "unsigned int get_flags() const")]
        fn get_flags(&self) -> u32;
    }
}

hicc::import_lib! {
    #![link_name = "enum_class"]

    #[cpp(func = "example::OperationResult* operation_result_new()")]
    fn operation_result_new() -> *mut example::OperationResult;

    #[cpp(func = "void operation_result_delete(example::OperationResult*)")]
    unsafe fn operation_result_delete(p: *mut example::OperationResult);

    #[cpp(func = "void operation_result_set_error(example::OperationResult*, int)")]
    unsafe fn operation_result_set_error(p: *mut example::OperationResult, error_code: i32);

    #[cpp(func = "int operation_result_get_error(example::OperationResult*)")]
    unsafe fn operation_result_get_error(p: *mut example::OperationResult) -> i32;

    #[cpp(func = "void operation_result_set_state(example::OperationResult*, unsigned char)")]
    unsafe fn operation_result_set_state(p: *mut example::OperationResult, state: u8);

    #[cpp(func = "unsigned char operation_result_get_state(example::OperationResult*)")]
    unsafe fn operation_result_get_state(p: *mut example::OperationResult) -> u8;

    #[cpp(func = "void operation_result_set_flags(example::OperationResult*, unsigned int)")]
    unsafe fn operation_result_set_flags(p: *mut example::OperationResult, flags: u32);

    #[cpp(func = "unsigned int operation_result_get_flags(example::OperationResult*)")]
    unsafe fn operation_result_get_flags(p: *mut example::OperationResult) -> u32;

    #[cpp(func = "unsigned int combine_flags(unsigned int, unsigned int)")]
    fn combine_flags(f1: u32, f2: u32) -> u32;

    #[cpp(func = "int has_flag(unsigned int, unsigned int)")]
    fn has_flag(flags: u32, flag: u32) -> i32;
}

// 使用 opaque pointer 别名
type OperationResult = *mut std::ffi::c_void;

// 直接使用 extern "C" 声明
#[link(name = "enum_class")]
unsafe extern "C" {
    fn operation_result_new() -> OperationResult;
    fn operation_result_delete(p: OperationResult);
    fn operation_result_set_error(p: OperationResult, error_code: i32);
    fn operation_result_get_error(p: OperationResult) -> i32;
    fn operation_result_set_state(p: OperationResult, state: u8);
    fn operation_result_get_state(p: OperationResult) -> u8;
    fn operation_result_set_flags(p: OperationResult, flags: u32);
    fn operation_result_get_flags(p: OperationResult) -> u32;
    fn combine_flags(f1: u32, f2: u32) -> u32;
    fn has_flag(flags: u32, flag: u32) -> i32;
}

// Enum constants for Rust
pub const ERROR_NONE: i32 = 0;
pub const ERROR_INVALID_INPUT: i32 = 1;
pub const ERROR_OUT_OF_MEMORY: i32 = 2;
pub const ERROR_NOT_FOUND: i32 = 3;
pub const ERROR_PERMISSION_DENIED: i32 = 4;
pub const ERROR_UNKNOWN: i32 = 99;

pub const STATE_IDLE: u8 = 0;
pub const STATE_RUNNING: u8 = 1;
pub const STATE_PAUSED: u8 = 2;
pub const STATE_STOPPED: u8 = 3;

pub const FLAG_NONE: u32 = 0;
pub const FLAG_READ: u32 = 1;
pub const FLAG_WRITE: u32 = 2;
pub const FLAG_EXECUTE: u32 = 4;
pub const FLAG_ALL: u32 = 7;

fn error_code_name(code: i32) -> &'static str {
    match code {
        0 => "None",
        1 => "InvalidInput",
        2 => "OutOfMemory",
        3 => "NotFound",
        4 => "PermissionDenied",
        99 => "Unknown",
        _ => "Unknown",
    }
}

fn state_name(state: u8) -> &'static str {
    match state {
        0 => "Idle",
        1 => "Running",
        2 => "Paused",
        3 => "Stopped",
        _ => "Unknown",
    }
}

fn main() {
    println!("=== 044_enum_class - 强类型枚举 ===\n");

    let result = unsafe { operation_result_new() };

    // ErrorCode example
    println!("--- ErrorCode Demo ---");
    unsafe { operation_result_set_error(result, ERROR_INVALID_INPUT) };
    println!("Error: {} (code={})", error_code_name(unsafe { operation_result_get_error(result) }), unsafe { operation_result_get_error(result) });

    unsafe { operation_result_set_error(result, ERROR_NOT_FOUND) };
    println!("Error: {} (code={})", error_code_name(unsafe { operation_result_get_error(result) }), unsafe { operation_result_get_error(result) });

    // State example
    println!("\n--- State Demo ---");
    unsafe { operation_result_set_state(result, STATE_RUNNING) };
    println!("State: {} (value={})", state_name(unsafe { operation_result_get_state(result) }), unsafe { operation_result_get_state(result) });

    unsafe { operation_result_set_state(result, STATE_PAUSED) };
    println!("State: {} (value={})", state_name(unsafe { operation_result_get_state(result) }), unsafe { operation_result_get_state(result) });

    // Flags example
    println!("\n--- Flags Demo ---");
    unsafe { operation_result_set_flags(result, FLAG_READ | FLAG_WRITE) };
    let flags = unsafe { operation_result_get_flags(result) };
    println!("Flags: {:03b} (read={}, write={}, execute={})",
        flags,
        unsafe { has_flag(flags, FLAG_READ) } != 0,
        unsafe { has_flag(flags, FLAG_WRITE) } != 0,
        unsafe { has_flag(flags, FLAG_EXECUTE) } != 0
    );

    let combined = unsafe { combine_flags(FLAG_READ, FLAG_EXECUTE) };
    unsafe { operation_result_set_flags(result, combined) };
    println!("Combined flags: {:03b}", unsafe { operation_result_get_flags(result) });

    unsafe { operation_result_delete(result); }

    println!("\n--- 总结 ---");
    println!("1. enum class 是强类型，不会隐式转换为 int");
    println!("2. 可以指定底层类型：enum class Foo : int");
    println!("3. FFI 传递枚举值作为整数");
    println!("4. Rust 端定义相应常量来模拟枚举");
    println!("5. 强类型枚举更安全，避免枚举值混淆");
}



