use raii_pattern::*;

use hicc::AbiClass;

fn main() {
    println!("=== 033_raii_pattern - RAII 模式 ===\n");

    // 手动锁定/解锁示例
    println!("--- Manual Lock/Unlock ---");
    let mut mutex = unsafe { mutex_new().into_unique() };
    mutex.lock();
    println!("Critical section started");
    println!("Critical section ended");
    mutex.unlock();
    drop(mutex);

    println!();

    // ScopedLock 示例（模拟 RAII 自动解锁）
    println!("--- ScopedLock Demo ---");
    let mut mutex2 = unsafe { mutex_new().into_unique() };
    let lock = unsafe { scoped_lock_new(&mutex2.as_mut_ptr()).into_unique() };
    println!("Inside scoped lock region");
    println!("ScopedLock will auto-unlock on delete");
    drop(lock);
    drop(mutex2);

    println!();

    // FileLock 示例
    println!("--- FileLock Demo ---");
    let filename = std::ffi::CString::new("raii_test.txt").expect("CString::new failed");
    let mut file_lock = unsafe { file_lock_new(filename.as_ptr()).into_unique() };
    file_lock.lock();
    println!("File is locked, performing I/O...");
    file_lock.unlock();

    println!("\nRust FFI: RAII 模式映射");
    println!("1. C++ RAII: 构造函数加锁，析构函数解锁");
    println!("2. Rust 等效: Drop trait 自动调用");
    println!("3. FFI 边界: ScopedLock 对象在 Rust 析构时自动释放");
    println!("4. 推荐模式: Rust 封装 RAII guard 类型");
}
