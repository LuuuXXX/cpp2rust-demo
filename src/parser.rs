use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use regex::Regex;

use crate::ir::{
    Class, Function, FunctionKind, Method, MethodKind, Parameter, ParsedHeader, TypedefAlias,
};
use crate::typemap::{map_cpp_type_to_rust, normalize_cpp_type};

/// 解析头文件。
pub fn parse_header_file(path: &Path) -> Result<ParsedHeader> {
    let source = fs::read_to_string(path)
        .with_context(|| format!("failed to read header {}", path.display()))?;
    let header_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("generated.h")
        .to_string();
    parse_header_str(&header_name, &source)
}

/// 解析头文件字符串，供测试和 CLI 共用。
pub fn parse_header_str(header_name: &str, source: &str) -> Result<ParsedHeader> {
    let cleaned = sanitize_source(&strip_comments(source));
    // 匹配可选 template<...> 前缀 + class/struct + name + 可选 final + 可选基类 + 正文
    let class_regex = Regex::new(
        r"(?s)(template\s*<[^>]*>\s*)?(class|struct)\s+([A-Za-z_]\w*)\s*(?:final\s*)?(?:\:[^{]+)?\{(.*?)\};",
    )
    .unwrap();

    let mut classes = Vec::new();
    let mut all_friend_names: std::collections::HashSet<String> = std::collections::HashSet::new();
    for captures in class_regex.captures_iter(&cleaned) {
        let is_template = captures.get(1).is_some(); // template<...> 前缀
        let kind = captures.get(2).unwrap().as_str();
        let name = captures.get(3).unwrap().as_str();
        let body = captures.get(4).unwrap().as_str();
        // 跳过模板类（不能直接 FFI）
        if is_template {
            continue;
        }
        let (class, friend_names) = parse_class(kind, name, body);
        for fn_name in friend_names {
            all_friend_names.insert(fn_name);
        }
        classes.push(class);
    }

    // 先从整个源码（含类定义内部）扫描 typedef
    let typedefs = parse_typedefs(source);

    let source_without_classes = class_regex.replace_all(&cleaned, " ");
    let mut functions = Vec::new();
    for statement in split_top_level_statements(&source_without_classes) {
        if let Some(mut function) = parse_free_function(&statement) {
            if all_friend_names.contains(&function.name) {
                function.is_friend = true;
            }
            functions.push(function);
        }
    }

    Ok(ParsedHeader {
        header_name: header_name.to_string(),
        include_path: header_name.to_string(),
        functions,
        classes,
        typedefs,
    })
}

/// 提取所有函数指针 typedef，如 `typedef int (*IntBinaryOp)(int, int)` → `IntBinaryOp`。
fn parse_typedefs(source: &str) -> Vec<TypedefAlias> {
    let re = Regex::new(
        r"typedef\s+([A-Za-z_][\w\s\*]*?)\s*\(\s*\*\s*([A-Za-z_]\w*)\s*\)\s*\(([^)]*)\)",
    )
    .unwrap();
    let mut result = Vec::new();
    for caps in re.captures_iter(source) {
        let ret_cpp = caps.get(1).unwrap().as_str().trim();
        let name = caps.get(2).unwrap().as_str().trim();
        let args_raw = caps.get(3).unwrap().as_str().trim();

        let ret_rust = map_cpp_type_to_rust(ret_cpp);
        let arg_types: Vec<String> = if args_raw.is_empty() || args_raw == "void" {
            Vec::new()
        } else {
            split_arguments(args_raw)
                .iter()
                .map(|arg| {
                    // 参数可能有名字，也可能只有类型
                    let arg = arg.trim();
                    if let Some((ty, _name)) = split_type_and_name(arg) {
                        map_cpp_type_to_rust(ty)
                    } else {
                        map_cpp_type_to_rust(arg)
                    }
                })
                .collect()
        };

        let rust_type = if arg_types.is_empty() {
            if ret_rust == "()" {
                "extern \"C\" fn()".to_string()
            } else {
                format!("extern \"C\" fn() -> {ret_rust}")
            }
        } else {
            let args_str = arg_types.join(", ");
            if ret_rust == "()" {
                format!("extern \"C\" fn({args_str})")
            } else {
                format!("extern \"C\" fn({args_str}) -> {ret_rust}")
            }
        };

        result.push(TypedefAlias {
            name: name.to_string(),
            rust_type,
        });
    }
    result
}

