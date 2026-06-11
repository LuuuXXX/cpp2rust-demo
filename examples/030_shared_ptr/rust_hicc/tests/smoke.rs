//! 030_shared_ptr 冒烟测试
//!
//! 模拟 shared_ptr 共享数据与 weak_ptr 风格缓存；验证名称、克隆与缓存命中。

use shared_ptr::*;
use hicc::AbiClass;

fn decode_cstr(ptr: *const i8) -> String {
    unsafe { std::ffi::CStr::from_ptr(ptr).to_string_lossy().into_owned() }
}

#[test]
fn smoke_shared_data_name_and_clone() {
    let name = std::ffi::CString::new("TestData").unwrap();
    let data = unsafe { shareddata_new(name.as_ptr()) };
    assert_eq!(decode_cstr(data.get_name()), "TestData", "getName 应返回构造名");

    // clone() 返回共享同名数据的新实例。
    let cloned = data.clone();
    assert_eq!(decode_cstr(cloned.get_name()), "TestData", "克隆后名称应一致");
}

#[test]
fn smoke_cache_get_creates_entry() {
    let mut cache = cache_new();
    let key = std::ffi::CString::new("key1").unwrap();
    // 首次未命中则创建并以 key 命名；再次访问命中同一条目。
    let _first = unsafe { cache_get(&cache.as_mut_ptr(), key.as_ptr()) };
    let second = unsafe { cache_get(&cache.as_mut_ptr(), key.as_ptr()) };
    assert_eq!(decode_cstr(second.get_name()), "key1", "缓存条目应以 key 命名");
}
