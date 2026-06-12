//! 029_unique_ptr 冒烟测试
//!
//! 验证生成的 Rust FFI 绑定可编译、可链接 C++ 实现，且基本行为正确。

use unique_ptr::*;

#[test]
fn smoke_uniquebuffer_new() {
    let buffer = uniquebuffer_new(16);
    assert_eq!(buffer.get_size(), 16, "UniqueBuffer 容量应为 16");
}

#[test]
fn smoke_uniquebuffer_use_count() {
    let buffer = uniquebuffer_new(16);
    assert_eq!(buffer.use_count(), 1, "unique_ptr 的 use_count 应始终为 1");
}

#[test]
fn smoke_uniquebuffer_data() {
    let mut buffer = uniquebuffer_new(16);
    let data_ptr = buffer.get_data();
    assert!(!data_ptr.is_null(), "getData() 不应返回空指针");
    let slice = unsafe { std::slice::from_raw_parts(data_ptr as *const u8, 16) };
    // 初始数据应全部为零
    for &byte in slice {
        assert_eq!(byte, 0, "初始 buffer 数据应为零");
    }
}

#[test]
fn smoke_processor_new() {
    let _processor = processor_new();
    // 构造不崩溃即可
}

#[test]
fn smoke_processor_process() {
    let mut processor = processor_new();
    let input = std::ffi::CString::new("Hello").expect("CString::new failed");
    let result_ptr = processor.process(input.as_ptr());
    let result = unsafe {
        std::ffi::CStr::from_ptr(result_ptr as *const i8)
            .to_string_lossy()
            .into_owned()
    };
    assert!(!result.is_empty(), "处理结果不应为空");
}