fn parse_class(kind: &str, name: &str, body: &str) -> (Class, Vec<String>) {
    let default_public = kind == "struct";
    let mut is_public = default_public;
    let mut methods = Vec::new();
    let mut friend_fn_names: Vec<String> = Vec::new();

    // 先剥离内联方法体（{ ... }），防止方法体中的 `;` 破坏分割逻辑
    let body_no_inline = strip_inline_bodies(body);

    let normalized_body = Regex::new(r"(?m)\b(public|private|protected):")
        .unwrap()
        .replace_all(&body_no_inline, "$1:;")
        .into_owned();

    for chunk in normalized_body.split(';') {
        let statement = chunk.trim();
        if statement.is_empty() {
            continue;
        }
        match statement {
            "public:" => {
                is_public = true;
                continue;
            }
            "private:" | "protected:" => {
                is_public = false;
                continue;
            }
            _ => {}
        }

        // friend 声明永远跳过（无论可见性），但记录其函数名
        let stripped = statement.trim_start();
        if stripped.starts_with("friend ") || stripped.starts_with("friend\t") {
            if let Some(fn_name) = extract_friend_fn_name(stripped) {
                friend_fn_names.push(fn_name);
            }
            continue;
        }

        if !is_public || !statement.contains('(') {
            continue;
        }

        // 跳过函数指针成员变量（如 std::function 字段），这些不是方法
        if is_function_pointer_field(statement) {
            continue;
        }

        if let Some(method) = parse_method(name, statement) {
            methods.push(method);
        }
    }

    (
        Class {
            name: name.to_string(),
            methods,
        },
        friend_fn_names,
    )
}

/// 剥离类体中的内联方法体 `{ ... }`，保留声明部分。
/// 例如：`int size() const { return data.size(); }` → `int size() const ;`
/// 注意：用 `;` 替换每个被剥离的 `}` 的闭合位置，确保后续 `;` 分割正确工作。
fn strip_inline_bodies(body: &str) -> String {
    let mut result = String::new();
    let mut depth = 0usize;
    for ch in body.chars() {
        match ch {
            '{' => depth += 1,
            '}' => {
                if depth > 0 {
                    depth -= 1;
                    // 方法体关闭时插入 `;` 使后续按 `;` 分割时每个声明独立
                    if depth == 0 {
                        result.push(';');
                    }
                }
            }
            _ => {
                if depth == 0 {
                    result.push(ch);
                }
            }
        }
    }
    result
}

/// 判断是否为函数指针/std::function 成员变量（不是方法声明）。
fn is_function_pointer_field(statement: &str) -> bool {
    // 匹配 `int (*name)(...)` 模式
    let s = statement.trim();
    s.contains("(*") || s.contains("std::function<") || s.contains("std::function <")
}

fn parse_method(class_name: &str, statement: &str) -> Option<Method> {
    let declaration = normalize_decl(statement);
    let open = declaration.find('(')?;
    let close = declaration.rfind(')')?;
    let before = declaration[..open].trim();
    let after = declaration[close + 1..].trim();
    let params_text = declaration[open + 1..close].trim();
    let params = parse_params(params_text);

    let is_static = before.starts_with("static ");
    let before = before.strip_prefix("static ").unwrap_or(before).trim();
    let before = strip_leading_qualifiers(before);
    let is_const = after.split_whitespace().any(|item| item == "const");

    if before == class_name {
        return Some(Method {
            name: class_name.to_string(),
            rust_name: "new".to_string(),
            return_type: None,
            params,
            kind: MethodKind::Constructor,
            is_const: false,
            is_static,
            is_operator: false,
        });
    }
    if before == format!("~{class_name}") {
        return Some(Method {
            name: format!("~{class_name}"),
            rust_name: "delete".to_string(),
            return_type: None,
            params,
            kind: MethodKind::Destructor,
            is_const: false,
            is_static,
            is_operator: false,
        });
    }

    let (return_type, name) = split_return_type_and_name(before)?;
    let is_operator = name.contains("operator");
    Some(Method {
        name: name.to_string(),
        rust_name: to_snake_case(name),
        return_type: Some(normalize_cpp_type(return_type)),
        params,
        kind: MethodKind::Regular,
        is_const,
        is_static,
        is_operator,
    })
}

