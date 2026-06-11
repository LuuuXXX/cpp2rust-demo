//! 029_unique_ptr 冒烟测试
//!
//! 模拟 unique_ptr 的独占缓冲；验证大小、use_count 与处理器加工结果。

use unique_ptr::*;

#[test]
fn smoke_unique_buffer() {
    let buffer = uniquebuffer_new(16);
    assert_eq!(buffer.get_size(), 16, "缓冲大小应为构造参数 16");
    assert_eq!(buffer.use_count(), 1, "unique_ptr 语义下 use_count 恒为 1");
}

#[test]
fn smoke_processor_process() {
    let mut processor = processor_new();
    let input = std::ffi::CString::new("Hello, unique_ptr!").unwrap();
    let result_ptr = processor.process(input.as_ptr());
    let result = unsafe {
        std::ffi::CStr::from_ptr(result_ptr as *const i8)
            .to_string_lossy()
            .into_owned()
    };
    assert_eq!(result, "Hello, unique_ptr! [processed]", "process 应追加 [processed] 后缀");
}
