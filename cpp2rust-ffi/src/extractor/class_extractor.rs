use crate::types::*;

/// 从 CppAst 中提取类信息（后处理：补充 shim 生成逻辑）
pub struct ClassExtractor;

impl ClassExtractor {
    /// 为类生成 FFI shim 函数列表（用于 hicc::cpp! 块）
    pub fn extract_shims(class: &CppClass) -> Vec<ShimFunction> {
        let mut shims = Vec::new();
        let prefix = class.ffi_prefix();
        let class_name = &class.name;

        // 默认构造函数
        let ctors: Vec<&CppMethod> = class.methods.iter()
            .filter(|m| m.is_constructor)
            .collect();

        if ctors.is_empty() {
            shims.push(ShimFunction {
                name: format!("{}_new", prefix),
                return_type: format!("{}*", class_name),
                params: Vec::new(),
                body: format!("return new {}();", class_name),
            });
        } else {
            for (i, ctor) in ctors.iter().enumerate() {
                let suffix = if ctors.len() > 1 {
                    format!("_new_{}", i)
                } else {
                    "_new".to_string()
                };
                let params: Vec<(String, String)> = ctor.params.iter()
                    .map(|p| (p.cpp_type.clone(), p.name.clone()))
                    .collect();
                let args: String = params.iter()
                    .map(|(_, n)| n.clone())
                    .collect::<Vec<_>>()
                    .join(", ");
                shims.push(ShimFunction {
                    name: format!("{}{}", prefix, suffix),
                    return_type: format!("{}*", class_name),
                    params,
                    body: format!("return new {}({});", class_name, args),
                });
            }
        }

        // 析构函数
        shims.push(ShimFunction {
            name: format!("{}_delete", prefix),
            return_type: "void".to_string(),
            params: vec![(format!("{}*", class_name), "self".to_string())],
            body: "delete self;".to_string(),
        });

        // 拷贝构造
        let has_copy = class.methods.iter().any(|m| m.is_constructor && m.params.len() == 1
            && m.params[0].cpp_type.contains(&format!("const {}",&class_name)));
        if has_copy {
            shims.push(ShimFunction {
                name: format!("{}_copy", prefix),
                return_type: format!("{}*", class_name),
                params: vec![(format!("const {}*", class_name), "other".to_string())],
                body: format!("return new {}(*other);", class_name),
            });
        }

        shims
    }
}

/// 一个 C 语言 shim 函数描述
pub struct ShimFunction {
    pub name: String,
    pub return_type: String,
    pub params: Vec<(String, String)>, // (type, name)
    pub body: String,
}

impl ShimFunction {
    /// 生成 C++ 函数定义字符串（用于 hicc::cpp! 块）
    pub fn to_cpp_string(&self) -> String {
        let params_str = if self.params.is_empty() {
            String::new()
        } else {
            self.params.iter()
                .map(|(t, n)| format!("{} {}", t, n))
                .collect::<Vec<_>>()
                .join(", ")
        };
        format!(
            "{} {}({}) {{\n        {}\n    }}",
            self.return_type, self.name, params_str, self.body
        )
    }
}
