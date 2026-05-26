use crate::types::*;

/// 从 CppAst 生成 hicc 格式的 Rust FFI 代码
pub struct HiccCodegen {
    /// 是否生成 main() 函数（用于演示）
    pub generate_main: bool,
}

impl HiccCodegen {
    pub fn new() -> Self {
        Self {
            generate_main: true,
        }
    }

    /// 生成完整的 Rust FFI 文件内容
    pub fn generate(&self, ast: &CppAst) -> String {
        let mut output = String::new();

        let link_name = &ast.source_name;

        // 1. hicc::cpp! 块
        let cpp_block = self.generate_cpp_block(ast);
        if !cpp_block.is_empty() {
            output.push_str(&cpp_block);
            output.push_str("\n\n");
        }

        // 2. hicc::import_class! 块（如果有类）
        if !ast.classes.is_empty() {
            let class_block = self.generate_import_class_block(ast);
            if !class_block.is_empty() {
                output.push_str(&class_block);
                output.push_str("\n\n");
            }
        }

        // 3. hicc::import_lib! 块
        let lib_block = self.generate_import_lib_block(ast, link_name);
        if !lib_block.is_empty() {
            output.push_str(&lib_block);
            output.push_str("\n\n");
        }

        // 4. main() 函数
        if self.generate_main {
            let main_fn = self.generate_main_fn(ast);
            output.push_str(&main_fn);
        }

        output
    }

    /// 生成 hicc::cpp! 块
    fn generate_cpp_block(&self, ast: &CppAst) -> String {
        let mut lines: Vec<String> = Vec::new();

        // includes
        for inc in &ast.includes {
            lines.push(format!("    #include {}", inc));
        }

        // 如果有类，生成内联类定义和 shim 函数
        if !ast.classes.is_empty() {
            lines.push(String::new());
            for class in &ast.classes {
                if !class.is_abstract {
                    // 生成类的 shim 函数
                    self.generate_class_shims(class, &mut lines);
                }
            }
        }

        if lines.is_empty() {
            return String::new();
        }

        format!("hicc::cpp! {{\n{}\n}}", lines.join("\n"))
    }

    /// 生成类的 C++ shim 函数
    fn generate_class_shims(&self, class: &CppClass, lines: &mut Vec<String>) {
        let prefix = class.ffi_prefix();
        let class_name = &class.name;

        // 构造函数 shim
        let ctors: Vec<&CppMethod> = class.methods.iter()
            .filter(|m| m.is_constructor)
            .collect();

        if ctors.is_empty() {
            // 默认构造函数
            lines.push(format!("    {}* {}_new() {{", class_name, prefix));
            lines.push(format!("        return new {}();", class_name));
            lines.push("    }".to_string());
        } else {
            for (i, ctor) in ctors.iter().enumerate() {
                let suffix = if ctors.len() > 1 { format!("_new_{}", i) } else { "_new".to_string() };
                let params = ctor.params.iter()
                    .map(|p| format!("{} {}", p.cpp_type, p.name))
                    .collect::<Vec<_>>()
                    .join(", ");
                let args = ctor.params.iter()
                    .map(|p| p.name.clone())
                    .collect::<Vec<_>>()
                    .join(", ");
                lines.push(format!("    {}* {}{}({}) {{", class_name, prefix, suffix, params));
                lines.push(format!("        return new {}({});", class_name, args));
                lines.push("    }".to_string());
            }
        }

        // 析构函数 shim
        lines.push(format!("    void {}_delete({}* self) {{", prefix, class_name));
        lines.push(format!("        delete self;", ));
        lines.push("    }".to_string());
    }

    /// 生成 hicc::import_class! 块
    fn generate_import_class_block(&self, ast: &CppAst) -> String {
        let mut blocks: Vec<String> = Vec::new();

        for class in &ast.classes {
            if class.is_abstract {
                continue; // 抽象类只生成前向声明
            }

            let methods: Vec<&CppMethod> = class.methods.iter()
                .filter(|m| !m.is_constructor && !m.is_destructor && !m.is_static)
                .collect();

            if methods.is_empty() {
                continue;
            }

            let mut class_block = Vec::new();
            class_block.push(format!("    #[cpp(class = \"{}\")]", class.name));
            class_block.push(format!("    class {} {{", class.name));

            for method in &methods {
                let params_str = method.rust_params_str();
                let self_ref = method.rust_self_ref();
                let ret_type = method.rust_return_type();

                let sep = if params_str.is_empty() { "" } else { ", " };

                // 构建 C++ 签名
                let const_str = if method.is_const { " const" } else { "" };
                let cpp_params = method.params.iter()
                    .map(|p| format!("{} {}", p.cpp_type, p.name))
                    .collect::<Vec<_>>()
                    .join(", ");
                let cpp_sig = format!("{} {}({}){}", method.return_type, method.name, cpp_params, const_str);

                class_block.push(format!("        #[cpp(method = \"{}\")]", cpp_sig));
                let params_with_sep = if params_str.is_empty() {
                    String::new()
                } else {
                    format!(", {}", params_str)
                };
                class_block.push(format!("        fn {}({}{}){};",
                    method.rust_name(), self_ref, params_with_sep, ret_type));
                class_block.push(String::new());
            }

            // 去掉最后的空行
            if class_block.last().map_or(false, |s| s.is_empty()) {
                class_block.pop();
            }

            class_block.push("    }".to_string());
            blocks.push(class_block.join("\n"));
        }

        if blocks.is_empty() {
            return String::new();
        }

        format!("hicc::import_class! {{\n{}\n}}", blocks.join("\n\n"))
    }

