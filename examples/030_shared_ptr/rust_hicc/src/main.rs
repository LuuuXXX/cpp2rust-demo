use shared_ptr::*;

use hicc::AbiClass;

fn main() {
    println!("=== 030_shared_ptr - std::shared_ptr + weak_ptr ===\n");

    // SharedData - 模拟 shared_ptr
    let name = std::ffi::CString::new("TestData").expect("CString::new failed");
    let mut data1 = unsafe { shareddata_new(name.as_ptr()) };

    println!("Created SharedData: {}", unsafe {
        std::ffi::CStr::from_ptr(data1.get_name()).to_string_lossy().into_owned()
    });
    println!("Use count: {}", data1.use_count());

    // Clone - 共享所有权
    let data2 = data1.clone();
    println!("\nCloned SharedData: {}", unsafe {
        std::ffi::CStr::from_ptr(data2.get_name()).to_string_lossy().into_owned()
    });
    println!("Use count (shared): {}", data1.use_count());

    // Reset
    data1.reset();
    println!("\nAfter reset, data1 is cleared");

    println!();

    // Cache - 演示 weak_ptr 的作用（缓存）
    let mut cache = cache_new();

    let key1 = std::ffi::CString::new("key1").expect("CString::new failed");
    let key2 = std::ffi::CString::new("key2").expect("CString::new failed");
    let _cached1a = unsafe { cache_get(&cache.as_mut_ptr(), key1.as_ptr()) };
    let _cached1b = unsafe { cache_get(&cache.as_mut_ptr(), key1.as_ptr()) };  // 缓存命中
    let _cached2 = unsafe { cache_get(&cache.as_mut_ptr(), key2.as_ptr()) };

    println!("\nCache demo:");
    println!("cached1a and cached1b point to same cache entry");

    println!("\nRust FFI: shared_ptr 的处理方式");
    println!("1. C++ 侧管理引用计数");
    println!("2. Rust 侧通过 FFI 函数操作");
    println!("3. 相当于 Rust 的 Arc<T>");
    println!("\nweak_ptr 用于缓存，避免循环引用");
    println!("相当于 Rust 的 Weak<T>");
}
