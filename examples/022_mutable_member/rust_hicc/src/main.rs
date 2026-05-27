hicc::cpp! {
    #include <iostream>
    #include <cstring>

    class DataFetcher {
        const char* name;
        mutable int cache_count;
        char cache_data[256];
    public:
        DataFetcher(const char* n) : cache_count(0) {
    name = n;
}
        ~DataFetcher() {}
        const char* getName() const { return name; }
        int getCacheCount() const { return cache_count; }
        void refresh() { cache_count++; }
    };

    DataFetcher* datafetcher_new(const char* name) {
        return new DataFetcher(name);
    }

    void datafetcher_delete(DataFetcher* self) {
        delete self;
    }
}

hicc::import_class! {
    #[cpp(class = "DataFetcher")]
    class DataFetcher {
        #[cpp(method = "const char* getName() const")]
        fn get_name(&self) -> *const u8;

        #[cpp(method = "int getCacheCount() const")]
        fn get_cache_count(&self) -> i32;

        #[cpp(method = "void refresh()")]
        fn refresh(&mut self);
    }
}

hicc::import_lib! {
    #![link_name = "mutable_member"]

    class DataFetcher;

    #[cpp(func = "DataFetcher* datafetcher_new(const char*)")]
    unsafe fn datafetcher_new(name: *const i8) -> *mut DataFetcher;

    #[cpp(func = "void datafetcher_delete(DataFetcher* self)")]
    unsafe fn datafetcher_delete(self_: *mut DataFetcher);
}

fn main() {
    println!("=== 022_mutable_member - mutable 成员 ===\n");

    let name = std::ffi::CString::new("TestFetcher").expect("CString::new failed");
    let fetcher = datafetcher_new(name.as_ptr());

    println!("Calling getName() 3 times (const method with mutable cache):");
    for i in 0..3 {
        let count = datafetcher_getCacheCount(&fetcher);
        println!("  Call {}: name = {}, cache_count = {}", i + 1, i, count);
    }

    println!("\nRefreshing...");
    datafetcher_refresh(&fetcher);
    println!("Cache count after refresh: {}", datafetcher_getCacheCount(&fetcher));

    unsafe { datafetcher_delete(&fetcher) };

    println!("\nRust FFI: mutable 关键字在 FFI 中无影响");
    println!("mutable 只影响 C++ 编译器允许在 const 方法中修改该成员");
}



