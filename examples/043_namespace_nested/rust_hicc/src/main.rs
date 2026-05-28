// 043_namespace_nested - 嵌套命名空间
// 使用 raw extern "C" 模式，完全避开 hicc 宏

hicc::cpp! {
    // C++ implementation is in ../cpp/namespace_nested.cpp
    // Raw extern "C" declarations are used directly below
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



