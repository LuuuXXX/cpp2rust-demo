use std::collections::{BTreeSet, HashSet};
use std::path::{Component, Path, PathBuf};

use anyhow::Result;

use crate::ir::{Class, Function, FunctionKind, Method, MethodKind, Parameter, ParsedHeader};
use crate::parser::to_snake_case;
use crate::typemap::{is_raw_pointer_type, map_cpp_type_to_rust, normalize_cpp_type};

/// 生成输出项目的 Cargo.toml。
pub fn generate_output_cargo_toml(lib_name: &str) -> String {
    format!(
        r#"[package]
name = "{lib_name}"
version = "0.1.0"
edition = "2021"

[dependencies]
hicc = {{ version = "0.2" }}

[build-dependencies]
cc = "1.0"
hicc-build = {{ version = "0.2" }}
"#
    )
}

/// 生成输出项目的 build.rs。
pub fn generate_build_rs(
    input_dir: &Path,
    output_dir: &Path,
    lib_name: &str,
    headers: &[PathBuf],
    cpp_files: &[PathBuf],
) -> Result<String> {
    let cpp_dir = relative_path(output_dir, input_dir);
    let mut file_lines = String::new();
    for file in cpp_files {
        if let Some(name) = file.file_name().and_then(|value| value.to_str()) {
            file_lines.push_str(&format!("    cc_build.file(cpp_dir.join({name:?}));\n"));
        }
    }

    let mut rerun_lines = String::new();
    for file in cpp_files.iter().chain(headers.iter()) {
        if let Some(name) = file.file_name().and_then(|value| value.to_str()) {
            rerun_lines.push_str(&format!(
                "    println!(\"cargo::rerun-if-changed={}/{}\");\n",
                cpp_dir.replace('\\', "/"),
                name
            ));
        }
    }

    Ok(format!(
        "fn main() {{\n    let cpp_dir = std::path::PathBuf::from({cpp_dir:?});\n\n    let mut build = hicc_build::Build::new();\n    use std::ops::DerefMut;\n    let cc_build: &mut cc::Build = build.deref_mut();\n    cc_build.include(&cpp_dir);\n    cc_build.include(\".\");\n    cc_build.cpp(true);\n{file_lines}\n    build.rust_file(\"src/main.rs\").compile({lib_name:?});\n\n    println!(\"cargo::rustc-link-lib={lib_name}\");\n    #[cfg(not(all(target_os = \"windows\", target_env = \"msvc\")))]\n    println!(\"cargo::rustc-link-lib=stdc++\");\n    println!(\"cargo::rerun-if-changed=src/main.rs\");\n{rerun_lines}}}\n"
    ))
}

/// 生成 Rust hicc 绑定源码。
pub fn generate_rust_source(headers: &[ParsedHeader], lib_name: &str) -> Result<String> {
    let mut blocks = Vec::new();
    for header in headers {
        blocks.push(generate_header_bindings(header, lib_name));
    }
    blocks.push("fn main() {}".to_string());
    Ok(blocks.join("\n\n"))
}

fn generate_header_bindings(header: &ParsedHeader, lib_name: &str) -> String {
    let mut generated_shims = Vec::new();
    let functions = collect_export_functions(header, &mut generated_shims);
    let mut parts = Vec::new();

    parts.push(render_cpp_block(header, &generated_shims));

    for class in &header.classes {
        if has_importable_methods(class) {
            parts.push(render_import_class_block(class));
        }
    }

    parts.push(render_import_lib_block(header, &functions, lib_name));
    parts.join("\n\n")
}

