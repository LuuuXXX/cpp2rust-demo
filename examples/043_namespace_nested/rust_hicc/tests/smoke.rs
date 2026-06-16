//! 043_namespace_nested 冒烟测试
//!
//! 验证生成的 Rust FFI 绑定可编译、可链接 C++ 实现，且嵌套命名空间类行为正确。

use namespace_nested::*;
use std::ffi::{CStr, CString};

#[test]
fn smoke_config_manager_set_get_size() {
    let mut config = ConfigManager::new();
    let timeout = CString::new("timeout").expect("CString::new failed");
    let retry = CString::new("retry").expect("CString::new failed");
    let missing = CString::new("missing").expect("CString::new failed");

    config.set_value(timeout.as_ptr(), 30);
    config.set_value(retry.as_ptr(), 3);
    assert_eq!(config.size(), 2);
    assert_eq!(config.get_value(timeout.as_ptr()), 30);
    assert_eq!(config.get_value(retry.as_ptr()), 3);
    assert_eq!(config.get_value(missing.as_ptr()), -1);
}

#[test]
fn smoke_config_manager_overwrite() {
    let mut config = ConfigManager::new();
    let key = CString::new("key").expect("CString::new failed");

    config.set_value(key.as_ptr(), 10);
    config.set_value(key.as_ptr(), 20);
    assert_eq!(config.size(), 1);
    assert_eq!(config.get_value(key.as_ptr()), 20);
}

#[test]
fn smoke_data_processor() {
    let processor = DataProcessor::new();
    assert_eq!(processor.process(5), 15);
}

#[test]
fn smoke_top_level_functions() {
    let version = unsafe { CStr::from_ptr(get_version()).to_string_lossy().into_owned() };
    assert_eq!(version, "1.0.0");
    assert_eq!(get_build_number(), 42);
}
