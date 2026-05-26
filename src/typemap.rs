use regex::Regex;

/// 规范化 C++ 类型字符串，便于后续匹配和代码生成。
pub fn normalize_cpp_type(input: &str) -> String {
    let mut value = input.trim().replace('\n', " ");
    value = value.replace("struct ", "").replace("class ", "");
    value = Regex::new(r"\s+")
        .unwrap()
        .replace_all(&value, " ")
        .into_owned();
    value = value.replace(" *", "*").replace("* ", "*");
    value = value.replace(" &", "&").replace("& ", "&");
    value.trim().to_string()
}

/// C++ 类型到 Rust 类型的基础映射。
pub fn map_cpp_type_to_rust(cpp_type: &str) -> String {
    let normalized = normalize_cpp_type(cpp_type);
    match normalized.as_str() {
        "void" => "()".to_string(),
        "int" => "i32".to_string(),
        "unsigned int" => "u32".to_string(),
        "long" => "i64".to_string(),
        "unsigned long" => "u64".to_string(),
        "long long" => "i64".to_string(),
        "unsigned long long" => "u64".to_string(),
        "short" => "i16".to_string(),
        "unsigned short" => "u16".to_string(),
        "char" => "i8".to_string(),
        "unsigned char" => "u8".to_string(),
        "float" => "f32".to_string(),
        "double" => "f64".to_string(),
        "bool" => "bool".to_string(),
        "size_t" => "usize".to_string(),
        "int8_t" => "i8".to_string(),
        "int16_t" => "i16".to_string(),
        "int32_t" => "i32".to_string(),
        "int64_t" => "i64".to_string(),
        "uint8_t" => "u8".to_string(),
        "uint16_t" => "u16".to_string(),
        "uint32_t" => "u32".to_string(),
        "uint64_t" => "u64".to_string(),
        "const char*" => "*const i8".to_string(),
        "char*" => "*mut i8".to_string(),
        "void*" => "*mut std::ffi::c_void".to_string(),
        "const void*" => "*const std::ffi::c_void".to_string(),
        _ => map_custom_or_pointer_type(&normalized),
    }
}

pub fn is_raw_pointer_type(cpp_type: &str) -> bool {
    normalize_cpp_type(cpp_type).contains('*')
}

fn map_custom_or_pointer_type(normalized: &str) -> String {
    if let Some(inner) = normalized
        .strip_prefix("const ")
        .and_then(|value| value.strip_suffix('*'))
    {
        return format!("*const {}", map_cpp_type_to_rust(inner));
    }
    if let Some(inner) = normalized.strip_suffix('*') {
        return format!("*mut {}", map_cpp_type_to_rust(inner));
    }
    // C++ rvalue references (`T&&`) must be handled BEFORE single-`&` so that we don't
    // recurse twice and produce `*mut *mut T`.  Both `T&` and `T&&` map to raw pointers for FFI.
    if let Some(inner) = normalized
        .strip_prefix("const ")
        .and_then(|value| value.strip_suffix("&&"))
    {
        return format!("*const {}", map_cpp_type_to_rust(inner));
    }
    if let Some(inner) = normalized.strip_suffix("&&") {
        return format!("*mut {}", map_cpp_type_to_rust(inner));
    }
    if let Some(inner) = normalized
        .strip_prefix("const ")
        .and_then(|value| value.strip_suffix('&'))
    {
        return format!("*const {}", map_cpp_type_to_rust(inner));
    }
    if let Some(inner) = normalized.strip_suffix('&') {
        return format!("*mut {}", map_cpp_type_to_rust(inner));
    }
    normalized.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_primitive_types() {
        assert_eq!(map_cpp_type_to_rust("int"), "i32");
        assert_eq!(map_cpp_type_to_rust("unsigned long long"), "u64");
        assert_eq!(map_cpp_type_to_rust("size_t"), "usize");
    }

    #[test]
    fn maps_pointer_types() {
        assert_eq!(map_cpp_type_to_rust("const char*"), "*const i8");
        assert_eq!(map_cpp_type_to_rust("char*"), "*mut i8");
        assert_eq!(map_cpp_type_to_rust("const Foo*"), "*const Foo");
        assert_eq!(map_cpp_type_to_rust("Bar*"), "*mut Bar");
        assert_eq!(
            map_cpp_type_to_rust("const void*"),
            "*const std::ffi::c_void"
        );
    }

    #[test]
    fn maps_primitive_pointer_types_recursively() {
        // int*, double* etc. should map inner type to Rust primitive
        assert_eq!(map_cpp_type_to_rust("int*"), "*mut i32");
        assert_eq!(map_cpp_type_to_rust("double*"), "*mut f64");
        assert_eq!(map_cpp_type_to_rust("const int*"), "*const i32");
        assert_eq!(map_cpp_type_to_rust("unsigned char*"), "*mut u8");
    }

    #[test]
    fn normalizes_cpp_types() {
        assert_eq!(normalize_cpp_type(" struct Counter * "), "Counter*");
        assert_eq!(normalize_cpp_type("const  char *"), "const char*");
    }

    #[test]
    fn detects_raw_pointers() {
        assert!(is_raw_pointer_type("Counter*"));
        assert!(!is_raw_pointer_type("int"));
    }
}
