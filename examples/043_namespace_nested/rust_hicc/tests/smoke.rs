use namespace_nested::*;

fn decode_cstr(ptr: *const i8) -> String {
    unsafe { std::ffi::CStr::from_ptr(ptr).to_string_lossy().into_owned() }
}

#[test]
fn smoke_config_manager_set_get() {
    let mut config = config_manager_new();
    config.set_value("timeout\0".as_ptr() as *const i8, 30);
    config.set_value("retry\0".as_ptr() as *const i8, 3);
    config.set_value("port\0".as_ptr() as *const i8, 8080);
    assert_eq!(config.get_value("timeout\0".as_ptr() as *const i8), 30, "timeout should be 30");
    assert_eq!(config.get_value("retry\0".as_ptr() as *const i8), 3, "retry should be 3");
    assert_eq!(config.get_value("port\0".as_ptr() as *const i8), 8080, "port should be 8080");
}

#[test]
fn smoke_config_manager_missing_key() {
    let config = config_manager_new();
    let val = config.get_value("nonexistent\0".as_ptr() as *const i8);
    assert_eq!(val, 0, "nonexistent key should return 0");
}

#[test]
fn smoke_config_manager_overwrite() {
    let mut config = config_manager_new();
    config.set_value("key\0".as_ptr() as *const i8, 10);
    config.set_value("key\0".as_ptr() as *const i8, 20);
    let val = config.get_value("key\0".as_ptr() as *const i8);
    assert_eq!(val, 20, "overwritten value should be 20");
}

#[test]
fn smoke_string_length() {
    let len = unsafe { string_length("Hello, World!\0".as_ptr() as *const i8) };
    assert_eq!(len, 13, "string_length(\"Hello, World!\") should be 13");
}

#[test]
fn smoke_string_length_empty() {
    let len = unsafe { string_length("\0".as_ptr() as *const i8) };
    assert_eq!(len, 0, "empty string length should be 0");
}

#[test]
fn smoke_data_processor() {
    let processor = data_processor_new();
    let result = processor.process(42);
    assert_eq!(result, 42, "data_processor_process(42) should return 42");
}

#[test]
fn smoke_get_version() {
    let version = unsafe { get_version() };
    let version_str = decode_cstr(version);
    assert_eq!(version_str, "1.0.0", "version should be 1.0.0");
}

#[test]
fn smoke_get_build_number() {
    let build = get_build_number();
    assert_eq!(build, 42, "build number should be 42");
}
