use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use regex::Regex;

use crate::ir::{Class, Function, FunctionKind, Method, MethodKind, Parameter, ParsedHeader};
use crate::typemap::normalize_cpp_type;

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
    let class_regex =
        Regex::new(r"(?s)(class|struct)\s+([A-Za-z_]\w*)\s*(?:final\s*)?(?:\:[^{]+)?\{(.*?)\};")
            .unwrap();

    let mut classes = Vec::new();
    for captures in class_regex.captures_iter(&cleaned) {
        let kind = captures.get(1).unwrap().as_str();
        let name = captures.get(2).unwrap().as_str();
        let body = captures.get(3).unwrap().as_str();
        classes.push(parse_class(kind, name, body));
    }

    let source_without_classes = class_regex.replace_all(&cleaned, " ");
    let mut functions = Vec::new();
    for statement in split_top_level_statements(&source_without_classes) {
        if let Some(function) = parse_free_function(&statement) {
            functions.push(function);
        }
    }

    Ok(ParsedHeader {
        header_name: header_name.to_string(),
        include_path: header_name.to_string(),
        functions,
        classes,
    })
}

fn parse_class(kind: &str, name: &str, body: &str) -> Class {
    let default_public = kind == "struct";
    let mut is_public = default_public;
    let mut methods = Vec::new();
    let normalized_body = Regex::new(r"(?m)\b(public|private|protected):")
        .unwrap()
        .replace_all(body, "$1:;")
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

        if !is_public || !statement.contains('(') {
            continue;
        }

        if let Some(method) = parse_method(name, statement) {
            methods.push(method);
        }
    }

    Class {
        name: name.to_string(),
        methods,
    }
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
        });
    }

    let (return_type, name) = split_return_type_and_name(before)?;
    Some(Method {
        name: name.to_string(),
        rust_name: to_snake_case(name),
        return_type: Some(normalize_cpp_type(return_type)),
        params,
        kind: MethodKind::Regular,
        is_const,
        is_static,
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

    Some(Function {
        name: name.to_string(),
        rust_name: to_snake_case(name),
        return_type: normalize_cpp_type(return_type),
        params,
        kind: FunctionKind::Free,
        explicit_void,
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
    for qualifier in ["inline ", "constexpr ", "virtual ", "friend ", "extern "] {
        if let Some(stripped) = result.strip_prefix(qualifier) {
            result = stripped.trim();
        }
    }
    result
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
}
