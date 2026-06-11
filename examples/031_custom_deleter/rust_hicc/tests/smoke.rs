//! 031_custom_deleter 冒烟测试
//!
//! 自定义删除器在对象销毁时被调用；验证默认删除器下的打开/写入/文件名。

use custom_deleter::*;

fn decode_cstr(ptr: *const i8) -> String {
    unsafe { std::ffi::CStr::from_ptr(ptr).to_string_lossy().into_owned() }
}

#[test]
fn smoke_file_open_default_write() {
    let path = std::env::temp_dir().join("cpp2rust_031_smoke.txt");
    let filename = std::ffi::CString::new(path.to_str().unwrap()).unwrap();
    let mode = std::ffi::CString::new("w").unwrap();

    let mut handle = unsafe { file_open_default(filename.as_ptr(), mode.as_ptr()) };
    assert!(handle.is_open(), "默认删除器打开文件应成功");
    assert_eq!(decode_cstr(handle.filename()), path.to_str().unwrap(), "filename 应返回打开时的路径");

    let data = std::ffi::CString::new("Hello, custom deleter!").unwrap();
    let len = data.to_bytes().len() as i32;
    let written = handle.write(data.as_ptr(), len);
    assert_eq!(written, len, "write 应返回写入的字节数");

    handle.close_file();
    let _ = std::fs::remove_file(&path);
}
