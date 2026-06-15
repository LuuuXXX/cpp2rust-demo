fn main() {
    let mut result = rapidjson_direct::parse_result_new();
    assert!(!result.is_error());
    assert_eq!(result.offset(), 0);
    result.clear();
    println!("Rust FFI: rapidjson ParseResult smoke test passed!");
}
