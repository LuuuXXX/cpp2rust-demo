use mutable_member::*;

fn decode_cstr(ptr: *const i8) -> String {
    unsafe { std::ffi::CStr::from_ptr(ptr).to_string_lossy().into_owned() }
}

fn main() {
    println!("=== 022_mutable_member - mutable 成员 ===\n");

    let name = std::ffi::CString::new("TestFetcher").expect("CString::new failed");
    let mut fetcher = unsafe { data_fetcher_new_with_n(name.as_ptr()) };

    println!("Calling getName() 3 times (const method with mutable cache):");
    for i in 0..3 {
        let name_str = decode_cstr(fetcher.get_name());
        let count = fetcher.get_cache_count();
        println!("  Call {}: name = {}, cache_count = {}", i + 1, name_str, count);
    }

    println!("\nRefreshing...");
    fetcher.refresh();
    println!("Cache count after refresh: {}", fetcher.get_cache_count());

    println!("\nRust FFI: mutable 关键字在 FFI 中无影响");
    println!("mutable 只影响 C++ 编译器允许在 const 方法中修改该成员");
}