    /// 生成 hicc::import_lib! 块
    fn generate_import_lib_block(&self, ast: &CppAst, link_name: &str) -> String {
        let mut lines: Vec<String> = Vec::new();

        lines.push(format!("    #![link_name = \"{}\"]", link_name));
        lines.push(String::new());

        // 类的前向声明
        for class in &ast.classes {
            lines.push(format!("    class {};", class.name));
        }

        if !ast.classes.is_empty() {
            lines.push(String::new());
        }

        // 类的构造/析构 shim 函数
        for class in &ast.classes {
            let prefix = class.ffi_prefix();
            let class_name = &class.name;

            // 构造函数
            let ctors: Vec<&CppMethod> = class.methods.iter()
                .filter(|m| m.is_constructor)
                .collect();

            if ctors.is_empty() {
                lines.push(format!("    #[cpp(func = \"{}* {}_new()\")]", class_name, prefix));
                lines.push(format!("    fn {}_new() -> *mut {};", prefix, class_name));
                lines.push(String::new());
            } else {
                for (i, ctor) in ctors.iter().enumerate() {
                    let suffix = if ctors.len() > 1 { format!("_new_{}", i) } else { "_new".to_string() };
                    let cpp_params = ctor.params.iter()
                        .map(|p| format!("{} {}", p.cpp_type, p.name))
                        .collect::<Vec<_>>()
                        .join(", ");
                    let rust_params = ctor.params.iter()
                        .map(|p| format!("{}: {}", p.rust_name, p.rust_type))
                        .collect::<Vec<_>>()
                        .join(", ");
                    lines.push(format!("    #[cpp(func = \"{}* {}{}({})\")]",
                        class_name, prefix, suffix, cpp_params));
                    let sep = if rust_params.is_empty() { "" } else { "" };
                    lines.push(format!("    fn {}{}({}) -> *mut {};",
                        prefix, suffix, rust_params, class_name));
                    lines.push(String::new());
                }
            }

            // 析构函数
            lines.push(format!("    #[cpp(func = \"void {}_delete({}*)\")]", prefix, class_name));
            lines.push(format!("    unsafe fn {}_delete(self_: *mut {});", prefix, class_name));
            lines.push(String::new());

            // 静态成员函数 shim
            for method in class.methods.iter().filter(|m| m.is_static && !m.is_constructor && !m.is_destructor) {
                let cpp_params = method.params.iter()
                    .map(|p| format!("{} {}", p.cpp_type, p.name))
                    .collect::<Vec<_>>()
                    .join(", ");
                let rust_params = method.params.iter()
                    .map(|p| format!("{}: {}", p.rust_name, p.rust_type))
                    .collect::<Vec<_>>()
                    .join(", ");
                let ret_type = method.rust_return_type();
                let fn_name = format!("{}_{}", prefix, method.rust_name());
                let needs_unsafe = method.needs_unsafe();

                lines.push(format!("    #[cpp(func = \"{} {}::{}({})\" /* static */)]",
                    method.return_type, class_name, method.name, cpp_params));
                let unsafe_kw = if needs_unsafe { "unsafe " } else { "" };
                lines.push(format!("    {}fn {}({}){};",
                    unsafe_kw, fn_name, rust_params, ret_type));
                lines.push(String::new());
            }
        }

        // 全局函数
        for func in &ast.functions {
            // 跳过以类名为前缀的 shim 函数（已在类处理中生成）
            let is_class_shim = ast.classes.iter().any(|c| {
                let prefix = c.ffi_prefix();
                func.name.starts_with(&prefix)
            });
            if is_class_shim {
                continue;
            }

            let rust_params = func.params.iter()
                .map(|p| format!("{}: {}", p.rust_name, p.rust_type))
                .collect::<Vec<_>>()
                .join(", ");
            let ret_type = func.rust_return_type();
            let needs_unsafe = func.needs_unsafe();

            // 使用 cpp_signature 字段（已包含正确的 void 表示）
            let sig = if func.cpp_signature.is_empty() {
                format!("{} {}(void)", func.return_type, func.name)
            } else {
                func.cpp_signature.clone()
            };
            lines.push(format!("    #[cpp(func = \"{}\")]", sig));
            let unsafe_kw = if needs_unsafe { "unsafe " } else { "" };
            lines.push(format!("    {}fn {}({}){};",
                unsafe_kw, func.rust_name(), rust_params, ret_type));
            lines.push(String::new());
        }

        // 枚举常量（enum class 的值）
        for enum_ in &ast.enums {
            for (val_name, val) in &enum_.values {
                let rust_type = map_cpp_type_to_rust(&enum_.underlying_type).0;
                if enum_.is_scoped {
                    lines.push(format!("    // enum class {}::{} = {}",
                        enum_.name, val_name, val));
                }
                lines.push(format!("    const {}_{}: {} = {};",
                    to_snake_case(&enum_.name).to_uppercase(),
                    val_name.to_uppercase(),
                    rust_type, val));
            }
            if !enum_.values.is_empty() {
                lines.push(String::new());
            }
        }

        // 去掉结尾的空行
        while lines.last().map_or(false, |s: &String| s.is_empty()) {
            lines.pop();
        }

        format!("hicc::import_lib! {{\n{}\n}}", lines.join("\n"))
    }

