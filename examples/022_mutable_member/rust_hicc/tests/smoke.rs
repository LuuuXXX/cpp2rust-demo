//! 022_mutable_member 冒烟测试
//!
//! mutable 在 FFI 中无影响；验证 getName 与 refresh 后的缓存计数。

use mutable_member::*;

fn decode_cstr(ptr: *const i8) -> String {
    unsafe { std::ffi::CStr::from_ptr(ptr).to_string_lossy().into_owned() }
}

#[test]
fn smoke_fetcher_name() {
    let name = std::ffi::CString::new("TestFetcher").unwrap();
    let fetcher = unsafe { datafetcher_new(name.as_ptr()) };
    assert_eq!(decode_cstr(fetcher.get_name()), "TestFetcher", "getName 应返回构造名");
}

#[test]
fn smoke_fetcher_refresh_increments_cache() {
    let name = std::ffi::CString::new("TestFetcher").unwrap();
    let mut fetcher = unsafe { datafetcher_new(name.as_ptr()) };
    assert_eq!(fetcher.get_cache_count(), 0, "初始缓存计数应为 0");
    fetcher.refresh();
    assert_eq!(fetcher.get_cache_count(), 1, "refresh 后缓存计数应为 1");
}