fn collect_export_functions(
    header: &ParsedHeader,
    generated_shims: &mut Vec<GeneratedShim>,
) -> Vec<Function> {
    let mut functions = header
        .functions
        .iter()
        .filter(|function| !is_redundant_instance_wrapper(function, &header.classes))
        .cloned()
        .map(|function| classify_existing_function(function, &header.classes))
        .collect::<Vec<_>>();

    let mut existing_names = functions
        .iter()
        .map(|item| item.name.clone())
        .collect::<HashSet<_>>();

    for class in &header.classes {
        let constructors = class
            .methods
            .iter()
            .filter(|method| matches!(method.kind, MethodKind::Constructor))
            .collect::<Vec<_>>();
        for (index, constructor) in constructors.iter().enumerate() {
            let function_name = if index == 0 {
                format!("{}_new", to_snake_case(&class.name))
            } else {
                format!("{}_new_{}", to_snake_case(&class.name), index)
            };
            if existing_names.insert(function_name.clone()) {
                functions.push(Function {
                    name: function_name.clone(),
                    rust_name: function_name.clone(),
                    return_type: format!("{}*", class.name),
                    params: constructor.params.clone(),
                    kind: FunctionKind::Constructor {
                        class_name: class.name.clone(),
                    },
                    explicit_void: false,
                });
                generated_shims.push(GeneratedShim::constructor(
                    class,
                    constructor,
                    function_name,
                ));
            }
        }

        let delete_name = format!("{}_delete", to_snake_case(&class.name));
        if existing_names.insert(delete_name.clone()) {
            let self_param = Parameter {
                name: "self_".to_string(),
                cpp_type: format!("{}*", class.name),
            };
            functions.push(Function {
                name: delete_name.clone(),
                rust_name: delete_name.clone(),
                return_type: "void".to_string(),
                params: vec![self_param],
                kind: FunctionKind::Destructor {
                    class_name: class.name.clone(),
                },
                explicit_void: false,
            });
            generated_shims.push(GeneratedShim::destructor(class, delete_name));
        }

        for method in class.methods.iter().filter(|method| method.is_static) {
            let function_name = format!("{}_{}", to_snake_case(&class.name), method.name);
            if existing_names.insert(function_name.clone()) {
                functions.push(Function {
                    name: function_name.clone(),
                    rust_name: to_snake_case(&function_name),
                    return_type: method
                        .return_type
                        .clone()
                        .unwrap_or_else(|| "void".to_string()),
                    params: method.params.clone(),
                    kind: FunctionKind::StaticMethodShim {
                        class_name: class.name.clone(),
                        method_name: method.name.clone(),
                    },
                    explicit_void: false,
                });
                generated_shims.push(GeneratedShim::static_method(class, method, function_name));
            }
        }
    }

    functions.sort_by(|left, right| left.name.cmp(&right.name));
    functions
}

fn render_cpp_block(header: &ParsedHeader, generated_shims: &[GeneratedShim]) -> String {
    let mut lines = vec![
        "hicc::cpp! {".to_string(),
        format!("    #include \"{}\"", header.include_path),
    ];
    if !generated_shims.is_empty() {
        lines.push(String::new());
        for (index, shim) in generated_shims.iter().enumerate() {
            for line in shim.body.lines() {
                lines.push(format!("    {line}"));
            }
            if index + 1 != generated_shims.len() {
                lines.push(String::new());
            }
        }
    }
    lines.push("}".to_string());
    lines.join("\n")
}

fn render_import_class_block(class: &Class) -> String {
    let mut lines = vec![
        "hicc::import_class! {".to_string(),
        format!("    #[cpp(class = \"{}\")]", class.name),
        format!("    class {} {{", class.name),
    ];

    let methods = class
        .methods
        .iter()
        .filter(|method| matches!(method.kind, MethodKind::Regular) && !method.is_static)
        .collect::<Vec<_>>();

    for (index, method) in methods.iter().enumerate() {
        lines.push(format!(
            "        #[cpp(method = \"{}\")]",
            render_method_signature(method)
        ));
        lines.push(format!("        {};", render_rust_method(method)));
        if index + 1 != methods.len() {
            lines.push(String::new());
        }
    }

    lines.push("    }".to_string());
    lines.push("}".to_string());
    lines.join("\n")
}

