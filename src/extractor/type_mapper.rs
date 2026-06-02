//! C++ → Rust 类型映射

/// 将 libclang 返回的 C++ 显示类型字符串映射为 Rust 类型字符串。
///
/// 规则（按优先级）:
/// 1. 精确匹配已知原始类型
/// 2. `const T *` / `T *` → `*const T_rust` / `*mut T_rust`
/// 3. `const T &` / `T &` → 忽略引用（不支持）
/// 4. 未知类型返回原样
pub fn cpp_to_rust(cpp: &str) -> String {
    let cpp = cpp.trim();

    // 去掉 `volatile` 前缀（volatile 在 Rust 中无效）
    if let Some(rest) = cpp.strip_prefix("volatile ") {
        return cpp_to_rust(rest.trim());
    }

    // 去掉 `__restrict__` / `__restrict` 前缀形式（MSVC 风格，如 `__restrict int *`）
    if let Some(rest) = cpp
        .strip_prefix("__restrict__ ")
        .or_else(|| cpp.strip_prefix("__restrict "))
    {
        return cpp_to_rust(rest.trim());
    }

    // 去掉 `__restrict__` / `__restrict` / `restrict` 后缀形式
    // 这些限定符可出现在指针类型末尾（如 `wchar_t *__restrict`），在 Rust 中没有对应语义
    let cpp_no_restrict = cpp
        .strip_suffix(" __restrict__")
        .or_else(|| cpp.strip_suffix(" __restrict"))
        .or_else(|| cpp.strip_suffix(" restrict"))
        .map(str::trim)
        .unwrap_or(cpp);

    // 原始类型精确映射
    match cpp_no_restrict {
        "void" => return String::new(), // void → ()，调用方处理
        "bool" | "_Bool" => return "bool".to_string(),
        "char" => return "i8".to_string(),
        "signed char" => return "i8".to_string(),
        "unsigned char" => return "u8".to_string(),
        "short" | "short int" | "signed short" => return "i16".to_string(),
        "unsigned short" | "unsigned short int" => return "u16".to_string(),
        "int" | "signed int" | "signed" => return "i32".to_string(),
        "unsigned int" | "unsigned" => return "u32".to_string(),
        "long" | "long int" | "signed long" => return "i64".to_string(),
        "unsigned long" | "unsigned long int" => return "u64".to_string(),
        "long long" | "long long int" | "signed long long" => return "i64".to_string(),
        "unsigned long long" | "unsigned long long int" => return "u64".to_string(),
        "float" => return "f32".to_string(),
        "double" => return "f64".to_string(),
        "long double" => return "f64".to_string(),
        "size_t" => return "usize".to_string(),
        "ptrdiff_t" => return "isize".to_string(),
        "intptr_t" => return "isize".to_string(),
        "uintptr_t" => return "usize".to_string(),
        "int8_t" => return "i8".to_string(),
        "int16_t" => return "i16".to_string(),
        "int32_t" => return "i32".to_string(),
        "int64_t" => return "i64".to_string(),
        "uint8_t" => return "u8".to_string(),
        "uint16_t" => return "u16".to_string(),
        "uint32_t" => return "u32".to_string(),
        "uint64_t" => return "u64".to_string(),
        _ => {}
    }

    // `const char *` 系列 → *const i8（C char 为 signed，对应 Rust i8）
    if cpp_no_restrict == "const char *"
        || cpp_no_restrict == "const char*"
        || cpp_no_restrict == "char const *"
    {
        return "*const i8".to_string();
    }
    // `char *` → *mut i8
    if cpp_no_restrict == "char *" || cpp_no_restrict == "char*" {
        return "*mut i8".to_string();
    }

    // `const T *` → `*const T_rust`
    if let Some(rest) = cpp_no_restrict
        .strip_suffix(" *")
        .or_else(|| cpp_no_restrict.strip_suffix("*"))
    {
        let rest = rest.trim();
        if let Some(inner) = rest.strip_prefix("const ") {
            let inner = inner.trim();
            let inner_rust = cpp_to_rust(inner);
            if inner_rust.is_empty() {
                // `const void *` → `*const u8`
                return "*const u8".to_string();
            }
            return format!("*const {}", inner_rust);
        }
        // `T *` → `*mut T_rust`
        let inner_rust = cpp_to_rust(rest);
        if inner_rust.is_empty() {
            // `void *` → `*mut u8`
            return "*mut u8".to_string();
        }
        return format!("*mut {}", inner_rust);
    }

    // 引用类型：T& → &mut T，const T& → &T
    if let Some(rest) = cpp_no_restrict
        .strip_suffix(" &")
        .or_else(|| cpp_no_restrict.strip_suffix("&"))
    {
        let rest = rest.trim();
        if let Some(inner) = rest.strip_prefix("const ") {
            let inner = inner.trim();
            let inner_rust = cpp_to_rust(inner);
            if inner_rust.is_empty() {
                return "&u8".to_string();
            }
            return format!("&{}", inner_rust);
        }
        let inner_rust = cpp_to_rust(rest);
        if inner_rust.is_empty() {
            return "&mut u8".to_string();
        }
        return format!("&mut {}", inner_rust);
    }

    // 剥除 struct/class 前缀
    if let Some(rest) = cpp_no_restrict
        .strip_prefix("struct ")
        .or_else(|| cpp_no_restrict.strip_prefix("class "))
    {
        return cpp_to_rust(rest);
    }

    // 未知：原样返回
    cpp_no_restrict.to_string()
}

