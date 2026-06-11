//! 035_map_basic 冒烟测试
//!
//! 验证生成的 Rust FFI 绑定可编译、可链接 C++ 实现，且基本行为正确。

use map_basic::*;
use std::ffi::CString;

#[test]
fn smoke_string_int_map_new() {
    let map = string_int_map_new();
    assert!(map.empty(), "新建 StringIntMap 应为空");
    assert_eq!(map.size(), 0, "新建 StringIntMap size 应为 0");
}

#[test]
fn smoke_string_int_map_insert_get() {
    let mut map = string_int_map_new();
    let key = CString::new("hello").unwrap();
    let inserted = map.insert(key.as_ptr(), 42);
    assert!(inserted, "首次插入应成功");
    assert_eq!(map.size(), 1, "插入后 size 应为 1");
    let val = map.get(key.as_ptr());
    assert_eq!(val, 42, "get 应返回插入的值");
}

#[test]
fn smoke_string_int_map_set_overwrite() {
    let mut map = string_int_map_new();
    let key = CString::new("key").unwrap();
    map.insert(key.as_ptr(), 10);
    map.set(key.as_ptr(), 99);
    let val = map.get(key.as_ptr());
    assert_eq!(val, 99, "set 后应覆盖原值");
}

#[test]
fn smoke_string_int_map_erase() {
    let mut map = string_int_map_new();
    let key = CString::new("erase_me").unwrap();
    map.insert(key.as_ptr(), 1);
    assert_eq!(map.size(), 1);
    let erased = map.erase(key.as_ptr());
    assert!(erased, "删除存在的 key 应返回 true");
    assert!(map.empty(), "erase 后应为空");
}

#[test]
fn smoke_string_int_map_clear() {
    let mut map = string_int_map_new();
    let k1 = CString::new("a").unwrap();
    let k2 = CString::new("b").unwrap();
    map.insert(k1.as_ptr(), 1);
    map.insert(k2.as_ptr(), 2);
    map.clear();
    assert!(map.empty(), "clear 后应为空");
}

#[test]
fn smoke_int_string_map_type_available() {
    // 验证 IntStringMap 类型可用（编译期类型检查）
    let _ = int_string_map_new();
}
