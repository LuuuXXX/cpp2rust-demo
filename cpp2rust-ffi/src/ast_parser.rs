use anyhow::{anyhow, Result};
use clang::{Clang, Entity, EntityKind, EvaluationResult, Index, TranslationUnit, TypeKind};
use std::path::Path;

use crate::types::*;

/// 解析 C++ 预处理文件（.c2rust 或 .cpp），提取 AST 信息
pub fn parse_cpp_file(file: &Path) -> Result<CppAst> {
    let clang = Clang::new().map_err(|e| anyhow!("Failed to initialize libclang: {}", e))?;
    let index = Index::new(&clang, false, false);

    let source_name = file
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .trim_end_matches(".cpp")
        .trim_end_matches(".c2rust")
        .to_string();

    // C++ 解析参数
    let args: Vec<&str> = vec!["-x", "c++", "-std=c++17", "-fparse-all-comments"];

    let tu = index
        .parser(file)
        .arguments(&args)
        .detailed_preprocessing_record(true)
        .skip_function_bodies(false)
        .parse()
        .map_err(|e| anyhow!("Failed to parse C++ file: {:?}", e))?;

    let mut ast = CppAst {
        source_name,
        ..Default::default()
    };

    // 收集 includes
    collect_includes(&tu, file, &mut ast);

    // 遍历顶层 AST 节点（直接处理根的子节点，跳过根本身）
    let root = tu.get_entity();
    let mut namespace_stack = Vec::new();
    for child in root.get_children() {
        visit_entity(&child, &mut ast, &mut namespace_stack, file);
    }

    // 去重：相同名称+签名的函数只保留一份（定义优先于声明）
    dedup_functions(&mut ast.functions);
    dedup_classes(&mut ast.classes);

    Ok(ast)
}

/// 去重函数（保留第一次出现的）
fn dedup_functions(functions: &mut Vec<crate::types::CppFunction>) {
    let mut seen = std::collections::HashSet::new();
    functions.retain(|f| {
        let key = format!("{}:{}:{}", f.name,
            f.params.iter().map(|p| &p.cpp_type as &str).collect::<Vec<_>>().join(","),
            &f.return_type
        );
        seen.insert(key)
    });
}

/// 去重类（保留第一次出现的）
fn dedup_classes(classes: &mut Vec<crate::types::CppClass>) {
    let mut seen = std::collections::HashSet::new();
    classes.retain(|c| seen.insert(c.name.clone()));
}
/// 收集 #include 指令
fn collect_includes(tu: &TranslationUnit, source_file: &Path, ast: &mut CppAst) {
    // 从 TU 的预处理包含中提取
    let file = tu.get_file(source_file);
    if file.is_none() {
        return;
    }

    // 收集顶级 includes（简单策略：从 AST 节点的源文件推断）
    // 对于工具目标，我们主要从 cpp 源文件名推断头文件
    let stem = source_file
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .trim_end_matches(".c2rust");

    // 检查是否有对应的 .h 文件
    let header = format!("{}.h", stem);
    ast.includes.push(format!("\"{}\"", header));
}

/// 判断一个路径是否是用户代码（非系统头文件）
fn is_user_file(file_path: &std::path::Path) -> bool {
    let path_str = file_path.to_string_lossy();
    // 跳过系统头文件
    if path_str.contains("/usr/include")
        || path_str.contains("/usr/lib")
        || path_str.contains("/usr/local/include")
        || path_str.contains("/usr/local/lib")
        || path_str.contains("/include/c++")
        || path_str.contains("/lib/gcc")
        || path_str.contains("bits/")
        || path_str.contains("__")
    {
        return false;
    }
    true
}