fn render_import_lib_block(
    header: &ParsedHeader,
    functions: &[Function],
    lib_name: &str,
) -> String {
    let mut lines = vec![
        "hicc::import_lib! {".to_string(),
        format!("    #![link_name = \"{lib_name}\"]"),
    ];

    if !header.classes.is_empty() {
        lines.push(String::new());
        let mut class_names = BTreeSet::new();
        for class in &header.classes {
            class_names.insert(class.name.clone());
        }
        for class_name in class_names {
            lines.push(format!("    class {class_name};"));
        }
    }

    if !functions.is_empty() {
        lines.push(String::new());
        for (index, function) in functions.iter().enumerate() {
            lines.push(format!(
                "    #[cpp(func = \"{}\")]",
                render_function_signature(function, &header.classes)
            ));
            lines.push(format!("    {};", render_rust_function(function)));
            if index + 1 != functions.len() {
                lines.push(String::new());
            }
        }
    }

    lines.push("}".to_string());
    lines.join("\n")
}

fn render_method_signature(method: &Method) -> String {
    let return_type = normalize_cpp_type(method.return_type.as_deref().unwrap_or("void"));
    let params = render_cpp_params_with_names(&method.params, false);
    let const_suffix = if method.is_const { " const" } else { "" };
    format!("{return_type} {}({params}){const_suffix}", method.name)
}

fn render_rust_method(method: &Method) -> String {
    let receiver = if method.is_const {
        "&self"
    } else {
        "&mut self"
    };
    let mut parts = vec![receiver.to_string()];
    for param in &method.params {
        parts.push(format!(
            "{}: {}",
            rust_param_name(&param.name),
            map_cpp_type_to_rust(&param.cpp_type)
        ));
    }
    let return_type = map_cpp_type_to_rust(method.return_type.as_deref().unwrap_or("void"));
    if return_type == "()" {
        format!("fn {}({})", method.rust_name, parts.join(", "))
    } else {
        format!(
            "fn {}({}) -> {return_type}",
            method.rust_name,
            parts.join(", ")
        )
    }
}

fn render_function_signature(function: &Function, classes: &[Class]) -> String {
    let params = if function.params.is_empty() {
        if function.explicit_void && !is_class_related(function, classes) {
            "void".to_string()
        } else {
            String::new()
        }
    } else {
        function
            .params
            .iter()
            .map(|param| normalize_cpp_type(&param.cpp_type))
            .collect::<Vec<_>>()
            .join(", ")
    };
    format!(
        "{} {}({params})",
        normalize_cpp_type(&function.return_type),
        function.name
    )
}

fn render_rust_function(function: &Function) -> String {
    let safety = if is_unsafe_function(function) {
        "unsafe "
    } else {
        ""
    };
    let params = function
        .params
        .iter()
        .map(|param| {
            format!(
                "{}: {}",
                rust_param_name(&param.name),
                map_cpp_type_to_rust(&param.cpp_type)
            )
        })
        .collect::<Vec<_>>()
        .join(", ");
    let return_type = map_cpp_type_to_rust(&function.return_type);
    if return_type == "()" {
        format!("{safety}fn {}({params})", function.rust_name)
    } else {
        format!(
            "{safety}fn {}({params}) -> {return_type}",
            function.rust_name
        )
    }
}

fn render_cpp_params_with_names(params: &[Parameter], use_void: bool) -> String {
    if params.is_empty() {
        return if use_void {
            "void".to_string()
        } else {
            String::new()
        };
    }
    params
        .iter()
        .map(|param| format!("{} {}", normalize_cpp_type(&param.cpp_type), param.name))
        .collect::<Vec<_>>()
        .join(", ")
}

fn rust_param_name(name: &str) -> String {
    if name == "self" {
        "self_".to_string()
    } else {
        to_snake_case(name)
    }
}