fn parse_free_function(statement: &str) -> Option<Function> {
    let declaration = normalize_decl(statement);
    if declaration.is_empty()
        || !declaration.contains('(')
        || declaration.starts_with('#')
        || declaration.ends_with('{')
        || declaration == "extern \"C\""
        || declaration == "extern 'C'"
        || declaration == "extern \"C\" {"
        || declaration == "extern 'C' {"
        || declaration == "}"
        || declaration.starts_with("typedef ")
        || declaration.starts_with("using ")
        || declaration.starts_with("namespace ")
        // 跳过函数指针声明：如 `int (*Name)(int, int)`
        || declaration.contains("(*")
    {
        return None;
    }

    let open = declaration.find('(')?;
    let close = declaration.rfind(')')?;
    let before = strip_leading_qualifiers(declaration[..open].trim());
    let params_text = declaration[open + 1..close].trim();
    let explicit_void = params_text == "void";
    let params = parse_params(params_text);
    let (return_type, name) = split_return_type_and_name(before)?;

    // 跳过运算符函数（作为顶层函数不太可能出现，但防御性检查）
    if name.contains("operator") {
        return None;
    }

    Some(Function {
        name: name.to_string(),
        rust_name: to_snake_case(name),
        return_type: normalize_cpp_type(return_type),
        params,
        kind: FunctionKind::Free,
        explicit_void,
        is_friend: false,
    })
}

fn parse_params(params_text: &str) -> Vec<Parameter> {
    if params_text.is_empty() || params_text == "void" {
        return Vec::new();
    }

    split_arguments(params_text)
        .into_iter()
        .enumerate()
        .filter_map(|(index, raw)| parse_param(&raw, index))
        .collect()
}

fn parse_param(raw: &str, index: usize) -> Option<Parameter> {
    let without_default = raw.split('=').next()?.trim();
    if without_default.is_empty() {
        return None;
    }
    if without_default == "..." {
        return Some(Parameter {
            name: format!("arg{index}"),
            cpp_type: "...".to_string(),
        });
    }

    // 函数指针参数 `int (*fn)(int, int)` 或 `int (*)(int, int)` → 提取别名或用 opaque 类型
    if without_default.contains("(*") {
        let fn_ptr_name = extract_fn_ptr_name(without_default);
        return Some(Parameter {
            name: fn_ptr_name.unwrap_or_else(|| format!("arg{index}")),
            cpp_type: "fn_ptr".to_string(),
        });
    }

    if let Some((cpp_type, name)) = split_type_and_name(without_default) {
        return Some(Parameter {
            name: sanitize_param_name(name),
            cpp_type: normalize_cpp_type(cpp_type),
        });
    }

    Some(Parameter {
        name: format!("arg{index}"),
        cpp_type: normalize_cpp_type(without_default),
    })
}

/// 从 `int (*fn_name)(int, int)` 中提取参数名 `fn_name`。
fn extract_fn_ptr_name(raw: &str) -> Option<String> {
    let start = raw.find("(*")? + 2;
    let end = raw[start..].find(')')?;
    let name = raw[start..start + end]
        .trim()
        .trim_start_matches('*')
        .trim();
    if name.is_empty() {
        None
    } else {
        Some(sanitize_param_name(name))
    }
}

fn normalize_decl(statement: &str) -> String {
    let single_line = statement.replace('\n', " ");
    let single_line = Regex::new(r"\s+").unwrap().replace_all(&single_line, " ");
    single_line
        .trim()
        .trim_end_matches(';')
        .trim_end_matches('{')
        .trim()
        .split(" = ")
        .next()
        .unwrap_or("")
        .trim()
        .to_string()
}

fn strip_leading_qualifiers(value: &str) -> &str {
    let mut result = value.trim();
    let qualifiers = [
        "inline ",
        "constexpr ",
        "virtual ",
        "friend ",
        "extern ",
        "explicit ",
        "override ",
        "final ",
        "[[nodiscard]] ",
    ];
    loop {
        let prev = result;
        for qualifier in &qualifiers {
            if let Some(stripped) = result.strip_prefix(qualifier) {
                result = stripped.trim();
            }
        }
        if result == prev {
            break;
        }
    }
    result
}

