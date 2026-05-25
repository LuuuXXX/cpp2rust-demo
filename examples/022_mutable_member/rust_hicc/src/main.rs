hicc::cpp! {
    #include <iostream>
    #include <cstring>

    class DataFetcher {
        const char* name;
        mutable int cache_count;
        char cache_data[256];
    public:
        DataFetcher(const char* n);
        ~DataFetcher();
        const char* getName() const;
        int getCacheCount() const;
        void refresh();
    };

    DataFetcher* datafetcher_new(const char* name) {
        return new DataFetcher(name);
    }

    void datafetcher_delete(DataFetcher* self) {
        delete self;
    }

    const char* datafetcher_getName(DataFetcher* self) {
        return self->getName();
    }

    int datafetcher_getCacheCount(DataFetcher* self) {
        return self->getCacheCount();
    }

    void datafetcher_refresh(DataFetcher* self) {
        self->refresh();
    }

    DataFetcher::DataFetcher(const char* n) : cache_count(0) {
        name = n;
    }
    DataFetcher::~DataFetcher() {}
    const char* DataFetcher::getName() const { return name; }
    int DataFetcher::getCacheCount() const { return cache_count; }
    void DataFetcher::refresh() { cache_count++; }
}

hicc::import_class! {
    #[cpp(class = "DataFetcher")]
    class DataFetcher {
        #[cpp(method = "const char* getName() const")]
        fn getName(&self) -> *const i8;

        #[cpp(method = "int getCacheCount() const")]
        fn getCacheCount(&self) -> i32;

        #[cpp(method = "void refresh()")]
        fn refresh(&mut self);
    }
}

hicc::import_lib! {
    #![link_name = "mutable_member"]

    class DataFetcher;

    #[cpp(func = "DataFetcher* datafetcher_new(const char* name)")]
    fn datafetcher_new(name: *const i8) -> *mut DataFetcher;

    #[cpp(func = "void datafetcher_delete(DataFetcher* self)")]
    unsafe fn datafetcher_delete(self_: *mut DataFetcher);

    #[cpp(func = "const char* datafetcher_getName(DataFetcher* self)")]
    fn datafetcher_getName(self_: *mut DataFetcher) -> *const i8;

    #[cpp(func = "int datafetcher_getCacheCount(DataFetcher* self)")]
    fn datafetcher_getCacheCount(self_: *mut DataFetcher) -> i32;

    #[cpp(func = "void datafetcher_refresh(DataFetcher* self)")]
    fn datafetcher_refresh(self_: *mut DataFetcher);
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
