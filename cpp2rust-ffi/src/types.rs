use serde::{Deserialize, Serialize};

/// C++ 类型 → Rust 类型映射
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CppType {
    /// 原始 C++ 类型字符串（如 "int", "const char*", "Foo*"）
    pub cpp: String,
    /// 对应 Rust 类型（如 "i32", "*const i8", "*mut Foo"）
    pub rust: String,
    /// 是否需要 unsafe 调用
    pub is_unsafe: bool,
}

impl CppType {
    pub fn from_cpp(cpp: &str) -> Self {
        let (rust, is_unsafe) = map_cpp_type_to_rust(cpp);
        Self {
            cpp: cpp.to_string(),
            rust,
            is_unsafe,
        }
    }

    pub fn is_void(&self) -> bool {
        self.cpp.trim() == "void"
    }
}

/// 将 C++ 类型字符串映射到 Rust 类型
pub fn map_cpp_type_to_rust(cpp: &str) -> (String, bool) {
    let cpp = cpp.trim();
    // 处理 const 指针
    if cpp.contains('*') {
        return map_pointer_type(cpp);
    }
    // 处理引用
    if cpp.contains('&') {
        let inner = cpp.trim_end_matches('&').trim().trim_start_matches("const").trim();
        let (inner_rust, _) = map_cpp_type_to_rust(inner);
        if cpp.starts_with("const ") {
            return (format!("&{}", inner_rust), false);
        } else {
            return (format!("&mut {}", inner_rust), false);
        }
    }
    let rust = match cpp {
        "void" => "()",
        "int" | "signed int" => "i32",
        "unsigned int" | "unsigned" => "u32",
        "long" | "long int" | "signed long" => "i64",
        "unsigned long" | "unsigned long int" => "u64",
        "long long" | "long long int" | "signed long long" => "i64",
        "unsigned long long" | "unsigned long long int" => "u64",
        "short" | "short int" | "signed short" => "i16",
        "unsigned short" | "unsigned short int" => "u16",
        "char" | "signed char" => "i8",
        "unsigned char" => "u8",
        "float" => "f32",
        "double" => "f64",
        "long double" => "f64",
        "bool" => "bool",
        "size_t" => "usize",
        "ptrdiff_t" => "isize",
        "intptr_t" => "isize",
        "uintptr_t" => "usize",
        "int8_t" => "i8",
        "uint8_t" => "u8",
        "int16_t" => "i16",
        "uint16_t" => "u16",
        "int32_t" => "i32",
        "uint32_t" => "u32",
        "int64_t" => "i64",
        "uint64_t" => "u64",
        _ => cpp, // 其他类型（类名等）直接使用
    };
    (rust.to_string(), false)
}

/// 映射指针类型
fn map_pointer_type(cpp: &str) -> (String, bool) {
    // 去掉最外层 const/volatile 和空格
    let cpp = cpp.trim();
    // const char* → *const i8
    // char* → *mut i8
    // const T* → *const T
    // T* → *mut T
    // void* → *mut c_void

    let mut s = cpp;
    // 计算指针层级
    let ptr_count = s.matches('*').count();

    // 去掉所有 * 和 const，得到基本类型
    let s_no_ptr = s.replace('*', "").replace("const", "").replace("volatile", "");
    let base = s_no_ptr.trim();

    // 判断最终是 const 还是 mut
    // 如果包含 const 在 * 前面，说明指向的是 const
    let is_const_pointee = cpp.starts_with("const ") || cpp.contains("const ")
        && {
            // 找到第一个 *，看其前面是否有 const
            let star_pos = cpp.find('*').unwrap_or(cpp.len());
            cpp[..star_pos].contains("const")
        };

    let base_rust = match base {
        "void" => "std::ffi::c_void",
        "char" | "signed char" => "i8",
        "unsigned char" => "u8",
        "int" | "signed int" => "i32",
        "unsigned int" | "unsigned" => "u32",
        "long" | "long int" => "i64",
        "unsigned long" | "unsigned long int" => "u64",
        "long long" => "i64",
        "unsigned long long" => "u64",
        "short" => "i16",
        "unsigned short" => "u16",
        "float" => "f32",
        "double" => "f64",
        "bool" => "bool",
        "size_t" => "usize",
        "int8_t" => "i8",
        "uint8_t" => "u8",
        "int16_t" => "i16",
        "uint16_t" => "u16",
        "int32_t" => "i32",
        "uint32_t" => "u32",
        "int64_t" => "i64",
        "uint64_t" => "u64",
        other => other,
    };

    let qualifier = if is_const_pointee { "const" } else { "mut" };
    let mut result = format!("*{} {}", qualifier, base_rust);
    // 额外的指针层级
    for _ in 1..ptr_count {
        if is_const_pointee {
            result = format!("*const {}", result);
        } else {
            result = format!("*mut {}", result);
        }
    }
    (result, true) // 指针类型通常需要 unsafe
}

