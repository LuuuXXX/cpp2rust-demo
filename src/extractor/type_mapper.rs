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

    // 处理 `*__restrict` 无空格形式（libclang 预处理展开后常见，如 `wchar_t *__restrict`）
    // 必须在后缀形式处理之前，因为 `*__restrict` 不以 ` ` 开始无法被 strip_suffix 匹配
    if cpp.contains("*__restrict") {
        let normalized = cpp
            .replace("*__restrict__", "*")
            .replace("*__restrict", "*");
        return cpp_to_rust(normalized.trim());
    }

    // 纯值类型 `const T`（不含 `*`、`&`、`[`）→ 去掉 const 限定符
    // 例如 `const struct timespec` → `struct timespec` → `timespec`
    // `const T *`（指针到 const T）和 `const T[N]`（const 数组）不在此处理
    if let Some(rest) = cpp.strip_prefix("const ") {
        let rest = rest.trim();
        if !rest.contains('*') && !rest.contains('&') && !rest.contains('[') {
            return cpp_to_rust(rest);
        }
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

    // C 定长数组参数类型（如 `char[20]`、`__cancel_jmp_buf_tag[1]`）及无界数组（如 `double[]`）
    // 在 C 函数签名中数组参数会退化为指针，映射为 `*mut T`（const 基类型 → `*const T`）
    if cpp_no_restrict.ends_with(']') {
        if let Some(bracket_pos) = cpp_no_restrict.rfind('[') {
            let between = &cpp_no_restrict[bracket_pos + 1..cpp_no_restrict.len() - 1];
            // `T[N]`（N 为纯数字）或 `T[]`（无界）都退化为指针
            let is_array = between.is_empty() || between.chars().all(|c| c.is_ascii_digit());
            if is_array {
                let base = cpp_no_restrict[..bracket_pos].trim();
                // `const T[N]` → 元素不可变 → `*const T`
                let (inner, is_const) = base
                    .strip_prefix("const ")
                    .map(|b| (b.trim(), true))
                    .unwrap_or((base, false));
                let inner_rust = cpp_to_rust(inner);
                return if is_const {
                    if inner_rust.is_empty() {
                        "*const u8".to_string()
                    } else {
                        format!("*const {}", inner_rust)
                    }
                } else if inner_rust.is_empty() {
                    "*mut u8".to_string()
                } else {
                    format!("*mut {}", inner_rust)
                };
            }
        }
    }

    // `T *const` → C 语言中指针本身是 const（如 `char *const`、`void *const`）
    // 在 Rust FFI 中等价于 `T *`，映射为 `*mut T`（Rust 无"指针本身不可变"的概念）
    if let Some(rest) = cpp_no_restrict
        .strip_suffix(" *const")
        .or_else(|| cpp_no_restrict.strip_suffix("*const"))
    {
        let normalized = format!("{} *", rest.trim());
        return cpp_to_rust(&normalized);
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

    // C 函数指针 `RetType (*)(T1, T2, ...)` → `unsafe extern "C" fn(T1, T2) -> R`
    if let Some(mapped) = try_map_c_fn_ptr(cpp_no_restrict) {
        return mapped;
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

/// 尝试将 C 函数指针类型字符串映射为 Rust `unsafe extern "C" fn(...)` 类型。
///
/// 识别的形式：`RetType (*)(T1, T2, ...)` 或 `RetType (*)(void)` 或 `RetType (*)()`。
/// 不处理嵌套函数指针（参数中再含 `(*)`）；这些情况返回 `None`。
///
/// 返回 `Some(mapped_type)` 当成功解析，`None` 当类型不是合法的顶层 C 函数指针形式。
pub fn try_map_c_fn_ptr(cpp: &str) -> Option<String> {
    // 必须含有 `(*)(` 模式
    let star_paren = cpp.find("(*)(")?;

    // 提取 `(*)` 左边的返回类型
    let ret_cpp = cpp[..star_paren].trim();

    // 提取 `(*)(` 后到字符串末尾 `)` 之间的参数列表
    let after = &cpp[star_paren + 4..]; // 跳过 `(*)(` 这 4 个字符

    // 最后的 `)` 必须在末尾（顶层，不支持嵌套）
    let params_str = after.strip_suffix(')')?;

    // 若参数字符串本身含有 `(*)`，表示嵌套函数指针，不处理
    if params_str.contains("(*)") {
        return None;
    }

    // 解析参数列表
    let rust_params: Vec<String> = if params_str.trim().is_empty() || params_str.trim() == "void" {
        // `void (*)()` 或 `void (*)(void)` → 无参数
        vec![]
    } else {
        params_str
            .split(',')
            .map(|t| cpp_to_rust(t.trim()))
            .collect()
    };

    // 映射返回类型
    let rust_ret = cpp_to_rust(ret_cpp);

    // 构造 `unsafe extern "C" fn(T1, T2) -> R` 字符串
    let params_joined = rust_params.join(", ");
    let ret_suffix = if rust_ret.is_empty() {
        String::new() // void 返回 → 省略返回类型注解（无 `-> ...`）
    } else {
        format!(" -> {}", rust_ret)
    };

    Some(format!(
        "unsafe extern \"C\" fn({}){}",
        params_joined, ret_suffix
    ))
}

/// 清理 C++ 类型中的 `struct ` / `class ` 前缀
pub fn clean_type(ty: &str) -> &str {
    ty.strip_prefix("struct ")
        .or_else(|| ty.strip_prefix("class "))
        .unwrap_or(ty)
        .trim()
}

/// C++ camelCase / PascalCase 命名 → Rust snake_case 命名
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
        // 后缀形式（有空格）：__restrict / __restrict__ 出现在指针末尾时应被去掉，生成合法 Rust 类型
        assert_eq!(cpp_to_rust("wchar_t * __restrict"), "*mut wchar_t");
        assert_eq!(cpp_to_rust("const wchar_t * __restrict"), "*const wchar_t");
        assert_eq!(cpp_to_rust("wchar_t * __restrict__"), "*mut wchar_t");
        assert_eq!(cpp_to_rust("char * restrict"), "*mut i8");
        // 无空格形式：libclang 预处理展开后的实际输出（如 `wchar_t *__restrict`）
        assert_eq!(cpp_to_rust("wchar_t *__restrict"), "*mut wchar_t");
        assert_eq!(cpp_to_rust("const wchar_t *__restrict"), "*const wchar_t");
        assert_eq!(cpp_to_rust("wchar_t *__restrict__"), "*mut wchar_t");
        assert_eq!(cpp_to_rust("char *__restrict"), "*mut i8");
        // 前缀形式：MSVC 风格 `__restrict int *` / `__restrict__ char *`
        assert_eq!(cpp_to_rust("__restrict int *"), "*mut i32");
        assert_eq!(cpp_to_rust("__restrict__ char *"), "*mut i8");
    }

    #[test]
    fn test_c_array_param_becomes_pointer() {
        // C 定长数组参数类型（libclang 将其显示为 `T[N]`），应映射为 `*mut T`
        assert_eq!(cpp_to_rust("char[20]"), "*mut i8");
        assert_eq!(cpp_to_rust("int[4]"), "*mut i32");
        assert_eq!(
            cpp_to_rust("__cancel_jmp_buf_tag[1]"),
            "*mut __cancel_jmp_buf_tag"
        );
        assert_eq!(cpp_to_rust("unsigned char[16]"), "*mut u8");
        // 无界数组 `T[]` 同样退化为指针
        assert_eq!(cpp_to_rust("double[]"), "*mut f64");
        assert_eq!(cpp_to_rust("int[]"), "*mut i32");
        // const 数组 `const T[N]` → `*const T`
        assert_eq!(cpp_to_rust("const struct timespec[2]"), "*const timespec");
        assert_eq!(cpp_to_rust("const char[16]"), "*const i8");
    }

    #[test]
    fn test_const_pointer_qualifier() {
        // `T *const` 是 C 中指针本身 const（不可重赋值），Rust FFI 中等同于 `T *`
        assert_eq!(cpp_to_rust("char *const"), "*mut i8");
        assert_eq!(cpp_to_rust("void *const"), "*mut u8");
        // `char *const *` → 指向 const 指针的指针 → `*mut *mut i8`
        assert_eq!(cpp_to_rust("char *const *"), "*mut *mut i8");
    }

    // ── try_map_c_fn_ptr / C 函数指针映射 ────────────────────────────

    #[test]
    fn fn_ptr_basic() {
        // int (*)(int, int) → unsafe extern "C" fn(i32, i32) -> i32
        assert_eq!(
            cpp_to_rust("int (*)(int, int)"),
            "unsafe extern \"C\" fn(i32, i32) -> i32"
        );
    }

    #[test]
    fn fn_ptr_void_return() {
        // void (*)(int) → unsafe extern "C" fn(i32)（无返回类型后缀）
        assert_eq!(cpp_to_rust("void (*)(int)"), "unsafe extern \"C\" fn(i32)");
    }

    #[test]
    fn fn_ptr_no_params() {
        // void (*)() → unsafe extern "C" fn()
        assert_eq!(cpp_to_rust("void (*)()"), "unsafe extern \"C\" fn()");
    }

    #[test]
    fn fn_ptr_void_param() {
        // void (*)(void) 与 void (*)() 等价
        assert_eq!(cpp_to_rust("void (*)(void)"), "unsafe extern \"C\" fn()");
    }

    #[test]
    fn fn_ptr_ptr_param() {
        // void (*)(void*) → unsafe extern "C" fn(*mut u8)
        assert_eq!(
            cpp_to_rust("void (*)(void *)"),
            "unsafe extern \"C\" fn(*mut u8)"
        );
    }

    #[test]
    fn fn_ptr_const_char_return() {
        // const char* (*)(int) → unsafe extern "C" fn(i32) -> *const i8
        assert_eq!(
            cpp_to_rust("const char *(*)(int)"),
            "unsafe extern \"C\" fn(i32) -> *const i8"
        );
    }

    #[test]
    fn fn_ptr_nested_not_supported() {
        // 嵌套函数指针（参数中含 `(*)`）不递归处理，try_map_c_fn_ptr 返回 None，
        // cpp_to_rust 回退为原样字符串
        let nested = "int (*)(int (*)(int), int)";
        // 不应是合法的 unsafe extern "C" fn(...) 形式，原样返回
        let result = cpp_to_rust(nested);
        assert!(
            !result.starts_with("unsafe extern"),
            "嵌套函数指针不应递归处理，但得到了 {}",
            result
        );
    }

    #[test]
    fn non_fn_ptr_unchanged() {
        // 普通类型不受影响
        assert_eq!(cpp_to_rust("int"), "i32");
        assert_eq!(cpp_to_rust("Counter *"), "*mut Counter");
        assert_eq!(cpp_to_rust("const char *"), "*const i8");
    }
}