/// 遍历 AST 实体，提取 C++ 构造
fn visit_entity(
    entity: &Entity,
    ast: &mut CppAst,
    namespace_stack: &mut Vec<String>,
    source_file: &Path,
) {
    // 只处理来自用户代码的节点（非系统头文件）
    if let Some(loc) = entity.get_location() {
        let file_loc = loc.get_file_location();
        if let Some(file) = file_loc.file {
            let file_path = file.get_path();
            if !is_user_file(&file_path) {
                return;
            }
        }
    } else {
        // 无位置信息的节点（如 TranslationUnitDecl）允许继续
    }

    match entity.get_kind() {
        EntityKind::Namespace => {
            // 跳过标准命名空间
            let ns_name = entity.get_name().unwrap_or_default();
            if matches!(ns_name.as_str(), "std" | "__gnu_cxx" | "__cxxabiv1" | "abi") {
                return;
            }
            namespace_stack.push(ns_name);
            for child in entity.get_children() {
                visit_entity(&child, ast, namespace_stack, source_file);
            }
            namespace_stack.pop();
        }

        EntityKind::StructDecl | EntityKind::ClassDecl => {
            if let Some(class) = extract_class(entity, namespace_stack) {
                ast.classes.push(class);
            }
        }

        EntityKind::ClassTemplate => {
            // 模板类本身不处理，只处理实例化
        }

        EntityKind::ClassTemplatePartialSpecialization => {
            if let Some(class) = extract_class(entity, namespace_stack) {
                let mut class = class;
                class.is_template_specialization = true;
                ast.classes.push(class);
            }
        }

        EntityKind::FunctionDecl => {
            if let Some(func) = extract_function(entity, namespace_stack) {
                ast.functions.push(func);
            }
        }

        EntityKind::FunctionTemplate => {
            // 函数模板：尝试提取已实例化的版本
            // 模板函数声明本身跳过
        }

        EntityKind::EnumDecl => {
            if let Some(enum_) = extract_enum(entity, namespace_stack) {
                ast.enums.push(enum_);
            }
        }

        EntityKind::VarDecl => {
            if let Some(cnst) = extract_const(entity) {
                ast.consts.push(cnst);
            }
        }

        EntityKind::TypedefDecl | EntityKind::TypeAliasDecl => {
            // 类型别名：暂不处理
        }

        EntityKind::LinkageSpec => {
            // extern "C" 块：处理其中的函数
            for child in entity.get_children() {
                visit_entity(&child, ast, namespace_stack, source_file);
            }
        }

        _ => {
            // 其他顶级节点：不再深度递归（防止栈溢出）
        }
    }
}

/// 从 AST 节点提取类信息
fn extract_class(entity: &Entity, namespace_stack: &[String]) -> Option<CppClass> {
    let name = entity.get_name()?;
    if name.is_empty() {
        return None; // 跳过匿名类
    }

    // 跳过前向声明（没有定义体）
    if !entity.is_definition() {
        return None;
    }

    let mut class = CppClass::new(&name);
    class.namespace = namespace_stack.to_vec();

    // 提取基类
    for child in entity.get_children() {
        if child.get_kind() == EntityKind::BaseSpecifier {
            if let Some(base_type) = child.get_type() {
                let base_name = get_type_name(&base_type);
                if !base_name.is_empty() {
                    class.base_classes.push(base_name);
                }
            }
        }
    }

    // 提取成员函数
    for child in entity.get_children() {
        match child.get_kind() {
            EntityKind::Constructor => {
                if let Some(method) = extract_constructor(&child) {
                    class.methods.push(method);
                }
            }
            EntityKind::Destructor => {
                if let Some(method) = extract_destructor(&child) {
                    class.methods.push(method);
                }
            }
            EntityKind::Method => {
                if let Some(method) = extract_method(&child) {
                    if method.is_pure_virtual {
                        class.is_abstract = true;
                    }
                    class.methods.push(method);
                }
            }
            _ => {}
        }
    }

    Some(class)
}

