//! C++ AST 解析 — Phase 1
//!
//! 使用 `clang` crate 解析 `g++ -E -C` 产出的 `.cpp2rust` 预处理文件。
//! 过滤系统头节点，只保留用户代码中的类、函数、枚举等声明。

use anyhow::{anyhow, Result};
use clang::{Clang, EntityKind, Index};
use std::path::{Path, PathBuf};

// ─────────────────────────────────────────────
//  数据结构
// ─────────────────────────────────────────────

/// 函数参数
#[derive(Debug, Clone)]
pub struct ParamInfo {
    pub name: String,
    /// 参数类型的显示字符串（来自 libclang）
    pub type_name: String,
    pub has_default: bool,
}

/// 类的成员字段
#[derive(Debug, Clone)]
pub struct FieldInfo {
    pub name: String,
    pub type_name: String,
    pub is_mutable: bool,
    pub is_static: bool,
    /// "public" | "protected" | "private"
    pub accessibility: String,
    /// 字段声明的字节范围（用于读取实际的默认值文本）
    pub field_offset: Option<(u32, u32)>,
}

/// 类的成员方法（含构造/析构）
#[derive(Debug, Clone)]
pub struct MethodInfo {
    pub name: String,
    pub return_type: String,
    pub params: Vec<ParamInfo>,
    pub is_const: bool,
    pub is_virtual: bool,
    pub is_pure_virtual: bool,
    pub is_static: bool,
    pub is_constructor: bool,
    pub is_destructor: bool,
    pub is_inline: bool,
    /// "public" | "protected" | "private"
    pub accessibility: String,
    /// 方法定义的字节范围（在 .cpp2rust 文件中）：(start, end)
    pub body_offset: Option<(u32, u32)>,
    /// 是否是 override（覆盖基类虚函数）
    pub is_override: bool,
    /// 是否是 `= default` 显式默认函数
    pub is_default: bool,
}

/// 基类说明符
#[derive(Debug, Clone)]
pub struct BaseInfo {
    pub name: String,
    pub is_virtual: bool,
}

/// 枚举变体
#[derive(Debug, Clone)]
pub struct EnumVariantInfo {
    pub name: String,
    pub value: i64,
}

/// C++ 枚举（含 enum class）
#[derive(Debug, Clone)]
pub struct EnumInfo {
    pub name: String,
    pub is_scoped: bool,
    pub underlying_type: String,
    pub variants: Vec<EnumVariantInfo>,
}

/// C++ 类或结构体
#[derive(Debug, Clone)]
pub struct ClassInfo {
    pub name: String,
    pub is_struct: bool,
    pub is_abstract: bool,
    /// 模板实参列表（仅 ClassTemplateSpecialization 有值）
    pub template_args: Vec<String>,
    pub bases: Vec<BaseInfo>,
    pub methods: Vec<MethodInfo>,
    pub fields: Vec<FieldInfo>,
}

/// 全局函数
#[derive(Debug, Clone)]
pub struct FunctionInfo {
    pub name: String,
    pub return_type: String,
    pub params: Vec<ParamInfo>,
    pub is_inline: bool,
    pub is_variadic: bool,
    /// `extern "C"` linkage
    pub is_extern_c: bool,
    /// 如果是友元函数提取出来的，记录其所属类名
    pub friend_of: Option<String>,
    /// 函数定义的字节范围（在 .cpp2rust 文件中）：(start, end)
    pub body_offset: Option<(u32, u32)>,
}

/// 顶层 AST 结果
#[derive(Debug)]
pub struct CppAst {
    /// 解析的源文件路径
    pub file: PathBuf,
    pub classes: Vec<ClassInfo>,
    pub functions: Vec<FunctionInfo>,
    pub enums: Vec<EnumInfo>,
}

// ─────────────────────────────────────────────
//  解析入口
// ─────────────────────────────────────────────