/// C++ 函数参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CppParam {
    pub name: String,
    pub cpp_type: String,
    pub rust_name: String,
    pub rust_type: String,
}

impl CppParam {
    pub fn new(name: &str, cpp_type: &str) -> Self {
        let (rust_type, _) = map_cpp_type_to_rust(cpp_type);
        let rust_name = sanitize_rust_name(name);
        Self {
            name: name.to_string(),
            cpp_type: cpp_type.to_string(),
            rust_name,
            rust_type,
        }
    }
}

/// 将 C++ 名称转换为合法的 Rust 标识符
pub fn sanitize_rust_name(name: &str) -> String {
    match name {
        "self" => "self_".to_string(),
        "type" => "type_".to_string(),
        "ref" => "ref_".to_string(),
        "move" => "move_".to_string(),
        "fn" => "fn_".to_string(),
        "in" => "in_".to_string(),
        "loop" => "loop_".to_string(),
        "match" => "match_".to_string(),
        "use" => "use_".to_string(),
        "mod" => "mod_".to_string(),
        "trait" => "trait_".to_string(),
        "impl" => "impl_".to_string(),
        "where" => "where_".to_string(),
        "let" => "let_".to_string(),
        "mut" => "mut_".to_string(),
        "const" => "const_".to_string(),
        "static" => "static_".to_string(),
        "pub" => "pub_".to_string(),
        "crate" => "crate_".to_string(),
        "super" => "super_".to_string(),
        "extern" => "extern_".to_string(),
        "unsafe" => "unsafe_".to_string(),
        "return" => "return_".to_string(),
        "break" => "break_".to_string(),
        "continue" => "continue_".to_string(),
        "yield" => "yield_".to_string(),
        "abstract" => "abstract_".to_string(),
        "become" => "become_".to_string(),
        "box" => "box_".to_string(),
        "do" => "do_".to_string(),
        "final" => "final_".to_string(),
        "macro" => "macro_".to_string(),
        "override" => "override_".to_string(),
        "priv" => "priv_".to_string(),
        "try" => "try_".to_string(),
        "typeof" => "typeof_".to_string(),
        "unsized" => "unsized_".to_string(),
        "virtual" => "virtual_".to_string(),
        // Rust std 库函数冲突
        "min" => "min_val".to_string(),
        "max" => "max_val".to_string(),
        "abs" => "abs_val".to_string(),
        "" => "arg".to_string(),
        name => {
            // 将非法字符替换为下划线
            let sanitized: String = name.chars().map(|c| {
                if c.is_alphanumeric() || c == '_' { c } else { '_' }
            }).collect();
            // 如果以数字开头，加前缀
            if sanitized.chars().next().map_or(false, |c| c.is_ascii_digit()) {
                format!("arg_{}", sanitized)
            } else {
                sanitized
            }
        }
    }
}

