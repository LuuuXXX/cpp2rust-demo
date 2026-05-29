hicc::cpp! {
    #include <string>
    #include <iostream>
    #include <memory>
    #include <cstring>
    #include <unordered_map>

    class SharedData {
        std::string name_;
    public:
        int value;
    public:
        SharedData(const char* n) : name_(n ? n : ""), value(0) {
}
        ~SharedData() {
}
        int useCount() const {
    return 1; // Simplified - actual shared_ptr would have ref count
}
        const char* getName() const {
    return name_.c_str();
}
        SharedData* clone() const {
    return new SharedData(name_.c_str());
}
        void reset() {
    name_.clear();
}
    };

    class Cache {
        std::unordered_map<std::string, void*> data_;
    public:
        Cache() : data_() {
}
        ~Cache() {
}
        SharedData* get(const char* name) {
    if (!name) return nullptr;
    std::string key(name);
    auto it = data_.find(key);
    if (it != data_.end()) {
        return reinterpret_cast<SharedData*>(it->second);
    }
    // If not found, create new and store
    SharedData* new_data = new SharedData(name);
    data_[key] = reinterpret_cast<void*>(new_data);
    return new_data;
}
    };

    SharedData* shareddata_new(const char* name) {
        return new SharedData(name);
    }

    void shareddata_delete(SharedData* self) {
        delete self;
    }

    Cache* cache_new() {
        return new Cache();
    }

    void cache_delete(Cache* self) {
        delete self;
    }
}

hicc::import_class! {
    #[cpp(class = "SharedData")]
    class SharedData {
        #[cpp(method = "int useCount() const")]
        fn use_count(&self) -> i32;

        #[cpp(method = "const char* getName() const")]
        fn get_name(&self) -> *const i8;

        #[cpp(method = "SharedData* clone() const")]
        fn clone(&self) -> *mut SharedData;

        #[cpp(method = "void reset()")]
        fn reset(&mut self);
    }
}

hicc::import_class! {
    #[cpp(class = "Cache")]
    class Cache {
        #[cpp(method = "SharedData* get(const char* name)")]
        fn get(&mut self, name: *const i8) -> *mut SharedData;
    }
}

hicc::import_lib! {
    #![link_name = "shared_ptr"]

    class SharedData;
    class Cache;

    #[cpp(func = "SharedData* shareddata_new(const char*)")]
    unsafe fn shareddata_new(name: *const i8) -> *mut SharedData;

    #[cpp(func = "void shareddata_delete(SharedData* self)")]
    unsafe fn shareddata_delete(self_: *mut SharedData);

    #[cpp(func = "Cache* cache_new()")]
    fn cache_new() -> *mut Cache;

    #[cpp(func = "void cache_delete(Cache* self)")]
    unsafe fn cache_delete(self_: *mut Cache);
}

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
    eprintln!("DEBUG: before cache_new");

    // Cache - 演示 weak_ptr 的作用（缓存）
    let mut cache = cache_new();
    eprintln!("DEBUG: after cache_new");

    let key1 = std::ffi::CString::new("key1").expect("CString::new failed");
    let key2 = std::ffi::CString::new("key2").expect("CString::new failed");
    eprintln!("DEBUG: before cache.get key1");
    // inspect raw Cache layout
    unsafe {
        let raw = &cache as *const _ as *const usize;
        eprintln!("cache[0] methods ptr = {:016x}", *raw);
        eprintln!("cache[1] obj ptr     = {:016x}", *raw.add(1));
        eprintln!("cache[2] level       = {}", *raw.add(2));
        let methods = *raw as *const usize;
        eprintln!("methods[0] destroy  = {:016x}", *methods);
        eprintln!("methods[1] unique   = {:016x}", *methods.add(1));
        eprintln!("methods[2] make_ref = {:016x}", *methods.add(2));
        eprintln!("methods[3] size_of  = {:016x}", *methods.add(3));
        eprintln!("methods[4] write    = {:016x}", *methods.add(4));
        eprintln!("methods[5] get      = {:016x}", *methods.add(5));
    }
    let cached1a = cache.get(key1.as_ptr());
    eprintln!("DEBUG: after cache.get key1 (cached1a)");
    let cached1b = cache.get(key1.as_ptr());  // 缓存命中
    eprintln!("DEBUG: after cache.get key1 (cached1b)");
    let _cached2 = cache.get(key2.as_ptr());
    eprintln!("DEBUG: after cache.get key2");

    println!("\nCache demo:");
    println!("cached1a and cached1b point to same cache entry");

    unsafe { cache_delete(&cache) };

    println!("\nRust FFI: shared_ptr 的处理方式");
    println!("1. C++ 侧管理引用计数");
    println!("2. Rust 侧通过 FFI 函数操作");
    println!("3. 相当于 Rust 的 Arc<T>");
    println!("\nweak_ptr 用于缓存，避免循环引用");
    println!("相当于 Rust 的 Weak<T>");
}



