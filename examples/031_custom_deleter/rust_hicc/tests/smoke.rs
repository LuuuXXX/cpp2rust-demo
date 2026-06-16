//! 031_custom_deleter 冒烟测试
//!
//! 验证生成的 Rust FFI 绑定可编译、可链接 C++ 实现，且自定义删除器行为正确。

use custom_deleter::*;

#[test]
fn smoke_managed_resource_name() {
    let name = std::ffi::CString::new("logfile.txt").expect("CString::new failed");
    let res = ManagedResource::new(name.as_ptr());
    let nm = unsafe { std::ffi::CStr::from_ptr(res.name()).to_string_lossy().into_owned() };
    assert_eq!(nm, "logfile.txt");
}

#[test]
fn smoke_managed_resource_release() {
    let name = std::ffi::CString::new("x").expect("CString::new failed");
    let mut res = ManagedResource::new(name.as_ptr());
    assert_eq!(res.released(), 0);
    res.release();
    assert_eq!(res.released(), 1, "release 后应为已释放");
}

#[test]
fn smoke_custom_deleter_invoked() {
    let before = cleanup_count();
    {
        let name = std::ffi::CString::new("x").expect("CString::new failed");
        let _res = ManagedResource::new(name.as_ptr());
    } // 离开作用域，Rust Drop 触发 C++ 析构 → 自定义删除器被调用
    assert!(
        cleanup_count() > before,
        "对象析构时自定义删除器应被调用一次"
    );
}
