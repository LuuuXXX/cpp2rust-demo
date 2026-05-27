// 043_namespace_nested - 嵌套命名空间
// 使用 raw extern "C" 模式，完全避开 hicc 宏

hicc::cpp! {
    #include <cstddef>
    #include <iostream>
    #include <cstring>

    class config_ConfigManager {
        int values_[MAX_ENTRIES];
        const char* keys_[MAX_ENTRIES];
        size_t count_;
    public:
        static constexpr size_t MAX_ENTRIES = 10;
    public:
        ConfigManager() = default;
        ~ConfigManager() = default;
        void set_value(const char* key, int value) {}
        int get_value(const char* key) const {}
    };

    const size_t config_ConfigManager::MAX_ENTRIES;

    class baz_DataProcessor {
        int multiplier_;
    public:
        DataProcessor() = default;
        ~DataProcessor() = default;
        int process(int input) const {}
    };

    void* config_manager_new() {
        return new foo::bar::config::ConfigManager();
    }

    void config_manager_delete(void* self) {
        if (self) {
            delete static_cast<foo::bar::config::ConfigManager*>(self);
        }
    }

    void config_manager_set_value(void* self, const char* key, int value) {
        if (self) {
            static_cast<foo::bar::config::ConfigManager*>(self)->set_value(key, value);
        }
    }

    int config_manager_get_value(void* self, const char* key) {
        if (self) {
            return static_cast<foo::bar::config::ConfigManager*>(self)->get_value(key);
        }
        return 0;
    }

    int string_length(const char* str) {
        if (!str) return 0;
        return strlen(str);
    }

    void* data_processor_new() {
        return new foo::baz::DataProcessor();
    }

    void data_processor_delete(void* self) {
        if (self) {
            delete static_cast<foo::baz::DataProcessor*>(self);
        }
    }

    int data_processor_process(void* self, int input) {
        if (self) {
            return static_cast<foo::baz::DataProcessor*>(self)->process(input);
        }
        return input;
    }

    const char* get_version() {
        return "1.0.0";
    }

    int get_build_number() {
        return 42;
    }
}

hicc::import_class! {
    #[cpp(class = "config_ConfigManager")]
    class config_ConfigManager {
        #[cpp(method = "void set_value(const char* key, int value)")]
        fn set_value(&mut self, key: *const u8, value: i32);

        #[cpp(method = "int get_value(const char* key) const")]
        fn get_value(&self, key: *const u8) -> i32;
    }
}

hicc::import_class! {
    #[cpp(class = "baz_DataProcessor")]
    class baz_DataProcessor {
        #[cpp(method = "int process(int input) const")]
        fn process(&self, input: i32) -> i32;
    }
}

hicc::import_lib! {
    #![link_name = "namespace_nested"]

    #[cpp(func = "void* config_manager_new()")]
    fn config_manager_new() -> *mut void;

    #[cpp(func = "void config_manager_delete(void*)")]
    unsafe fn config_manager_delete(self_: *mut void);

    #[cpp(func = "void config_manager_set_value(void*, const char*, int)")]
    unsafe fn config_manager_set_value(self_: *mut void, key: *const i8, value: i32);

    #[cpp(func = "int config_manager_get_value(void*, const char*)")]
    unsafe fn config_manager_get_value(self_: *mut void, key: *const i8) -> i32;

    #[cpp(func = "int string_length(const char*)")]
    unsafe fn string_length(str: *const i8) -> i32;

    #[cpp(func = "void* data_processor_new()")]
    fn data_processor_new() -> *mut void;

    #[cpp(func = "void data_processor_delete(void*)")]
    unsafe fn data_processor_delete(self_: *mut void);

    #[cpp(func = "int data_processor_process(void*, int)")]
    unsafe fn data_processor_process(self_: *mut void, input: i32) -> i32;

    #[cpp(func = "const char* get_version()")]
    unsafe fn get_version() -> *const i8;

    #[cpp(func = "int get_build_number()")]
    fn get_build_number() -> i32;
}

// 使用 opaque pointer 别名
type ConfigManager = *mut std::ffi::c_void;
type DataProcessor = *mut std::ffi::c_void;

// 直接使用 extern "C" 声明，不通过 hicc 宏
#[link(name = "namespace_nested")]
unsafe extern "C" {
    fn config_manager_new() -> ConfigManager;
    fn config_manager_delete(p: ConfigManager);
    fn config_manager_set_value(p: ConfigManager, key: *const i8, value: i32);
    fn config_manager_get_value(p: ConfigManager, key: *const i8) -> i32;
    fn data_processor_new() -> DataProcessor;
    fn data_processor_delete(p: DataProcessor);
    fn data_processor_process(p: DataProcessor, input: i32) -> i32;
    fn string_length(s: *const i8) -> i32;
    fn get_version() -> *const i8;
    fn get_build_number() -> i32;
}

fn main() {
    println!("=== 043_namespace_nested - 嵌套命名空间 ===\n");

    println!("--- foo::bar::config::ConfigManager ---");
    let config = unsafe { config_manager_new() };
    unsafe {
        config_manager_set_value(config, "timeout\0".as_ptr() as *const i8, 30);
        config_manager_set_value(config, "retry\0".as_ptr() as *const i8, 3);
        config_manager_set_value(config, "port\0".as_ptr() as *const i8, 8080);
    }

    println!("timeout = {}", unsafe { config_manager_get_value(config, "timeout\0".as_ptr() as *const i8) });
    println!("retry = {}", unsafe { config_manager_get_value(config, "retry\0".as_ptr() as *const i8) });
    println!("port = {}", unsafe { config_manager_get_value(config, "port\0".as_ptr() as *const i8) });
    unsafe { config_manager_delete(config); }

    println!("\n--- string_length ---");
    let test_str = "Hello, World!\0";
    let len = unsafe { string_length(test_str.as_ptr() as *const i8) };
    println!("string_length(\"{}\") = {}", &test_str[..len as usize], len);

    println!("\n--- foo::baz::DataProcessor ---");
    let processor = unsafe { data_processor_new() };
    println!("process(42) = {}", unsafe { data_processor_process(processor, 42) });
    unsafe { data_processor_delete(processor); }

    println!("\n--- Top-level Functions ---");
    let version = unsafe { get_version() };
    println!("version = {}", unsafe { std::ffi::CStr::from_ptr(version).to_str().unwrap() });
    println!("build_number = {}", unsafe { get_build_number() });

    println!("\n--- 总结 ---");
    println!("1. C++ 嵌套命名空间：foo::bar::config");
    println!("2. 命名空间影响符号名称");
    println!("3. FFI 声明使用完全限定名称");
    println!("4. Rust 端使用 opaque pointer 模式");
    println!("5. hicc import_class! 不支持嵌套命名空间，使用 raw extern \"C\"");
}


