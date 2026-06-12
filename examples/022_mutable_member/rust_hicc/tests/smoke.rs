//! 022_mutable_member 冒烟测试
//!
//! 验证生成的 Rust FFI 绑定可编译、可链接 C++ 实现，且基本行为正确。

use mutable_member::*;
use std::ffi::CStr;

#[test]
fn smoke_datafetcher_create() {
    let name = std::ffi::CString::new("TestFetcher").expect("CString::new failed");
    let fetcher = unsafe { datafetcher_new(name.as_ptr()) };
    let got_name = unsafe { CStr::from_ptr(fetcher.get_name()) }
        .to_string_lossy()
        .to_string();
    assert_eq!(got_name, "TestFetcher", "DataFetcher 名称应为 'TestFetcher'");
}

#[test]
fn smoke_datafetcher_initial_cache_count() {
    let name = std::ffi::CString::new("Fetcher1").expect("CString::new failed");
    let fetcher = unsafe { datafetcher_new(name.as_ptr()) };
    assert_eq!(fetcher.get_cache_count(), 0, "初始 cache_count 应为 0");
}

#[test]
fn smoke_datafetcher_refresh_increments() {
    let name = std::ffi::CString::new("Fetcher2").expect("CString::new failed");
    let mut fetcher = unsafe { datafetcher_new(name.as_ptr()) };
    assert_eq!(fetcher.get_cache_count(), 0, "初始 cache_count 应为 0");
    fetcher.refresh();
    assert_eq!(fetcher.get_cache_count(), 1, "refresh 一次后 cache_count 应为 1");
    fetcher.refresh();
    assert_eq!(fetcher.get_cache_count(), 2, "refresh 两次后 cache_count 应为 2");
    fetcher.refresh();
    assert_eq!(fetcher.get_cache_count(), 3, "refresh 三次后 cache_count 应为 3");
}
