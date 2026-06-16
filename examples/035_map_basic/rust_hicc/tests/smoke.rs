//! 035_map_basic 冒烟测试
//!
//! 验证生成的 Rust FFI 绑定可编译、可链接 C++ 实现，且关联容器操作行为正确。

use map_basic::*;
use std::ffi::{CStr, CString};

#[test]
fn smoke_string_int_map_insert_get_contains() {
    let mut m = StringIntMap::new();
    let apple = CString::new("apple").expect("CString::new failed");
    let banana = CString::new("banana").expect("CString::new failed");
    let missing = CString::new("missing").expect("CString::new failed");

    m.insert(apple.as_ptr(), 3);
    m.insert(banana.as_ptr(), 5);
    assert_eq!(m.size(), 2);
    assert_eq!(m.get(apple.as_ptr()), 3);
    assert_eq!(m.get(missing.as_ptr()), -1);
    assert_eq!(m.contains(banana.as_ptr()), 1);
    assert_eq!(m.contains(missing.as_ptr()), 0);
}

#[test]
fn smoke_string_int_map_overwrite_and_first_key() {
    let mut m = StringIntMap::new();
    let beta = CString::new("beta").expect("CString::new failed");
    let alpha = CString::new("alpha").expect("CString::new failed");

    m.insert(beta.as_ptr(), 2);
    m.insert(alpha.as_ptr(), 1);
    m.insert(beta.as_ptr(), 22);
    assert_eq!(m.size(), 2);
    assert_eq!(m.get(beta.as_ptr()), 22);

    let first = unsafe { CStr::from_ptr(m.first_key()).to_string_lossy().into_owned() };
    assert_eq!(first, "alpha");
}

#[test]
fn smoke_string_int_map_erase_clear() {
    let mut m = StringIntMap::new();
    let a = CString::new("a").expect("CString::new failed");
    let b = CString::new("b").expect("CString::new failed");
    m.insert(a.as_ptr(), 1);
    m.insert(b.as_ptr(), 2);

    assert_eq!(m.erase(a.as_ptr()), 1);
    assert_eq!(m.erase(a.as_ptr()), 0);
    assert_eq!(m.size(), 1);
    m.clear();
    assert_eq!(m.size(), 0);
}

#[test]
fn smoke_counter_counts_words() {
    let mut c = Counter::new();
    for word in ["rust", "cpp", "rust"] {
        let cw = CString::new(word).expect("CString::new failed");
        c.add(cw.as_ptr());
    }
    let rust = CString::new("rust").expect("CString::new failed");
    let cpp = CString::new("cpp").expect("CString::new failed");
    let missing = CString::new("missing").expect("CString::new failed");

    assert_eq!(c.count(rust.as_ptr()), 2);
    assert_eq!(c.count(cpp.as_ptr()), 1);
    assert_eq!(c.count(missing.as_ptr()), 0);
    assert_eq!(c.unique_words(), 2);
}

#[test]
fn smoke_counter_last_word_clear_and_anchor() {
    let mut c = Counter::new();
    let word = CString::new("last").expect("CString::new failed");
    c.add(word.as_ptr());

    let last = unsafe { CStr::from_ptr(c.last_word()).to_string_lossy().into_owned() };
    assert_eq!(last, "last");
    c.clear();
    assert_eq!(c.unique_words(), 0);
    assert_eq!(map_basic_anchor(), 0);
}
