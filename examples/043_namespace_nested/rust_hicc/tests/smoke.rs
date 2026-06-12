//! 043_namespace_nested 冒烟测试
//!
//! 验证生成的 Rust FFI 绑定可编译、可链接 C++ 实现，且基本行为正确。

use namespace_nested::*;

#[test]
fn smoke_config_manager_set_get() {
    let config = config_manager_new();
    unsafe {
        config_manager_set_value(config, "timeout\0".as_ptr() as *const i8, 30);
        config_manager_set_value(config, "retry\0".as_ptr() as *const i8, 3);
        config_manager_set_value(config, "port\0".as_ptr() as *const i8, 8080);
    }
    assert_eq!(
        unsafe { config_manager_get_value(config, "timeout\0".as_ptr() as *const i8) },
        30,
        "timeout 应为 30"
    );
    assert_eq!(
        unsafe { config_manager_get_value(config, "retry\0".as_ptr() as *const i8) },
        3,
        "retry 应为 3"
    );
    assert_eq!(
        unsafe { config_manager_get_value(config, "port\0".as_ptr() as *const i8) },
        8080,
        "port 应为 8080"
    );
    unsafe { config_manager_delete(config); }
}

#[test]
fn smoke_config_manager_missing_key() {
    let config = config_manager_new();
    let val = unsafe { config_manager_get_value(config, "nonexistent\0".as_ptr() as *const i8) };
    assert_eq!(val, 0, "不存在的键应返回 0");
    unsafe { config_manager_delete(config); }
}

#[test]
fn smoke_config_manager_overwrite() {
    let config = config_manager_new();
    unsafe {
        config_manager_set_value(config, "key\0".as_ptr() as *const i8, 10);
        config_manager_set_value(config, "key\0".as_ptr() as *const i8, 20);
    }
    let val = unsafe { config_manager_get_value(config, "key\0".as_ptr() as *const i8) };
    assert_eq!(val, 20, "覆盖后的值应为 20");
    unsafe { config_manager_delete(config); }
}

#[test]
fn smoke_string_length() {
    let len = unsafe { string_length("Hello, World!\0".as_ptr() as *const i8) };
    assert_eq!(len, 13, "string_length(\"Hello, World!\") 应为 13");
}

#[test]
fn smoke_string_length_empty() {
    let len = unsafe { string_length("\0".as_ptr() as *const i8) };
    assert_eq!(len, 0, "空字符串长度应为 0");
}

#[test]
fn smoke_data_processor() {
    let processor = data_processor_new();
    let result = unsafe { data_processor_process(processor, 42) };
    // DataProcessor::multiplier_ = 1, so result = 42 * 1 = 42
    assert_eq!(result, 42, "data_processor_process(42) 应返回 42");
    unsafe { data_processor_delete(processor); }
}

#[test]
fn smoke_get_version() {
    use std::ffi::CStr;
    let version = unsafe { get_version() };
    let version_str = unsafe { CStr::from_ptr(version) }
        .to_str()
        .unwrap();
    assert_eq!(version_str, "1.0.0", "版本号应为 1.0.0");
}

#[test]
fn smoke_get_build_number() {
    let build = get_build_number();
    assert_eq!(build, 42, "构建号应为 42");
}
