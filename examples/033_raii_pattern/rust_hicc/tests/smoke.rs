//! 033_raii_pattern 冒烟测试
//!
//! RAII：构造即加锁、析构即解锁；验证 Mutex/ScopedLock/FileLock 行为。

use raii_pattern::*;
use hicc::AbiClass;

fn decode_cstr(ptr: *const i8) -> String {
    unsafe { std::ffi::CStr::from_ptr(ptr).to_string_lossy().into_owned() }
}

#[test]
fn smoke_mutex_name_and_lock_cycle() {
    let mut mutex = mutex_new();
    assert_eq!(decode_cstr(mutex.name()), "unnamed", "默认构造的 Mutex 名应为 unnamed");
    mutex.lock();
    mutex.unlock();
}

#[test]
fn smoke_mutex_try_lock() {
    let mut mutex = mutex_new();
    assert!(mutex.try_lock(), "新建 Mutex 的 try_lock 应成功");
    mutex.unlock();
}

#[test]
fn smoke_scoped_lock_owns() {
    let mut mutex = mutex_new();
    let lock = unsafe { scoped_lock_new(&mutex.as_mut_ptr()) };
    assert!(lock.owns_lock(), "ScopedLock 构造后应持有锁");
}

#[test]
fn smoke_file_lock_filename() {
    let path = std::env::temp_dir().join("cpp2rust_033_smoke.txt");
    let fname = std::ffi::CString::new(path.to_str().unwrap()).unwrap();
    let file_lock = unsafe { file_lock_new(fname.as_ptr()) };
    assert_eq!(decode_cstr(file_lock.filename()), path.to_str().unwrap(), "filename 应返回构造路径");
    let _ = std::fs::remove_file(&path);
}
