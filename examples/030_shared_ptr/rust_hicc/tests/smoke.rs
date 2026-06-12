//! 030_shared_ptr 冒烟测试
//!
//! 验证生成的 Rust FFI 绑定可编译、可链接 C++ 实现，且基本行为正确。

use shared_ptr::*;
use hicc::AbiClass;

#[test]
fn smoke_shareddata_new() {
    let name = std::ffi::CString::new("TestData").expect("CString::new failed");
    let data = unsafe { shareddata_new(name.as_ptr()) };
    let name_str = unsafe {
        std::ffi::CStr::from_ptr(data.get_name())
            .to_string_lossy()
            .into_owned()
    };
    assert_eq!(name_str, "TestData", "SharedData 名称应为 TestData");
}

#[test]
fn smoke_shareddata_use_count() {
    let name = std::ffi::CString::new("TestData").expect("CString::new failed");
    let data = unsafe { shareddata_new(name.as_ptr()) };
    let count = data.use_count();
    assert!(count >= 1, "新建 SharedData 的 use_count 应 >= 1");
}

#[test]
fn smoke_shareddata_clone() {
    let name = std::ffi::CString::new("TestData").expect("CString::new failed");
    let data1 = unsafe { shareddata_new(name.as_ptr()) };

    // clone() 返回 *mut SharedData 原始指针
    let data2_ptr = data1.clone();
    assert!(!data2_ptr.is_null(), "clone 不应返回空指针");
    // clone 返回原始指针，验证非空即可
}

#[test]
fn smoke_shareddata_reset() {
    let name = std::ffi::CString::new("TestData").expect("CString::new failed");
    let mut data = unsafe { shareddata_new(name.as_ptr()) };
    data.reset();
    // reset 不崩溃即可
}

#[test]
fn smoke_cache_new() {
    let _cache = cache_new();
    // 构造不崩溃即可
}

#[test]
fn smoke_cache_get() {
    let mut cache = cache_new();
    let key = std::ffi::CString::new("key1").expect("CString::new failed");
    let ptr = unsafe { cache_get(&cache.as_mut_ptr(), key.as_ptr()) };
    assert!(!ptr.is_null(), "cache_get 应返回有效指针");
}
