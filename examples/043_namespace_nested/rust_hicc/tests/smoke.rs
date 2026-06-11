//! 043_namespace_nested 冒烟测试
//!
//! 验证嵌套命名空间下的 ConfigManager 读写、字符串长度、DataProcessor 处理与顶层函数。

use namespace_nested::*;

fn cstr(s: &str) -> std::ffi::CString {
    std::ffi::CString::new(s).unwrap()
}

#[test]
fn smoke_config_manager_set_get() {
    let config = config_manager_new();
    let timeout = cstr("timeout");
    let retry = cstr("retry");
    unsafe {
        config_manager_set_value(config, timeout.as_ptr(), 30);
        config_manager_set_value(config, retry.as_ptr(), 3);
        assert_eq!(config_manager_get_value(config, timeout.as_ptr()), 30);
        assert_eq!(config_manager_get_value(config, retry.as_ptr()), 3);
        config_manager_delete(config);
    }
}

#[test]
fn smoke_string_length() {
    let s = cstr("Hello, World!");
    assert_eq!(unsafe { string_length(s.as_ptr()) }, 13);
}

#[test]
fn smoke_data_processor_and_top_level() {
    let processor = data_processor_new();
    // 默认 multiplier_ 为 1，process(42) == 42
    assert_eq!(unsafe { data_processor_process(processor, 42) }, 42);
    unsafe { data_processor_delete(processor) };

    let version = unsafe { std::ffi::CStr::from_ptr(get_version()).to_str().unwrap() };
    assert_eq!(version, "1.0.0");
    assert_eq!(get_build_number(), 42);
}