fn has_importable_methods(class: &Class) -> bool {
    class
        .methods
        .iter()
        .any(|method| matches!(method.kind, MethodKind::Regular) && !method.is_static)
}

fn is_unsafe_function(function: &Function) -> bool {
    match function.kind {
        FunctionKind::Constructor { .. } => false,
        FunctionKind::Destructor { .. } => true,
        _ => {
            if function
                .params
                .iter()
                .any(|param| normalize_cpp_type(&param.cpp_type) == "const char*")
            {
                return true;
            }
            if function
                .params
                .iter()
                .any(|param| is_raw_pointer_type(&param.cpp_type))
            {
                return true;
            }
            is_raw_pointer_type(&function.return_type)
        }
    }
}

fn classify_existing_function(mut function: Function, classes: &[Class]) -> Function {
    for class in classes {
        let base = to_snake_case(&class.name);
        let return_type = normalize_cpp_type(&function.return_type);
        let class_ptr = format!("{}*", class.name);
        if function.name == format!("{base}_new") && return_type == class_ptr {
            function.kind = FunctionKind::Constructor {
                class_name: class.name.clone(),
            };
            return function;
        }
        if function.name == format!("{base}_delete")
            && function.params.len() == 1
            && normalize_cpp_type(&function.params[0].cpp_type) == class_ptr
        {
            function.kind = FunctionKind::Destructor {
                class_name: class.name.clone(),
            };
            return function;
        }
    }
    function
}

fn is_redundant_instance_wrapper(function: &Function, classes: &[Class]) -> bool {
    let first_param = match function.params.first() {
        Some(param) => normalize_cpp_type(&param.cpp_type),
        None => return false,
    };

    for class in classes {
        let class_ptr = format!("{}*", class.name);
        let const_class_ptr = format!("const {}*", class.name);
        if first_param != class_ptr && first_param != const_class_ptr {
            continue;
        }

        let prefix = format!("{}_", to_snake_case(&class.name));
        if !function.name.starts_with(&prefix) {
            continue;
        }
        let suffix = &function.name[prefix.len()..];
        if suffix == "new" || suffix.starts_with("new_") || suffix == "delete" {
            return false;
        }

        for method in class
            .methods
            .iter()
            .filter(|method| matches!(method.kind, MethodKind::Regular) && !method.is_static)
        {
            let direct = to_snake_case(&method.name);
            if suffix == direct || suffix.eq_ignore_ascii_case(&method.name) {
                return true;
            }
        }
    }

    false
}

fn is_class_related(function: &Function, classes: &[Class]) -> bool {
    classes.iter().any(|class| {
        let class_name = class.name.as_str();
        normalize_cpp_type(&function.return_type).contains(class_name)
            || function
                .params
                .iter()
                .any(|param| normalize_cpp_type(&param.cpp_type).contains(class_name))
    })
}

#[derive(Debug, Clone)]
struct GeneratedShim {
    body: String,
}

impl GeneratedShim {
    fn constructor(class: &Class, constructor: &Method, function_name: String) -> Self {
        let params = render_cpp_params_with_names(&constructor.params, false);
        let args = constructor
            .params
            .iter()
            .map(|param| param.name.clone())
            .collect::<Vec<_>>()
            .join(", ");
        let invocation = if args.is_empty() {
            format!("new {}()", class.name)
        } else {
            format!("new {}({args})", class.name)
        };
        Self {
            body: format!(
                "{}* {}({}) {{ return {}; }}",
                class.name, function_name, params, invocation
            ),
        }
    }

    fn destructor(class: &Class, function_name: String) -> Self {
        Self {
            body: format!(
                "void {}({}* self) {{ delete self; }}",
                function_name, class.name
            ),
        }
    }