/// 从 `friend int foo(...)` 这样的声明中提取函数名 `foo`。
/// 返回 None 表示这是 `friend class Foo` 之类的非函数声明。
fn extract_friend_fn_name(friend_decl: &str) -> Option<String> {
    // 去掉 "friend " 或 "friend\t" 前缀
    let after_friend = friend_decl
        .strip_prefix("friend ")
        .or_else(|| friend_decl.strip_prefix("friend\t"))?
        .trim();
    // 去掉限定符（inline / constexpr 等）
    let stripped = strip_leading_qualifiers(after_friend);
    // 找到第一个 `(`
    let paren_pos = stripped.find('(')?;
    let before_paren = stripped[..paren_pos].trim();
    // 提取最后一个 token 作为函数名
    let fn_name = before_paren.split_whitespace().last()?;
    // 排除 `friend class Foo` / `friend struct Foo`
    if fn_name == "class" || fn_name == "struct" {
        return None;
    }
    // 排除 operator
    if fn_name.contains("operator") {
        return None;
    }
    // 去掉可能的 `*` 前缀（如 `int* foo(...)` 中 `*foo` 不太可能，但防御）
    let fn_name = fn_name.trim_start_matches('*');
    if fn_name.is_empty() {
        return None;
    }
    Some(fn_name.to_string())
}

fn split_return_type_and_name(value: &str) -> Option<(&str, &str)> {
    let value = value.trim();
    let name_start = value.rfind(|ch: char| ch.is_whitespace())?;
    let return_type = value[..name_start].trim();
    let name = value[name_start + 1..].trim();
    if return_type.is_empty() || name.is_empty() {
        return None;
    }
    Some((return_type, name))
}

fn split_type_and_name(value: &str) -> Option<(&str, &str)> {
    let chars = value.char_indices().collect::<Vec<_>>();
    for (index, ch) in chars.into_iter().rev() {
        if ch.is_whitespace() {
            let cpp_type = value[..index].trim();
            let name = value[index + ch.len_utf8()..].trim();
            if !cpp_type.is_empty() && !name.is_empty() {
                return Some((cpp_type, name));
            }
        }
    }
    None
}

fn sanitize_param_name(name: &str) -> String {
    let cleaned = name.trim_start_matches('*').trim_start_matches('&');
    if cleaned.is_empty() {
        "arg".to_string()
    } else if cleaned == "self" {
        "self_".to_string()
    } else {
        cleaned.to_string()
    }
}

fn split_arguments(input: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut angle = 0i32;
    let mut paren = 0i32;

    for ch in input.chars() {
        match ch {
            '<' => angle += 1,
            '>' => angle -= 1,
            '(' => paren += 1,
            ')' => paren -= 1,
            ',' if angle == 0 && paren == 0 => {
                parts.push(current.trim().to_string());
                current.clear();
                continue;
            }
            _ => {}
        }
        current.push(ch);
    }

    if !current.trim().is_empty() {
        parts.push(current.trim().to_string());
    }
    parts
}

fn split_top_level_statements(source: &str) -> Vec<String> {
    let mut statements = Vec::new();
    let mut current = String::new();
    let mut brace_depth = 0i32;

    for ch in source.chars() {
        match ch {
            '{' => brace_depth += 1,
            '}' => brace_depth -= 1,
            ';' if brace_depth == 0 => {
                if !current.trim().is_empty() {
                    statements.push(current.trim().to_string());
                }
                current.clear();
                continue;
            }
            _ => {}
        }
        current.push(ch);
    }

    if !current.trim().is_empty() {
        statements.push(current.trim().to_string());
    }
    statements
}

fn strip_comments(source: &str) -> String {
    let line = Regex::new(r"//.*").unwrap().replace_all(source, "");
    Regex::new(r"(?s)/\*.*?\*/")
        .unwrap()
        .replace_all(&line, "")
        .into_owned()
}