/// `cpp_to_rust` 的 FFI 函数版本（现与 `cpp_to_rust` 行为一致，均使用 i8 表示 char*）。
/// 保留此函数供已有调用方使用。
pub fn cpp_to_rust_ffi(cpp: &str) -> String {
    cpp_to_rust(cpp)
}

/// 构造 C++ 函数签名字符串（用于 #[cpp(func = "...")]）
///
/// 例：`counter_new() -> Counter*` → `Counter* counter_new()`
pub fn build_cpp_fn_sig(name: &str, ret: &str, params: &[(&str, &str)]) -> String {
    let param_str = params
        .iter()
        .map(|(_, ty)| ty.to_string())
        .collect::<Vec<_>>()
        .join(", ");
    if ret.is_empty() || ret == "void" {
        format!("void {}({})", name, param_str)
    } else {
        format!("{}* {}({})", clean_type(ret), name, param_str)
    }
}

/// 清理 C++ 类型中的 `struct ` / `class ` 前缀
pub fn clean_type(ty: &str) -> &str {
    ty.strip_prefix("struct ")
        .or_else(|| ty.strip_prefix("class "))
        .unwrap_or(ty)
        .trim()
}

/// C++ camelCase / PascalCase → snake_case
pub fn to_snake_case(s: &str) -> String {
    let mut result = String::with_capacity(s.len() + 4);
    let mut prev_upper = false;
    let chars: Vec<char> = s.chars().collect();
    for (i, &ch) in chars.iter().enumerate() {
        if ch.is_uppercase() {
            let next_lower = chars.get(i + 1).map(|c| c.is_lowercase()).unwrap_or(false);
            if !result.is_empty() && (!prev_upper || next_lower) {
                result.push('_');
            }
            result.push(ch.to_lowercase().next().unwrap());
            prev_upper = true;
        } else {
            result.push(ch);
            prev_upper = false;
        }
    }
    result
}

/// 判断 Rust 类型是否需要 unsafe（含裸指针）
pub fn needs_unsafe(rust_type: &str) -> bool {
    rust_type.contains("*mut ") || rust_type.contains("*const ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primitives() {
        assert_eq!(cpp_to_rust("int"), "i32");
        assert_eq!(cpp_to_rust("double"), "f64");
        assert_eq!(cpp_to_rust("const char *"), "*const i8");
        assert_eq!(cpp_to_rust("char *"), "*mut i8");
    }

    #[test]
    fn test_pointer() {
        assert_eq!(cpp_to_rust("Counter *"), "*mut Counter");
        assert_eq!(cpp_to_rust("const Counter *"), "*const Counter");
    }

    #[test]
    fn test_snake_case() {
        assert_eq!(to_snake_case("getValue"), "get_value");
        assert_eq!(to_snake_case("getX"), "get_x");
        assert_eq!(to_snake_case("getName"), "get_name");
        assert_eq!(to_snake_case("hello"), "hello");
    }

    #[test]
    fn test_restrict_qualifier_stripped() {
        // 后缀形式：__restrict / __restrict__ 出现在指针末尾时应被去掉，生成合法 Rust 类型
        assert_eq!(cpp_to_rust("wchar_t * __restrict"), "*mut wchar_t");
        assert_eq!(cpp_to_rust("const wchar_t * __restrict"), "*const wchar_t");
        assert_eq!(cpp_to_rust("wchar_t * __restrict__"), "*mut wchar_t");
        assert_eq!(cpp_to_rust("char * restrict"), "*mut i8");
        // 前缀形式：MSVC 风格 `__restrict int *` / `__restrict__ char *`
        assert_eq!(cpp_to_rust("__restrict int *"), "*mut i32");
        assert_eq!(cpp_to_rust("__restrict__ char *"), "*mut i8");
    }
}