/// C++ 函数（全局函数或 shim 函数）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CppFunction {
    pub name: String,
    pub return_type: String,
    pub params: Vec<CppParam>,
    pub is_const: bool,
    pub is_virtual: bool,
    pub is_static: bool,
    pub is_inline: bool,
    pub is_noexcept: bool,
    pub is_unsafe: bool,
    pub is_variadic: bool,
    /// 函数所属类（如果是成员函数）
    pub class_name: Option<String>,
    /// 命名空间前缀
    pub namespace: Vec<String>,
    /// 原始 C++ 函数签名字符串
    pub cpp_signature: String,
}

impl CppFunction {
    pub fn new(name: &str, return_type: &str) -> Self {
        let (_, return_is_unsafe) = map_cpp_type_to_rust(return_type);
        Self {
            name: name.to_string(),
            return_type: return_type.to_string(),
            params: Vec::new(),
            is_const: false,
            is_virtual: false,
            is_static: false,
            is_inline: false,
            is_noexcept: false,
            is_unsafe: return_is_unsafe,
            is_variadic: false,
            class_name: None,
            namespace: Vec::new(),
            cpp_signature: String::new(),
        }
    }

    /// Rust 函数名（可能加命名空间前缀）
    pub fn rust_name(&self) -> String {
        sanitize_rust_name(&self.name)
    }

    /// 是否所有参数都需要 unsafe
    pub fn needs_unsafe(&self) -> bool {
        if self.is_unsafe {
            return true;
        }
        self.params.iter().any(|p| {
            let (_, unsafe_) = map_cpp_type_to_rust(&p.cpp_type);
            unsafe_
        })
    }

    /// 生成 Rust 参数列表字符串（不含 &self）
    pub fn rust_params_str(&self) -> String {
        self.params.iter().map(|p| {
            format!("{}: {}", p.rust_name, p.rust_type)
        }).collect::<Vec<_>>().join(", ")
    }

    /// 生成 Rust 返回类型字符串
    pub fn rust_return_type(&self) -> String {
        if self.return_type.trim() == "void" {
            String::new()
        } else {
            let (rust_type, _) = map_cpp_type_to_rust(&self.return_type);
            format!(" -> {}", rust_type)
        }
    }
}

/// C++ 类成员函数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CppMethod {
    pub name: String,
    pub return_type: String,
    pub params: Vec<CppParam>,
    pub is_const: bool,
    pub is_virtual: bool,
    pub is_pure_virtual: bool,
    pub is_static: bool,
    pub is_constructor: bool,
    pub is_destructor: bool,
    pub is_noexcept: bool,
    /// 原始 C++ 方法签名
    pub cpp_signature: String,
}

impl CppMethod {
    pub fn new(name: &str, return_type: &str) -> Self {
        Self {
            name: name.to_string(),
            return_type: return_type.to_string(),
            params: Vec::new(),
            is_const: false,
            is_virtual: false,
            is_pure_virtual: false,
            is_static: false,
            is_constructor: false,
            is_destructor: false,
            is_noexcept: false,
            cpp_signature: String::new(),
        }
    }

    pub fn rust_name(&self) -> String {
        sanitize_rust_name(&self.name)
    }

    pub fn rust_self_ref(&self) -> &'static str {
        if self.is_const { "&self" } else { "&mut self" }
    }

    pub fn needs_unsafe(&self) -> bool {
        if self.return_type.contains('*') {
            return true;
        }
        self.params.iter().any(|p| p.cpp_type.contains('*'))
    }

    /// 生成 import_class! 中方法声明的参数字符串（不含 self）
    pub fn rust_params_str(&self) -> String {
        self.params.iter().map(|p| {
            format!("{}: {}", p.rust_name, p.rust_type)
        }).collect::<Vec<_>>().join(", ")
    }

    pub fn rust_return_type(&self) -> String {
        if self.return_type.trim() == "void" {
            String::new()
        } else {
            let (rust_type, _) = map_cpp_type_to_rust(&self.return_type);
            format!(" -> {}", rust_type)
        }
    }
}