/// 解析 `.cpp2rust` 预处理文件，返回结构化 AST。
///
/// 输入文件由 `g++ -E -C` 生成，扩展名为非标准的 `.cpp2rust`，
/// 因此必须通过 `-xc++` 告知 libclang 以 C++ 模式解析。
pub fn parse_preprocessed(file: &Path) -> Result<CppAst> {
    let clang = Clang::new().map_err(|e| anyhow!("failed to init libclang: {}", e))?;
    let index = Index::new(&clang, false, false);

    let tu = index
        .parser(file)
        .arguments(&["-xc++", "-std=c++17"])
        .parse()
        .map_err(|e| anyhow!("parse error in {}: {:?}", file.display(), e))?;

    let mut ast = CppAst {
        file: file.to_path_buf(),
        classes: Vec::new(),
        functions: Vec::new(),
        enums: Vec::new(),
    };

    let root = tu.get_entity();

    // 第一遍：收集类/函数/枚举声明
    for entity in root.get_children() {
        if entity
            .get_location()
            .map(|l| l.is_in_system_header())
            .unwrap_or(true)
        {
            continue;
        }
        match entity.get_kind() {
            EntityKind::ClassDecl | EntityKind::StructDecl => {
                if let Some(ci) = extract_class(&entity) {
                    ast.classes.push(ci);
                }
            }
            EntityKind::ClassTemplate => {}
            EntityKind::ClassTemplatePartialSpecialization => {
                if let Some(ci) = extract_class(&entity) {
                    ast.classes.push(ci);
                }
            }
            EntityKind::FunctionDecl => {
                if let Some(fi) = extract_function(&entity, None) {
                    ast.functions.push(fi);
                }
            }
            EntityKind::FunctionTemplate => {}
            EntityKind::EnumDecl => {
                if let Some(ei) = extract_enum(&entity) {
                    ast.enums.push(ei);
                }
            }
            EntityKind::Namespace => {
                collect_namespace(&entity, &mut ast);
            }
            EntityKind::LinkageSpec => {
                collect_linkage_spec(&entity, &mut ast);
            }
            _ => {}
        }
    }

    // 第二遍：收集类外方法定义（带方法体）并更新 body_offset
    for entity in root.get_children() {
        if entity
            .get_location()
            .map(|l| l.is_in_system_header())
            .unwrap_or(true)
        {
            continue;
        }

        let kind = entity.get_kind();
        let is_method_def = matches!(
            kind,
            EntityKind::Method | EntityKind::Constructor | EntityKind::Destructor
        ) && entity.is_definition();

        if is_method_def {
            if let Some(range) = entity.get_range() {
                let start = range.get_start().get_file_location().offset;
                let end = range.get_end().get_file_location().offset;
                let method_name = entity.get_name().unwrap_or_default();
                let def_param_types: Vec<String> = entity
                    .get_arguments()
                    .unwrap_or_default()
                    .iter()
                    .map(|a| a.get_type().map(|t| t.get_display_name()).unwrap_or_default())
                    .collect();
                let param_count = def_param_types.len();
                if let Some(parent) = entity.get_semantic_parent() {
                    if let Some(class_name) = parent.get_name() {
                        if let Some(class) =
                            ast.classes.iter_mut().find(|c| c.name == class_name)
                        {
                            // 先按名称+参数类型精确匹配，再按名称+参数数量匹配，最后仅按名称匹配
                            let idx = class
                                .methods
                                .iter()
                                .position(|m| {
                                    m.name == method_name
                                        && m.params.len() == param_count
                                        && m.params.iter().zip(def_param_types.iter()).all(|(p, t)| p.type_name == *t)
                                })
                                .or_else(|| {
                                    class
                                        .methods
                                        .iter()
                                        .position(|m| m.name == method_name && m.params.len() == param_count)
                                })
                                .or_else(|| {
                                    class.methods.iter().position(|m| m.name == method_name)
                                });
                            if let Some(i) = idx {
                                class.methods[i].body_offset = Some((start, end));
                            }
                        }
                    }
                }
            }
        }

        // 更新类外函数定义的 body_offset
        if kind == EntityKind::FunctionDecl && entity.is_definition() {
            if let Some(range) = entity.get_range() {
                let start = range.get_start().get_file_location().offset;
                let end = range.get_end().get_file_location().offset;
                let fn_name = entity.get_name().unwrap_or_default();
                if let Some(func) = ast.functions.iter_mut().find(|f| f.name == fn_name) {
                    func.body_offset = Some((start, end));
                }
            }
        }
    }

    Ok(ast)
}

// ─────────────────────────────────────────────
//  内部辅助
// ─────────────────────────────────────────────

