use shared_ptr::*;

fn main() {
    println!("=== 030_shared_ptr - std::shared_ptr（hicc 直出）===\n");

    let name = std::ffi::CString::new("TestData").expect("CString::new failed");
    let mut data = SharedData::new(name.as_ptr());

    let nm = unsafe { std::ffi::CStr::from_ptr(data.name()).to_string_lossy().into_owned() };
    println!("name={} use_count={} expired={}", nm, data.use_count(), data.expired());

    data.reset();
    println!("after reset expired={}", data.expired());

    println!();

    let mut cache = Cache::new();
    let key1 = std::ffi::CString::new("key1").expect("CString::new failed");
    let key2 = std::ffi::CString::new("key2").expect("CString::new failed");
    let c1 = cache.store(key1.as_ptr());
    let c2 = cache.store(key2.as_ptr());
    println!("store use_count={},{} size={}", c1, c2, cache.size());
    cache.clear();
    println!("after clear size={}", cache.size());

    println!("\nRust FFI: hicc 用 shared_ptr 表达共享所有权，相当于 Rust 的 Arc<T>");
}