/// C++ 类（含成员函数）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CppClass {
    pub name: String,
    pub namespace: Vec<String>,
    pub methods: Vec<CppMethod>,
    pub base_classes: Vec<String>,
    pub is_abstract: bool,
    pub is_template_specialization: bool,
    /// 模板参数（如果是模板实例化）
    pub template_args: Vec<String>,
}

impl CppClass {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            namespace: Vec::new(),
            methods: Vec::new(),
            base_classes: Vec::new(),
            is_abstract: false,
            is_template_specialization: false,
            template_args: Vec::new(),
        }
    }

    /// 完全限定名（含命名空间，用 :: 分隔）
    pub fn qualified_name(&self) -> String {
        if self.namespace.is_empty() {
            self.name.clone()
        } else {
            format!("{}::{}", self.namespace.join("::"), self.name)
        }
    }

    /// FFI 函数名前缀（命名空间用 _ 分隔，类名小写）
    pub fn ffi_prefix(&self) -> String {
        let mut parts = self.namespace.clone();
        parts.push(self.name.clone());
        parts.iter().map(|s| to_snake_case(s)).collect::<Vec<_>>().join("_")
    }
}

/// C++ 枚举
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CppEnum {
    pub name: String,
    pub namespace: Vec<String>,
    pub is_scoped: bool, // enum class
    pub underlying_type: String,
    pub values: Vec<(String, i64)>,
}

impl CppEnum {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            namespace: Vec::new(),
            is_scoped: false,
            underlying_type: "int".to_string(),
            values: Vec::new(),
        }
    }
}

/// C++ 全局常量
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CppConst {
    pub name: String,
    pub cpp_type: String,
    pub value: String,
}

/// 解析后的 C++ AST 信息
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CppAst {
    pub classes: Vec<CppClass>,
    pub functions: Vec<CppFunction>,
    pub enums: Vec<CppEnum>,
    pub consts: Vec<CppConst>,
    /// 用于 hicc::cpp! 块的头文件 includes
    pub includes: Vec<String>,
    /// 源文件名（不含路径和扩展名）
    pub source_name: String,
}

/// 将 CamelCase 转为 snake_case
pub fn to_snake_case(s: &str) -> String {
    if s.is_empty() {
        return String::new();
    }
    let chars: Vec<char> = s.chars().collect();
    let mut result = String::new();
    for (i, &c) in chars.iter().enumerate() {
        if c.is_uppercase() {
            if i > 0 {
                let prev_lower = chars[i - 1].is_lowercase() || chars[i - 1].is_ascii_digit();
                let next_lower = i + 1 < chars.len() && chars[i + 1].is_lowercase();
                if prev_lower || (next_lower && chars[i - 1].is_uppercase()) {
                    result.push('_');
                }
            }
            result.push(c.to_lowercase().next().unwrap());
        } else {
            result.push(c);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_mapping() {
        assert_eq!(map_cpp_type_to_rust("int").0, "i32");
        assert_eq!(map_cpp_type_to_rust("void").0, "()");
        assert_eq!(map_cpp_type_to_rust("const char*").0, "*const i8");
        assert_eq!(map_cpp_type_to_rust("int*").0, "*mut i32");
        assert_eq!(map_cpp_type_to_rust("double").0, "f64");
        assert_eq!(map_cpp_type_to_rust("bool").0, "bool");
    }

    #[test]
    fn test_sanitize_name() {
        assert_eq!(sanitize_rust_name("self"), "self_");
        assert_eq!(sanitize_rust_name("type"), "type_");
        assert_eq!(sanitize_rust_name("foo"), "foo");
    }

    #[test]
    fn test_snake_case() {
        assert_eq!(to_snake_case("Counter"), "counter");
        assert_eq!(to_snake_case("MyClass"), "my_class");
        assert_eq!(to_snake_case("XMLParser"), "xml_parser");
    }
}
