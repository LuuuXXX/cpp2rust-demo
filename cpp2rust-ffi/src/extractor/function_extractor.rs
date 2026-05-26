use crate::types::*;

/// 函数提取器
pub struct FunctionExtractor;

impl FunctionExtractor {
    /// 过滤：只保留不是类 shim 的全局函数
    pub fn filter_global_functions<'a>(
        functions: &'a [CppFunction],
        classes: &[CppClass],
    ) -> Vec<&'a CppFunction> {
        let class_prefixes: Vec<String> = classes.iter()
            .map(|c| c.ffi_prefix())
            .collect();

        functions.iter()
            .filter(|f| {
                !class_prefixes.iter().any(|prefix| f.name.starts_with(prefix.as_str()))
            })
            .collect()
    }

    /// 检测函数重载（同名但参数不同），生成带后缀的名称
    pub fn resolve_overloads(functions: &mut Vec<CppFunction>) {
        let names: Vec<String> = functions.iter().map(|f| f.name.clone()).collect();

        // 找出重名的函数
        let mut name_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
        for name in &names {
            *name_counts.entry(name.clone()).or_insert(0) += 1;
        }

        // 为重名函数加类型后缀
        let mut name_idx: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
        for func in functions.iter_mut() {
            if name_counts.get(&func.name).copied().unwrap_or(0) > 1 {
                let idx = name_idx.entry(func.name.clone()).or_insert(0);
                let suffix = if func.params.is_empty() {
                    "_void".to_string()
                } else {
                    let type_suffix = func.params.iter()
                        .map(|p| type_to_suffix(&p.rust_type))
                        .collect::<Vec<_>>()
                        .join("_");
                    format!("_{}", type_suffix)
                };
                func.name = format!("{}{}", func.name, suffix);
                *idx += 1;
            }
        }
    }
}

/// 将 Rust 类型转换为后缀字符串
fn type_to_suffix(rust_type: &str) -> String {
    match rust_type {
        "i8" => "i8",
        "i16" => "i16",
        "i32" => "i32",
        "i64" => "i64",
        "u8" => "u8",
        "u16" => "u16",
        "u32" => "u32",
        "u64" => "u64",
        "f32" => "f32",
        "f64" => "f64",
        "bool" => "bool",
        t if t.contains("i8") => "str",
        t if t.contains("void") => "ptr",
        _ => "ptr",
    }.to_string()
}