fn collect_namespace(ns: &clang::Entity<'_>, ast: &mut CppAst) {
    let ns_name = ns.get_name().unwrap_or_default();
    for entity in ns.get_children() {
        if entity
            .get_location()
            .map(|l| l.is_in_system_header())
            .unwrap_or(true)
        {
            continue;
        }
        match entity.get_kind() {
            EntityKind::ClassDecl | EntityKind::StructDecl => {
                if let Some(mut ci) = extract_class(&entity) {
                    // 命名空间前缀扁平化到类名
                    if !ns_name.is_empty() {
                        ci.name = format!("{}_{}", ns_name, ci.name);
                    }
                    ast.classes.push(ci);
                }
            }
            EntityKind::ClassTemplatePartialSpecialization => {
                if let Some(mut ci) = extract_class(&entity) {
                    if !ns_name.is_empty() {
                        ci.name = format!("{}_{}", ns_name, ci.name);
                    }
                    ast.classes.push(ci);
                }
            }
            EntityKind::FunctionDecl => {
                if let Some(fi) = extract_function(&entity, None) {
                    ast.functions.push(fi);
                }
            }
            EntityKind::EnumDecl => {
                if let Some(ei) = extract_enum(&entity) {
                    ast.enums.push(ei);
                }
            }
            EntityKind::Namespace => {
                collect_namespace(&entity, ast);
            }
            _ => {}
        }
    }
}

fn collect_linkage_spec(spec: &clang::Entity<'_>, ast: &mut CppAst) {
    for entity in spec.get_children() {
        if entity
            .get_location()
            .map(|l| l.is_in_system_header())
            .unwrap_or(true)
        {
            continue;
        }
        if entity.get_kind() == EntityKind::FunctionDecl {
            if let Some(mut fi) = extract_function(&entity, None) {
                fi.is_extern_c = true;
                ast.functions.push(fi);
            }
        }
    }
}

fn extract_class(entity: &clang::Entity<'_>) -> Option<ClassInfo> {
    let name = entity.get_name()?;
    let is_struct = entity.get_kind() == EntityKind::StructDecl;
    let is_abstract = entity.is_abstract_record();

    let mut template_args = Vec::new();
    if let Some(args) = entity.get_template_arguments() {
        for arg in &args {
            template_args.push(format!("{:?}", arg));
        }
    }

    let mut bases = Vec::new();
    let mut methods = Vec::new();
    let mut fields = Vec::new();

    for child in entity.get_children() {
        match child.get_kind() {
            EntityKind::BaseSpecifier => {
                let base_name = child
                    .get_type()
                    .and_then(|t| Some(t.get_display_name()))
                    .unwrap_or_default();
                bases.push(BaseInfo {
                    name: base_name,
                    is_virtual: child.is_virtual_base(),
                });
            }
            EntityKind::Method => {
                if let Some(mi) = extract_method(&child) {
                    methods.push(mi);
                }
            }
            EntityKind::Constructor => {
                if let Some(mi) = extract_method(&child) {
                    methods.push(mi);
                }
            }
            EntityKind::Destructor => {
                if let Some(mi) = extract_method(&child) {
                    methods.push(mi);
                }
            }
            EntityKind::FieldDecl => {
                let field_name = child.get_name().unwrap_or_default();
                let type_name = child
                    .get_type()
                    .map(|t| t.get_display_name())
                    .unwrap_or_default();
                let accessibility = access_str(child.get_accessibility());
                // 判断字段是否有内联默认值（如 int value = 0）
                let field_offset = child.get_range().map(|r| {
                    let start = r.get_start().get_file_location().offset;
                    let end = r.get_end().get_file_location().offset;
                    (start, end)
                });
                fields.push(FieldInfo {
                    name: field_name,
                    type_name,
                    is_mutable: child.is_mutable(),
                    is_static: false,
                    accessibility,
                    field_offset,
                });
            }
            EntityKind::VarDecl => {
                let field_name = child.get_name().unwrap_or_default();
                let type_name = child
                    .get_type()
                    .map(|t| t.get_display_name())
                    .unwrap_or_default();
                let accessibility = access_str(child.get_accessibility());
                let field_offset = child.get_range().map(|r| {
                    let start = r.get_start().get_file_location().offset;
                    let end = r.get_end().get_file_location().offset;
                    (start, end)
                });
                fields.push(FieldInfo {
                    name: field_name,
                    type_name,
                    is_mutable: false,
                    is_static: true,
                    accessibility,
                    field_offset,
                });
            }
            EntityKind::FunctionDecl => {
                // 友元函数声明 — 不放入 methods
            }
            _ => {}
        }
    }

    Some(ClassInfo {
        name,
        is_struct,
        is_abstract,
        template_args,
        bases,
        methods,
        fields,
    })
}