    /// 生成演示用的 main() 函数
    fn generate_main_fn(&self, ast: &CppAst) -> String {
        let mut lines: Vec<String> = Vec::new();
        lines.push("fn main() {".to_string());

        if ast.classes.is_empty() && ast.functions.is_empty() {
            lines.push(format!(
                "    println!(\"Rust FFI: {} loaded successfully!\");",
                ast.source_name
            ));
        } else if !ast.classes.is_empty() {
            let class = &ast.classes[0];
            let prefix = class.ffi_prefix();
            let class_name = &class.name;

            lines.push(format!("    let mut obj = {}_new();", prefix));

            // 调用几个方法
            let show_methods: Vec<&CppMethod> = class.methods.iter()
                .filter(|m| !m.is_constructor && !m.is_destructor && !m.is_static)
                .take(3)
                .collect();

            for method in show_methods {
                let simple_args: Vec<String> = method.params.iter()
                    .map(|p| default_value_for_type(&p.rust_type))
                    .collect();
                let args_str = simple_args.join(", ");
                let method_call = format!("obj.{}({})", method.rust_name(), args_str);

                if method.return_type.trim() == "void" {
                    lines.push(format!("    {};", method_call));
                } else {
                    let (rust_ret, _) = map_cpp_type_to_rust(&method.return_type);
                    if rust_ret.starts_with('*') {
                        lines.push(format!("    let result = unsafe {{ {} }};", method_call));
                    } else {
                        lines.push(format!("    let result = {};", method_call));
                        lines.push(format!("    println!(\"{}: {{}}\", result);",
                            method.rust_name()));
                    }
                }
            }

            lines.push(format!("    unsafe {{ {}_delete(&obj); }}", prefix));
        } else {
            // 调用全局函数
            for func in ast.functions.iter().take(3) {
                let simple_args: Vec<String> = func.params.iter()
                    .map(|p| default_value_for_type(&p.rust_type))
                    .collect();
                let args_str = simple_args.join(", ");
                let func_call = if func.needs_unsafe() {
                    format!("unsafe {{ {}({}) }}", func.rust_name(), args_str)
                } else {
                    format!("{}({})", func.rust_name(), args_str)
                };

                if func.return_type.trim() == "void" {
                    lines.push(format!("    {};", func_call));
                } else {
                    lines.push(format!("    let _ = {};", func_call));
                }
            }
        }

        lines.push(format!(
            "    println!(\"Rust FFI: {} completed!\");",
            ast.source_name
        ));
        lines.push("}".to_string());
        lines.join("\n")
    }
}

impl Default for HiccCodegen {
    fn default() -> Self {
        Self::new()
    }
}

/// 为给定 Rust 类型返回一个简单的默认值（用于生成演示代码）
fn default_value_for_type(rust_type: &str) -> String {
    match rust_type {
        "i8" | "i16" | "i32" | "i64" | "u8" | "u16" | "u32" | "u64" | "isize" | "usize" => {
            "0".to_string()
        }
        "f32" | "f64" => "0.0".to_string(),
        "bool" => "false".to_string(),
        t if t.starts_with("*const") => "std::ptr::null()".to_string(),
        t if t.starts_with("*mut") => "std::ptr::null_mut()".to_string(),
        _ => "todo!()".to_string(),
    }
}
