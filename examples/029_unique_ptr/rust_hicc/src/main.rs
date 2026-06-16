use unique_ptr::*;

fn main() {
    println!("=== 029_unique_ptr - std::unique_ptr（hicc 直出）===\n");

    // UniqueBuffer 由 hicc unique_ptr 独占持有，析构自动完成。
    let mut buffer = UniqueBuffer::new(16);
    let size = buffer.size();
    println!("Buffer size: {}", size);

    buffer.fill(b'A' as i8);
    let data_ptr = buffer.data();
    let slice = unsafe { std::slice::from_raw_parts(data_ptr as *const u8, size as usize) };
    let data_str: String = slice.iter().map(|&c| c as char).collect();
    println!("Buffer data: {}", data_str);

    println!("Use count: {} (unique_ptr 恒为 1)", buffer.use_count());

    println!();

    // Processor
    let mut processor = Processor::new();
    let input = std::ffi::CString::new("Hello, unique_ptr!").expect("CString::new failed");
    let result_ptr = processor.process(input.as_ptr());
    let result = unsafe {
        std::ffi::CStr::from_ptr(result_ptr)
            .to_string_lossy()
            .into_owned()
    };
    println!("Processed result: {}", result);

    println!("\nRust FFI: hicc 用 unique_ptr 管理 C++ 对象所有权");
    println!("析构由 Rust Drop 自动触发，相当于 Box<T>");
}