/// 提取构造函数
fn extract_constructor(entity: &Entity) -> Option<CppMethod> {
    let mut method = CppMethod::new("constructor", "void");
    method.is_constructor = true;

    // 参数
    for child in entity.get_children() {
        if child.get_kind() == EntityKind::ParmDecl {
            if let Some(param) = extract_param(&child) {
                method.params.push(param);
            }
        }
    }

    // 构建签名
    let params_str = method
        .params
        .iter()
        .map(|p| format!("{} {}", p.cpp_type, p.name))
        .collect::<Vec<_>>()
        .join(", ");
    method.cpp_signature = format!("constructor({})", params_str);

    Some(method)
}

/// 提取析构函数
fn extract_destructor(entity: &Entity) -> Option<CppMethod> {
    let mut method = CppMethod::new("destructor", "void");
    method.is_destructor = true;
    method.cpp_signature = "destructor()".to_string();
    Some(method)
}

/// 提取成员函数
fn extract_method(entity: &Entity) -> Option<CppMethod> {
    let name = entity.get_name()?;

    // 跳过操作符（单独处理）
    // let _is_operator = name.starts_with("operator");

    let return_type = if let Some(ty) = entity.get_result_type() {
        get_type_name(&ty)
    } else {
        "void".to_string()
    };

    let mut method = CppMethod::new(&name, &return_type);
    method.is_const = entity.is_const_method();
    method.is_virtual = entity.is_virtual_method();
    method.is_pure_virtual = entity.is_pure_virtual_method();
    method.is_static = entity.is_static_method();

    // 参数
    for child in entity.get_children() {
        if child.get_kind() == EntityKind::ParmDecl {
            if let Some(param) = extract_param(&child) {
                method.params.push(param);
            }
        }
    }

    // 构建签名
    let const_str = if method.is_const { " const" } else { "" };
    let params_str = method
        .params
        .iter()
        .map(|p| format!("{} {}", p.cpp_type, p.name))
        .collect::<Vec<_>>()
        .join(", ");
    method.cpp_signature = format!("{}({}){}",
        if method.is_static { format!("static {} {}", return_type, name) }
        else { format!("{} {}", return_type, name) },
        params_str,
        const_str
    );

    Some(method)
}

/// 提取全局函数
fn extract_function(entity: &Entity, namespace_stack: &[String]) -> Option<CppFunction> {
    let name = entity.get_name()?;

    // 跳过编译器内建函数
    if name.starts_with("__") {
        return None;
    }

    let return_type = if let Some(ty) = entity.get_result_type() {
        get_type_name(&ty)
    } else {
        "void".to_string()
    };

    let mut func = CppFunction::new(&name, &return_type);
    func.namespace = namespace_stack.to_vec();
    func.is_inline = entity.is_inline_function();

    // 参数
    for child in entity.get_children() {
        if child.get_kind() == EntityKind::ParmDecl {
            if let Some(param) = extract_param(&child) {
                func.params.push(param);
            }
        }
    }

    // 构建签名（C 风格：无参数时使用 void，有参数时只用类型不用名称）
    let params_str = if func.params.is_empty() {
        "void".to_string()
    } else {
        func.params
            .iter()
            .map(|p| normalize_cpp_type(&p.cpp_type))
            .collect::<Vec<_>>()
            .join(", ")
    };
    func.cpp_signature = format!("{} {}({})", normalize_cpp_type(&return_type), name, params_str);

    Some(func)
}

/// 规范化 C++ 类型名（去掉多余空格，指针靠近类型名）
fn normalize_cpp_type(ty: &str) -> String {
    let ty = ty.trim();
    // 规范化指针/引用：`char * ` → `char*`
    let ty = ty.replace(" * ", "*")
        .replace("* ", "*")
        .replace(" *", "*")
        .replace(" &", "&")
        .replace("& ", "&");
    // 修复 const pointer：`const char*` (keep leading const)
    ty
}
/// 提取函数/方法参数
fn extract_param(entity: &Entity) -> Option<CppParam> {
    let cpp_type = if let Some(ty) = entity.get_type() {
        get_type_name(&ty)
    } else {
        return None;
    };

    let name = entity.get_name().unwrap_or_else(|| {
        // 匿名参数生成名称
        "arg".to_string()
    });

    Some(CppParam::new(&name, &cpp_type))
}

