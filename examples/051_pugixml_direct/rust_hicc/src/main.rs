fn main() {
    let result = pugixml_direct::xml_parse_result_new();
    let desc = result.description();
    let desc_str = unsafe { std::ffi::CStr::from_ptr(desc) }
        .to_str()
        .unwrap_or("unknown");
    assert!(!desc_str.is_empty());
    println!("Rust FFI: pugixml xml_parse_result description = '{}'", desc_str);
}
