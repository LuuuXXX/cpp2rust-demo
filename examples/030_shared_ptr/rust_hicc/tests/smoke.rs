//! 030_shared_ptr 冒烟测试

use shared_ptr::*;

#[test]
fn smoke_shared_data_name() {
    let name = std::ffi::CString::new("TestData").expect("CString::new failed");
    let data = SharedData::new(name.as_ptr());
    let nm = unsafe { std::ffi::CStr::from_ptr(data.name()).to_string_lossy().into_owned() };
    assert_eq!(nm, "TestData");
}

#[test]
fn smoke_shared_data_use_count() {
    let name = std::ffi::CString::new("x").expect("CString::new failed");
    let data = SharedData::new(name.as_ptr());
    assert_eq!(data.use_count(), 1, "独立 SharedData 引用计数应为 1");
}

#[test]
fn smoke_shared_data_reset_expired() {
    let name = std::ffi::CString::new("x").expect("CString::new failed");
    let mut data = SharedData::new(name.as_ptr());
    assert_eq!(data.expired(), 0);
    data.reset();
    assert_eq!(data.expired(), 1, "reset 后应为已释放");
}

#[test]
fn smoke_cache_store() {
    let mut cache = Cache::new();
    let key = std::ffi::CString::new("key1").expect("CString::new failed");
    let count = cache.store(key.as_ptr());
    assert_eq!(count, 2, "缓存后引用计数应为 2");
    assert_eq!(cache.size(), 1);
}

#[test]
fn smoke_cache_clear() {
    let mut cache = Cache::new();
    let key = std::ffi::CString::new("key1").expect("CString::new failed");
    cache.store(key.as_ptr());
    cache.clear();
    assert_eq!(cache.size(), 0);
}
