fn main() {
    let input = r#"{"hello":"world"}"#;
    let c_input = std::ffi::CString::new(input).unwrap();
    let result_ptr = unsafe { nlohmann_json_direct::nlohmann_json_parse_and_dump(c_input.as_ptr()) };
    let result_cstr = unsafe { std::ffi::CStr::from_ptr(result_ptr) };
    let result_str = result_cstr.to_str().unwrap();
    assert!(result_str.contains("hello"));
    assert!(result_str.contains("world"));
    println!("Rust FFI: nlohmann_json parse_and_dump '{}' → '{}'", input, result_str);
}
