//! 031_custom_deleter 冒烟测试
//!
//! 验证生成的 Rust FFI 绑定可编译、可链接 C++ 实现，且基本行为正确。

use custom_deleter::*;
use std::ffi::CStr;

#[test]
fn smoke_file_open_default() {
    let filename = std::ffi::CString::new("smoke_test.txt").expect("CString::new failed");
    let mode = std::ffi::CString::new("w").expect("CString::new failed");
    let handle = unsafe { file_open_default(filename.as_ptr(), mode.as_ptr()) };
    assert!(handle.is_open(), "新打开的文件应为 open 状态");
}

#[test]
fn smoke_file_handle_filename() {
    let filename = std::ffi::CString::new("smoke_filename.txt").expect("CString::new failed");
    let mode = std::ffi::CString::new("w").expect("CString::new failed");
    let handle = unsafe { file_open_default(filename.as_ptr(), mode.as_ptr()) };
    let fname = unsafe { CStr::from_ptr(handle.filename()) }
        .to_string_lossy()
        .into_owned();
    assert_eq!(fname, "smoke_filename.txt", "filename 应匹配");
}

#[test]
fn smoke_file_write() {
    let filename = std::ffi::CString::new("smoke_write.txt").expect("CString::new failed");
    let mode = std::ffi::CString::new("w").expect("CString::new failed");
    let mut handle = unsafe { file_open_default(filename.as_ptr(), mode.as_ptr()) };
    let data = std::ffi::CString::new("Hello, smoke!").expect("CString::new failed");
    let written = handle.write(data.as_ptr(), data.to_bytes().len() as i32);
    assert_eq!(
        written,
        data.to_bytes().len() as i32,
        "写入字节数应匹配"
    );
}

#[test]
fn smoke_file_close() {
    let filename = std::ffi::CString::new("smoke_close.txt").expect("CString::new failed");
    let mode = std::ffi::CString::new("w").expect("CString::new failed");
    let mut handle = unsafe { file_open_default(filename.as_ptr(), mode.as_ptr()) };
    handle.close_file();
    // close_file 不崩溃即可
}

#[test]
fn smoke_file_open_with_custom_deleter() {
    extern "C" fn my_deleter(_handle: *mut FileHandle) {
        // 自定义删除器 - 不做操作
    }

    let filename = std::ffi::CString::new("smoke_custom.txt").expect("CString::new failed");
    let mode = std::ffi::CString::new("w").expect("CString::new failed");
    let _handle = unsafe { file_open(filename.as_ptr(), mode.as_ptr(), my_deleter) };
    // 构造不崩溃即可
}