    fn static_method(class: &Class, method: &Method, function_name: String) -> Self {
        let return_type = normalize_cpp_type(method.return_type.as_deref().unwrap_or("void"));
        let params = render_cpp_params_with_names(&method.params, false);
        let args = method
            .params
            .iter()
            .map(|param| param.name.clone())
            .collect::<Vec<_>>()
            .join(", ");
        let call = if args.is_empty() {
            format!("{}::{}()", class.name, method.name)
        } else {
            format!("{}::{}({args})", class.name, method.name)
        };
        let body = if return_type == "void" {
            format!("void {}({}) {{ {}; }}", function_name, params, call)
        } else {
            format!(
                "{return_type} {}({}) {{ return {}; }}",
                function_name, params, call
            )
        };
        Self { body }
    }
}

fn relative_path(from: &Path, to: &Path) -> String {
    let from = from.components().collect::<Vec<_>>();
    let to = to.components().collect::<Vec<_>>();
    let common_len = from
        .iter()
        .zip(to.iter())
        .take_while(|(left, right)| left == right)
        .count();

    let mut parts = Vec::new();
    for _ in common_len..from.len() {
        parts.push("..".to_string());
    }
    for component in &to[common_len..] {
        match component {
            Component::Normal(value) => parts.push(value.to_string_lossy().to_string()),
            Component::CurDir => {}
            Component::ParentDir => parts.push("..".to_string()),
            Component::RootDir => {}
            Component::Prefix(value) => parts.push(value.as_os_str().to_string_lossy().to_string()),
        }
    }
    if parts.is_empty() {
        ".".to_string()
    } else {
        parts.join("/")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_header_str;

    #[test]
    fn generates_free_function_bindings() {
        let parsed = parse_header_str(
            "hello_world.h",
            r#"
            void hello_world(void);
            "#,
        )
        .unwrap();
        let rust = generate_rust_source(&[parsed], "hello_world").unwrap();
        assert!(rust.contains("#include \"hello_world.h\""));
        assert!(rust.contains("#[cpp(func = \"void hello_world(void)\")]"));
        assert!(rust.contains("fn hello_world();"));
    }

    #[test]
    fn generates_class_bindings_and_filters_instance_wrappers() {
        let parsed = parse_header_str(
            "class_basic.h",
            r#"
            class Counter;
            struct Counter* counter_new(void);
            void counter_delete(struct Counter* self);
            int counter_get(struct Counter* self);
            class Counter {
            public:
                Counter();
                ~Counter();
                int get() const;
                void increment();
            };
            "#,
        )
        .unwrap();
        let rust = generate_rust_source(&[parsed], "class_basic").unwrap();
        assert!(rust.contains("#[cpp(class = \"Counter\")]"));
        assert!(rust.contains("fn get(&self) -> i32;"));
        assert!(rust.contains("unsafe fn counter_delete(self_: *mut Counter);"));
        assert!(!rust.contains("counter_get(self_"));
    }

    #[test]
    fn generates_static_method_shims_when_missing() {
        let parsed = parse_header_str(
            "class_static.h",
            r#"
            class Counter {
            public:
                Counter();
                static int getInstanceCount();
            };
            "#,
        )
        .unwrap();
        let rust = generate_rust_source(&[parsed], "class_static").unwrap();
        assert!(
            rust.contains("int counter_getInstanceCount() { return Counter::getInstanceCount(); }")
        );
        assert!(rust.contains("fn counter_get_instance_count() -> i32;"));
    }

    #[test]
    fn build_rs_uses_relative_cpp_dir() {
        let build_rs = generate_build_rs(
            Path::new("/repo/examples/001_hello_world/cpp"),
            Path::new("/repo/examples/001_hello_world/rust_hicc"),
            "hello_world",
            &[PathBuf::from(
                "/repo/examples/001_hello_world/cpp/hello_world.h",
            )],
            &[PathBuf::from(
                "/repo/examples/001_hello_world/cpp/hello_world.cpp",
            )],
        )
        .unwrap();
        assert!(build_rs.contains("PathBuf::from(\"../cpp\")"));
        assert!(build_rs.contains("cargo::rerun-if-changed=../cpp/hello_world.cpp"));
    }
}
