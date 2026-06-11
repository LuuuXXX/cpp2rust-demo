use custom_deleter::*;

fn main() {
    println!("=== 031_custom_deleter - 自定义删除器 ===\n");

    // 使用默认删除器
    let filename = std::ffi::CString::new("test_default.txt").expect("CString::new failed");
    let mode = std::ffi::CString::new("w").expect("CString::new failed");

    let mut handle = unsafe { file_open_default(filename.as_ptr(), mode.as_ptr()) };

    // 写入数据
    let data = std::ffi::CString::new("Hello, custom deleter!").expect("CString::new failed");
    let written = handle.write(data.as_ptr(), data.to_bytes().len() as i32);
    println!("Written {} bytes", written);

    // 关闭文件
    handle.close_file();

    println!("\nRust FFI: 自定义删除器模式");
    println!("1. C++ 允许传递函数指针作为删除器");
    println!("2. 删除器在对象销毁时自动调用");
    println!("3. Rust 可以传入自己的清理函数");
    println!("4. 适用于文件、内存、网络连接等资源");
}
