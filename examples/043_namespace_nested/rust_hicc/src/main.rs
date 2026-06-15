use namespace_nested::*;

fn decode_cstr(ptr: *const i8) -> String {
    unsafe { std::ffi::CStr::from_ptr(ptr).to_string_lossy().into_owned() }
}

fn main() {
    println!("=== 043_namespace_nested - Nested Namespaces ===\n");

    println!("--- foo::bar::config::ConfigManager ---");
    let mut config = config_manager_new();
    config.set_value("timeout\0".as_ptr() as *const i8, 30);
    config.set_value("retry\0".as_ptr() as *const i8, 3);
    config.set_value("port\0".as_ptr() as *const i8, 8080);

    println!("timeout = {}", config.get_value("timeout\0".as_ptr() as *const i8));
    println!("retry = {}", config.get_value("retry\0".as_ptr() as *const i8));
    println!("port = {}", config.get_value("port\0".as_ptr() as *const i8));

    println!("\n--- string_length ---");
    let test_str = "Hello, World!\0";
    let len = unsafe { string_length(test_str.as_ptr() as *const i8) };
    println!("string_length(\"{}\") = {}", &test_str[..len as usize], len);

    println!("\n--- foo::baz::DataProcessor ---");
    let processor = data_processor_new();
    println!("process(42) = {}", processor.process(42));

    println!("\n--- Top-level Functions ---");
    let version = unsafe { get_version() };
    println!("version = {}", decode_cstr(version));
    println!("build_number = {}", get_build_number());

    println!("\n--- Summary ---");
    println!("1. C++ nested namespaces: foo::bar::config");
    println!("2. Namespaces affect symbol names");
    println!("3. FFI declarations use fully-qualified names");
    println!("4. Rust side uses `using` aliases + import_class! for Direct mode");
    println!("5. hicc import_class! does not support nested namespaces directly");
}
