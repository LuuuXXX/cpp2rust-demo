hicc::cpp! {
    #include <stddef.h>
    #include <string>
    #include <iostream>
    #include <thread>
    #include <mutex>
    #include <fstream>
    #include <cstring>

    #include "raii_pattern.h"
}

hicc::import_class! {
    #[cpp(class = "Mutex", destroy = "mutex_delete")]
    class Mutex {
        #[cpp(method = "void lock()")]
        fn lock(&mut self);

        #[cpp(method = "void unlock()")]
        fn unlock(&mut self);

        #[cpp(method = "bool try_lock()")]
        fn try_lock(&mut self) -> bool;

        #[cpp(method = "const char* name() const")]
        fn name(&self) -> *const i8;
    }
}

hicc::import_class! {
    #[cpp(class = "ScopedLock", destroy = "scoped_lock_delete")]
    class ScopedLock {
        #[cpp(method = "bool owns_lock() const")]
        fn owns_lock(&self) -> bool;
    }
}

hicc::import_class! {
    #[cpp(class = "FileLock", destroy = "file_lock_delete")]
    class FileLock {
        #[cpp(method = "void lock()")]
        fn lock(&mut self);

        #[cpp(method = "void unlock()")]
        fn unlock(&mut self);

        #[cpp(method = "const char* filename() const")]
        fn filename(&self) -> *const i8;
    }
}

hicc::import_lib! {
    #![link_name = "raii_pattern"]

    class Mutex;
    class ScopedLock;
    class FileLock;

    #[cpp(func = "Mutex* mutex_new()")]
    fn mutex_new() -> Mutex;

    #[cpp(func = "ScopedLock* scoped_lock_new(Mutex* mutex)")]
    unsafe fn scoped_lock_new(mutex: *mut Mutex) -> ScopedLock;

    #[cpp(func = "FileLock* file_lock_new(const char*)")]
    unsafe fn file_lock_new(filename: *const i8) -> FileLock;
}

fn main() {
    println!("=== 033_raii_pattern - RAII 模式 ===\n");

    // 手动锁定/解锁示例
    println!("--- Manual Lock/Unlock ---");
    let mut mutex = unsafe { mutex_new() };
    mutex.lock();
    println!("Critical section started");
    println!("Critical section ended");
    mutex.unlock();
    unsafe { mutex_delete(&mutex) };

    println!();

    // ScopedLock 示例（模拟 RAII 自动解锁）
    println!("--- ScopedLock Demo ---");
    let mutex2 = unsafe { mutex_new() };
    let lock = unsafe { scoped_lock_new(&mutex2) };
    println!("Inside scoped lock region");
    println!("ScopedLock will auto-unlock on delete");
    unsafe { scoped_lock_delete(&lock) };
    unsafe { mutex_delete(&mutex2) };

    println!();

    // FileLock 示例
    println!("--- FileLock Demo ---");
    let filename = std::ffi::CString::new("raii_test.txt").expect("CString::new failed");
    let mut file_lock = unsafe { file_lock_new(filename.as_ptr()) };
    file_lock.lock();
    println!("File is locked, performing I/O...");
    file_lock.unlock();
    unsafe { file_lock_delete(&file_lock) };

    println!("\nRust FFI: RAII 模式映射");
    println!("1. C++ RAII: 构造函数加锁，析构函数解锁");
    println!("2. Rust 等效: Drop trait 自动调用");
    println!("3. FFI 边界: ScopedLock 对象在 Rust 析构时自动释放");
    println!("4. 推荐模式: Rust 封装 RAII guard 类型");
}