fn extract_method(entity: &clang::Entity<'_>) -> Option<MethodInfo> {
    let name = entity.get_name().unwrap_or_default();
    let return_type = entity
        .get_result_type()
        .map(|t| t.get_display_name())
        .unwrap_or_default();

    let params = extract_params(entity);
    let is_constructor = entity.get_kind() == EntityKind::Constructor;
    let is_destructor = entity.get_kind() == EntityKind::Destructor;
    let accessibility = access_str(entity.get_accessibility());

    // 内联方法定义：直接含方法体
    let body_offset = if entity.is_definition() {
        entity.get_range().map(|r| {
            let start = r.get_start().get_file_location().offset;
            let end = r.get_end().get_file_location().offset;
            (start, end)
        })
    } else {
        None
    };

    Some(MethodInfo {
        name,
        return_type,
        params,
        is_const: entity.is_const_method(),
        is_virtual: entity.is_virtual_method(),
        is_pure_virtual: entity.is_pure_virtual_method(),
        is_static: entity.is_static_method(),
        is_constructor,
        is_destructor,
        is_inline: entity.is_inline_function(),
        accessibility,
        body_offset,
        is_override: !entity.get_overridden_methods().unwrap_or_default().is_empty(),
        is_default: entity.is_defaulted(),
    })
}

fn extract_function(entity: &clang::Entity<'_>, friend_of: Option<&str>) -> Option<FunctionInfo> {
    let name = entity.get_name()?;
    let return_type = entity
        .get_result_type()
        .map(|t| t.get_display_name())
        .unwrap_or_default();

    let params = extract_params(entity);

    let is_variadic = entity
        .get_type()
        .map(|t| t.is_variadic())
        .unwrap_or(false);

    let body_offset = if entity.is_definition() {
        entity.get_range().map(|r| {
            let start = r.get_start().get_file_location().offset;
            let end = r.get_end().get_file_location().offset;
            (start, end)
        })
    } else {
        None
    };

    Some(FunctionInfo {
        name,
        return_type,
        params,
        is_inline: entity.is_inline_function(),
        is_variadic,
        is_extern_c: false,
        friend_of: friend_of.map(String::from),
        body_offset,
    })
}

fn extract_params(entity: &clang::Entity<'_>) -> Vec<ParamInfo> {
    entity
        .get_arguments()
        .unwrap_or_default()
        .iter()
        .map(|arg| {
            let name = arg.get_name().unwrap_or_else(|| "_".to_string());
            let type_name = arg
                .get_type()
                .map(|t| t.get_display_name())
                .unwrap_or_default();
            // libclang 没有直接暴露"是否有默认值"的 API；
            // 通过检查子节点是否有表达式来间接判断
            let has_default = arg
                .get_children()
                .iter()
                .any(|c| is_expression_kind(c.get_kind()));
            ParamInfo {
                name,
                type_name,
                has_default,
            }
        })
        .collect()
}

fn extract_enum(entity: &clang::Entity<'_>) -> Option<EnumInfo> {
    let name = entity.get_name()?;
    let underlying_type = entity
        .get_enum_underlying_type()
        .map(|t| t.get_display_name())
        .unwrap_or_else(|| "int".to_string());

    // enum class（scoped enum）通过检查子节点中不含有同名符号来判断
    // libclang 2.x 没有直接的 is_scoped_enum API；用 display_name 含 "::" 来近似
    let is_scoped = entity
        .get_children()
        .first()
        .and_then(|c| c.get_name())
        .map(|cn| {
            entity
                .get_name()
                .map(|en| cn.starts_with(&en))
                .unwrap_or(false)
        })
        .unwrap_or(false);

    let variants = entity
        .get_children()
        .iter()
        .filter(|c| c.get_kind() == EntityKind::EnumConstantDecl)
        .filter_map(|c| {
            let vname = c.get_name()?;
            let value = c.get_enum_constant_value().map(|(v, _)| v).unwrap_or(0);
            Some(EnumVariantInfo { name: vname, value })
        })
        .collect();

    Some(EnumInfo {
        name,
        is_scoped,
        underlying_type,
        variants,
    })
}