fn sanitize_source(source: &str) -> String {
    let without_preprocessor = source
        .lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n");
    let without_extern = Regex::new(r#"extern\s+["']C["']\s*\{"#)
        .unwrap()
        .replace_all(&without_preprocessor, "")
        .into_owned();
    Regex::new(r"(?m)^\s*}\s*$")
        .unwrap()
        .replace_all(&without_extern, "")
        .into_owned()
}

pub fn to_snake_case(value: &str) -> String {
    let mut output = String::new();
    for (index, ch) in value.chars().enumerate() {
        if ch.is_uppercase() {
            if index > 0 {
                output.push('_');
            }
            for lower in ch.to_lowercase() {
                output.push(lower);
            }
        } else {
            output.push(ch);
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_free_functions() {
        let header = r#"
            int add_int(int a, int b);
            const char* add_strings(const char* a, const char* b);
        "#;
        let parsed = parse_header_str("function_overload.h", header).unwrap();
        assert_eq!(parsed.functions.len(), 2);
        assert_eq!(parsed.functions[0].name, "add_int");
        assert_eq!(parsed.functions[1].params[0].cpp_type, "const char*");
    }

    #[test]
    fn parses_class_methods() {
        let header = r#"
            class Counter {
            public:
                Counter();
                ~Counter();
                int get() const;
                void increment();
                static int instances();
            };
        "#;
        let parsed = parse_header_str("counter.h", header).unwrap();
        let class = &parsed.classes[0];
        assert_eq!(class.name, "Counter");
        assert_eq!(class.methods.len(), 5);
        assert!(matches!(class.methods[0].kind, MethodKind::Constructor));
        assert!(matches!(class.methods[1].kind, MethodKind::Destructor));
        assert!(class.methods[2].is_const);
        assert!(class.methods[4].is_static);
    }

    #[test]
    fn parses_struct_pointer_params() {
        let function = parse_free_function("void counter_delete(struct Counter* self)").unwrap();
        assert_eq!(function.return_type, "void");
        assert_eq!(function.params[0].cpp_type, "Counter*");
        assert_eq!(function.params[0].name, "self_");
    }

    #[test]
    fn snake_case_conversion_is_stable() {
        assert_eq!(to_snake_case("getValue"), "get_value");
        assert_eq!(to_snake_case("sum3"), "sum3");
    }

    #[test]
    fn detects_operator_methods() {
        let header = r#"
            class Number {
            public:
                Number(int v);
                ~Number();
                int getValue() const;
                Number operator+(const Number& other) const;
                Number& operator++();
            };
        "#;
        let parsed = parse_header_str("number.h", header).unwrap();
        let class = &parsed.classes[0];
        assert!(!class.methods[2].is_operator); // getValue
        assert!(class.methods[3].is_operator); // operator+
        assert!(class.methods[4].is_operator); // operator++
    }

    #[test]
    fn skips_friend_declarations_in_class() {
        let header = r#"
            class MyClass {
                friend int friend_add(const MyClass* a, const MyClass* b);
            public:
                MyClass(int v);
                int getValue() const;
            };
        "#;
        let parsed = parse_header_str("myclass.h", header).unwrap();
        let class = &parsed.classes[0];
        // friend 声明不应出现在 methods 中
        assert_eq!(class.methods.len(), 2);
        assert!(matches!(class.methods[0].kind, MethodKind::Constructor));
    }

    #[test]
    fn parses_typedef_function_pointer() {
        let header = r#"
            typedef int (*IntBinaryOp)(int, int);
            typedef void (*Callback)(void);
        "#;
        let parsed = parse_header_str("lambda.h", header).unwrap();
        assert_eq!(parsed.typedefs.len(), 2);
        assert_eq!(parsed.typedefs[0].name, "IntBinaryOp");
        assert!(parsed.typedefs[0].rust_type.contains("fn(i32, i32)"));
        assert_eq!(parsed.typedefs[1].name, "Callback");
    }

    #[test]
    fn skips_function_pointer_free_decls() {
        let header = r#"
            typedef int (*IntBinaryOp)(int, int);
            int apply_operation(int a, int b, IntBinaryOp op);
        "#;
        let parsed = parse_header_str("lambda.h", header).unwrap();
        // typedef 不产生 function，apply_operation 产生 1 个
        assert_eq!(parsed.functions.len(), 1);
        assert_eq!(parsed.functions[0].name, "apply_operation");
    }

    #[test]
    fn strips_inline_method_bodies() {
        let header = r#"
            class Stack {
            public:
                Stack() = default;
                int size() const { return static_cast<int>(data.size()); }
                bool empty() const { return data.empty(); }
                void push(int value) { data.push(value); }
                int top() const { return data.top(); }
                void pop() { data.pop(); }
            };
        "#;
        let parsed = parse_header_str("stack.h", header).unwrap();
        let class = &parsed.classes[0];
        // constructor + 5 methods
        assert_eq!(class.methods.len(), 6);
        // size, empty, top are const
        assert!(class.methods[1].is_const); // size
        assert_eq!(class.methods[1].rust_name, "size");
        assert_eq!(class.methods[1].return_type.as_deref(), Some("int"));
    }

    #[test]
    fn skips_template_classes() {
        let header = r#"
            template<typename T>
            class Stack {
            public:
                void push(T value);
                T top() const;
            };
            class IntStack {
            public:
                IntStack();
                void push(int value);
            };
        "#;
        let parsed = parse_header_str("stack.h", header).unwrap();
        // template class Stack should be skipped, IntStack kept
        assert_eq!(parsed.classes.len(), 1);
        assert_eq!(parsed.classes[0].name, "IntStack");
    }
}
