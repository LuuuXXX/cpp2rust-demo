use namespace_nested::*;

fn main() {
    println!("=== 043_namespace_nested - 嵌套命名空间 ===\n");

    println!("--- foo::bar::config::ConfigManager ---");
    let config = config_manager_new();
    unsafe {
        config_manager_set_value(config, "timeout\0".as_ptr() as *const i8, 30);
        config_manager_set_value(config, "retry\0".as_ptr() as *const i8, 3);
        config_manager_set_value(config, "port\0".as_ptr() as *const i8, 8080);
    }

    println!("timeout = {}", unsafe { config_manager_get_value(config, "timeout\0".as_ptr() as *const i8) });
    println!("retry = {}", unsafe { config_manager_get_value(config, "retry\0".as_ptr() as *const i8) });
    println!("port = {}", unsafe { config_manager_get_value(config, "port\0".as_ptr() as *const i8) });

    println!("\n--- string_length ---");
    let test_str = "Hello, World!\0";
    let len = unsafe { string_length(test_str.as_ptr() as *const i8) };
    println!("string_length(\"{}\") = {}", &test_str[..len as usize], len);

    println!("\n--- foo::baz::DataProcessor ---");
    let processor = data_processor_new();
    println!("process(42) = {}", unsafe { data_processor_process(processor, 42) });

    println!("\n--- Top-level Functions ---");
    let version = unsafe { get_version() };
    println!("version = {}", unsafe { std::ffi::CStr::from_ptr(version).to_str().unwrap() });
    println!("build_number = {}", get_build_number());

    println!("\n--- 总结 ---");
    println!("1. C++ 嵌套命名空间：foo::bar::config");
    println!("2. 命名空间影响符号名称");
    println!("3. FFI 声明使用完全限定名称");
    println!("4. Rust 端使用 opaque pointer 模式");
    println!("5. hicc import_class! 不支持嵌套命名空间，使用 raw extern \"C\"");
}