fn is_expression_kind(kind: EntityKind) -> bool {
    matches!(
        kind,
        EntityKind::IntegerLiteral
            | EntityKind::FloatingLiteral
            | EntityKind::StringLiteral
            | EntityKind::CharacterLiteral
            | EntityKind::BoolLiteralExpr
            | EntityKind::NullPtrLiteralExpr
            | EntityKind::UnaryOperator
            | EntityKind::BinaryOperator
            | EntityKind::CallExpr
            | EntityKind::DeclRefExpr
            | EntityKind::UnexposedExpr
            | EntityKind::CStyleCastExpr
    )
}

fn access_str(acc: Option<clang::Accessibility>) -> String {
    match acc {
        Some(clang::Accessibility::Public) => "public".to_string(),
        Some(clang::Accessibility::Protected) => "protected".to_string(),
        Some(clang::Accessibility::Private) | None => "private".to_string(),
    }
}

// ─────────────────────────────────────────────
//  调试输出
// ─────────────────────────────────────────────

impl CppAst {
    /// 以树形文本格式打印 AST 内容（`parse` 子命令使用）。
    pub fn print_tree(&self) {
        println!("File: {}", self.file.display());

        for class in &self.classes {
            let kind = if class.is_struct { "StructDecl" } else { "ClassDecl" };
            let abstract_tag = if class.is_abstract { " [abstract]" } else { "" };
            println!("- {}: {}{}", kind, class.name, abstract_tag);
            for base in &class.bases {
                let virt = if base.is_virtual { " [virtual]" } else { "" };
                println!("  - BaseSpecifier: {}{}", base.name, virt);
            }
            for method in &class.methods {
                let tags = method_tags(method);
                println!("  - {}: {}{}", method_kind_str(method), method.name, tags);
                for param in &method.params {
                    let def = if param.has_default { " [default]" } else { "" };
                    println!("    - ParmDecl: {} : {}{}", param.name, param.type_name, def);
                }
            }
            for field in &class.fields {
                let tags = field_tags(field);
                println!("  - FieldDecl: {} : {}{}", field.name, field.type_name, tags);
            }
        }

        for func in &self.functions {
            let tags = func_tags(func);
            println!("- FunctionDecl: {}{}", func.name, tags);
            for param in &func.params {
                let def = if param.has_default { " [default]" } else { "" };
                println!("  - ParmDecl: {} : {}{}", param.name, param.type_name, def);
            }
        }

        for en in &self.enums {
            let scoped = if en.is_scoped { " [scoped]" } else { "" };
            println!("- EnumDecl: {}{}", en.name, scoped);
            for v in &en.variants {
                println!("  - EnumConstantDecl: {} = {}", v.name, v.value);
            }
        }
    }
}

fn method_kind_str(m: &MethodInfo) -> &'static str {
    if m.is_constructor {
        "CXXConstructorDecl"
    } else if m.is_destructor {
        "CXXDestructorDecl"
    } else {
        "CXXMethodDecl"
    }
}

fn method_tags(m: &MethodInfo) -> String {
    let mut tags = Vec::new();
    if m.is_const { tags.push("const"); }
    if m.is_virtual { tags.push("virtual"); }
    if m.is_pure_virtual { tags.push("pure_virtual"); }
    if m.is_static { tags.push("static"); }
    if m.is_inline { tags.push("inline"); }
    if tags.is_empty() {
        String::new()
    } else {
        format!(" [{}]", tags.join(", "))
    }
}

fn field_tags(f: &FieldInfo) -> String {
    let mut tags = Vec::new();
    if f.is_mutable { tags.push("mutable"); }
    if f.is_static { tags.push("static"); }
    if tags.is_empty() {
        String::new()
    } else {
        format!(" [{}]", tags.join(", "))
    }
}

fn func_tags(f: &FunctionInfo) -> String {
    let mut tags = Vec::new();
    if f.is_extern_c { tags.push("extern_c"); }
    if f.is_inline { tags.push("inline"); }
    if f.is_variadic { tags.push("variadic"); }
    if let Some(ref cls) = f.friend_of {
        tags.push(cls.as_str());
    }
    if tags.is_empty() {
        String::new()
    } else {
        format!(" [{}]", tags.join(", "))
    }
}
