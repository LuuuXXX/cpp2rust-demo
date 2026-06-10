hicc::cpp! {
    #include <iostream>
    #include <cstdio>
    #include <cstring>

    #include "custom_deleter.h"

    typedef void (*FileDeleter)(struct FileHandle*);
}

hicc::import_class! {
    #[cpp(class = "FileHandle", destroy = "refcounted_file_deleter")]
    pub class FileHandle {
        #[cpp(method = "bool is_open() const")]
        fn is_open(&self) -> bool;

        #[cpp(method = "int read(char* buffer, int size)")]
        fn read(&mut self, buffer: *mut i8, size: i32) -> i32;

        #[cpp(method = "int write(const char* data, int size)")]
        fn write(&mut self, data: *const i8, size: i32) -> i32;

        #[cpp(method = "const char* filename() const")]
        fn filename(&self) -> *const i8;

        #[cpp(method = "void close_file()")]
        fn close_file(&mut self);

        #[cpp(method = "void invoke_deleter()")]
        fn invoke_deleter(&mut self);
    }
}

hicc::import_lib! {
    #![link_name = "custom_deleter"]

    class FileHandle;

    // cpp2rust-todo[FP]: 含函数指针参数，需确保回调符合 extern "C" 调用约定
    #[cpp(func = "FileHandle* file_open(const char*, const char*, void (*)(FileHandle*))")]
    unsafe fn file_open(filename: *const i8, mode: *const i8, deleter: unsafe extern "C" fn(*mut FileHandle)) -> *mut FileHandle;

    #[cpp(func = "void file_close(FileHandle* handle)")]
    unsafe fn file_close(handle: *mut FileHandle);

    #[cpp(func = "int file_read(FileHandle* handle, char*, int)")]
    unsafe fn file_read(handle: *mut FileHandle, buffer: *mut i8, size: i32) -> i32;

    #[cpp(func = "int file_write(FileHandle* handle, const char*, int)")]
    unsafe fn file_write(handle: *mut FileHandle, data: *const i8, size: i32) -> i32;

    #[cpp(func = "FileHandle* file_open_default(const char*, const char*)")]
    unsafe fn file_open_default(filename: *const i8, mode: *const i8) -> *mut FileHandle;
}

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

