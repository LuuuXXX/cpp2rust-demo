//! 029_unique_ptr 冒烟测试
//!
//! 验证生成的 Rust FFI 绑定可编译、可链接 C++ 实现，且基本行为正确。

use unique_ptr::*;

#[test]
fn smoke_unique_buffer_new() {
    let buffer = UniqueBuffer::new(16);
    assert_eq!(buffer.size(), 16, "UniqueBuffer 容量应为 16");
}

#[test]
fn smoke_unique_buffer_use_count() {
    let buffer = UniqueBuffer::new(16);
    assert_eq!(buffer.use_count(), 1, "unique_ptr 的 use_count 应始终为 1");
}

#[test]
fn smoke_unique_buffer_fill_at() {
    let mut buffer = UniqueBuffer::new(16);
    buffer.fill(b'Z' as i8);
    assert_eq!(buffer.at(0), b'Z' as i8);
    assert_eq!(buffer.at(15), b'Z' as i8);
}

#[test]
fn smoke_unique_buffer_data() {
    let mut buffer = UniqueBuffer::new(16);
    let data_ptr = buffer.data();
    assert!(!data_ptr.is_null(), "data() 不应返回空指针");
}

#[test]
fn smoke_processor_process() {
    let mut processor = Processor::new();
    let input = std::ffi::CString::new("Hello").expect("CString::new failed");
    let result_ptr = processor.process(input.as_ptr());
    let result = unsafe {
        std::ffi::CStr::from_ptr(result_ptr)
            .to_string_lossy()
            .into_owned()
    };
    assert!(result.contains("Hello"), "处理结果应含输入");
    assert!(result.contains("processed"), "处理结果应含 [processed]");
}
