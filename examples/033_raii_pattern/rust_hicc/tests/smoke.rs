//! 033_raii_pattern 冒烟测试
//!
//! 验证生成的 Rust FFI 绑定可编译、可链接 C++ 实现，且基本行为正确。

use raii_pattern::*;
use hicc::AbiClass;

#[test]
fn smoke_mutex_new() {
    let mutex = mutex_new();
    // 构造不崩溃即可
    drop(mutex);
}

#[test]
fn smoke_mutex_lock_unlock() {
    let mut mutex = mutex_new();
    mutex.lock();
    mutex.unlock();
    // lock/unlock 不崩溃即可
}

#[test]
fn smoke_mutex_try_lock() {
    let mut mutex = mutex_new();
    let result = mutex.try_lock();
    assert!(result, "无竞争的 try_lock 应返回 true");
    mutex.unlock();
}

#[test]
fn smoke_mutex_name() {
    use std::ffi::CStr;
    let mutex = mutex_new();
    let name_ptr = mutex.name();
    let name = unsafe { CStr::from_ptr(name_ptr) }
        .to_string_lossy()
        .into_owned();
    assert!(!name.is_empty(), "Mutex 名称不应为空");
}

#[test]
fn smoke_scoped_lock() {
    let mut mutex = mutex_new();
    let lock = unsafe { scoped_lock_new(&mutex.as_mut_ptr()) };
    assert!(lock.owns_lock(), "ScopedLock 构造后应拥有锁");
}

#[test]
fn smoke_file_lock_new() {
    let filename = std::ffi::CString::new("smoke_raii_test.txt").expect("CString::new failed");
    let file_lock = unsafe { file_lock_new(filename.as_ptr()) };
    // 构造不崩溃即可
    drop(file_lock);
}

#[test]
fn smoke_file_lock_filename() {
    use std::ffi::CStr;
    let filename = std::ffi::CString::new("smoke_raii_file.txt").expect("CString::new failed");
    let file_lock = unsafe { file_lock_new(filename.as_ptr()) };
    let fname = unsafe { CStr::from_ptr(file_lock.filename()) }
        .to_string_lossy()
        .into_owned();
    assert_eq!(fname, "smoke_raii_file.txt", "FileLock filename 应匹配");
}

#[test]
fn smoke_file_lock_lock_unlock() {
    let filename = std::ffi::CString::new("smoke_raii_lock.txt").expect("CString::new failed");
    let mut file_lock = unsafe { file_lock_new(filename.as_ptr()) };
    file_lock.lock();
    file_lock.unlock();
    // lock/unlock 不崩溃即可
}