/// 提取枚举
fn extract_enum(entity: &Entity, namespace_stack: &[String]) -> Option<CppEnum> {
    let name = entity.get_name()?;
    if name.is_empty() {
        return None;
    }

    if !entity.is_definition() {
        return None;
    }

    let mut enum_ = CppEnum::new(&name);
    enum_.namespace = namespace_stack.to_vec();
    enum_.is_scoped = entity.is_scoped();

    // 提取枚举值
    for child in entity.get_children() {
        if child.get_kind() == EntityKind::EnumConstantDecl {
            let val_name = child.get_name().unwrap_or_default();
            let val = child.get_enum_constant_value().unwrap_or((0, 0)).0;
            enum_.values.push((val_name, val));
        }
    }

    Some(enum_)
}

/// 提取常量
fn extract_const(entity: &Entity) -> Option<CppConst> {
    // 只提取全局 const 变量
    let name = entity.get_name()?;
    let ty = entity.get_type()?;

    // 只关注整型和浮点型常量
    match ty.get_kind() {
        TypeKind::Int | TypeKind::UInt | TypeKind::Long | TypeKind::ULong
        | TypeKind::LongLong | TypeKind::ULongLong | TypeKind::Float | TypeKind::Double => {}
        _ => return None,
    }

    let cpp_type = get_type_name(&ty);

    // 尝试获取值
    let value = if let Some(eval) = entity.evaluate() {
        match eval {
            EvaluationResult::SignedInteger(v) => v.to_string(),
            EvaluationResult::UnsignedInteger(v) => v.to_string(),
            EvaluationResult::Float(v) => format!("{}", v),
            EvaluationResult::String(s) => format!("\"{}\"", s.to_string_lossy()),
            _ => return None,
        }
    } else {
        return None;
    };

    Some(CppConst {
        name,
        cpp_type,
        value,
    })
}

/// 从 clang Type 获取 C++ 类型名称字符串
fn get_type_name(ty: &clang::Type) -> String {
    // 使用 clang 的 spelling 作为类型名
    let spelling = ty.get_display_name();

    // 规范化类型名
    normalize_type_name(&spelling)
}

/// 规范化 C++ 类型名（去掉多余空格等）
fn normalize_type_name(name: &str) -> String {
    let name = name.trim();
    // 处理 clang 的特殊表示
    let name = name.replace("_Bool", "bool");
    let name = name.replace("__int128", "long long");
    // 规范化指针周围的空格
    let mut result = String::new();
    let mut prev_space = false;
    for c in name.chars() {
        if c == ' ' {
            if !prev_space && !result.ends_with('*') {
                result.push(c);
                prev_space = true;
            }
        } else {
            result.push(c);
            prev_space = false;
        }
    }
    result.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    // libclang 只允许一个 Clang 实例同时存在，所以将多个解析测试合并为一个函数
    #[test]
    fn test_parse_cpp_files() {
        // --- 测试1：简单函数解析 ---
        let dir = tempdir().unwrap();
        let file = dir.path().join("test_func.cpp");
        fs::write(
            &file,
            r#"extern "C" void hello_world(void) {}"#,
        )
        .unwrap();

        let ast = parse_cpp_file(&file).unwrap();
        assert!(!ast.functions.is_empty());
        let func = &ast.functions[0];
        assert_eq!(func.name, "hello_world");

        // --- 测试2：类解析 ---
        let file2 = dir.path().join("test_class.cpp");
        fs::write(
            &file2,
            r#"class Counter {
                int value = 0;
            public:
                int get() const { return value; }
                void increment() { value++; }
            };"#,
        )
        .unwrap();

        let ast2 = parse_cpp_file(&file2).unwrap();
        assert!(!ast2.classes.is_empty());
        let class = &ast2.classes[0];
        assert_eq!(class.name, "Counter");
        assert!(!class.methods.is_empty());
    }
}
