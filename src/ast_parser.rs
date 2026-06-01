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
    pub is_volatile: bool,
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
    /// 是否来自命名空间（collect_namespace 收集的类）
    pub is_in_namespace: bool,
    /// 是否定义在当前被解析的 `.cpp2rust` 文件中（false 表示来自被 include 的头文件）
    pub is_from_current_file: bool,
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
    /// 函数声明/定义是否位于当前编译单元（.cpp 文件自身），而非被 include 的头文件。
    /// 与 `ClassInfo::is_from_current_file` 语义一致，通过 `cpp_byte_ranges` 判断。
    pub is_from_current_file: bool,
}

/// 顶层 AST 结果
#[derive(Debug)]
pub struct CppAst {
    /// 解析的源文件路径
    pub file: PathBuf,
    pub classes: Vec<ClassInfo>,
    pub functions: Vec<FunctionInfo>,
    pub enums: Vec<EnumInfo>,
    /// typedef 声明列表：(名称, 起始偏移, 结束偏移)
    pub typedefs: Vec<(String, u32, u32)>,
    /// 模板类源码范围列表：(名称, 起始偏移, 结束偏移)
    pub template_class_ranges: Vec<(String, u32, u32)>,
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

    // 扫描预处理文件中的行号标记，确定哪些字节范围属于 shim cpp 文件自身
    // （而非 include 进来的头文件）。libclang 对预处理文件始终返回物理文件路径，
    // 所以必须通过字节偏移量来区分来源。
    let file_content = std::fs::read_to_string(file)
        .map_err(|e| anyhow!("failed to read {} for line marker scan: {}", file.display(), e))?;
    let cpp_ranges = cpp_byte_ranges(&file_content);

    let mut ast = CppAst {
        file: file.to_path_buf(),
        classes: Vec::new(),
        functions: Vec::new(),
        enums: Vec::new(),
        typedefs: Vec::new(),
        template_class_ranges: Vec::new(),
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
                if let Some(ci) = extract_class(&entity, &cpp_ranges) {
                    ast.classes.push(ci);
                }
            }
            EntityKind::ClassTemplate => {
                // 仅收集来自当前编译单元的模板类（用字节范围排除 include 进来的头文件）
                let from_current = entity
                    .get_range()
                    .map(|r| {
                        let offset = r.get_start().get_file_location().offset;
                        cpp_ranges.iter().any(|range| range.contains(&offset))
                    })
                    .unwrap_or(false);
                if from_current {
                    if let Some(range) = entity.get_range() {
                        let start = range.get_start().get_file_location().offset;
                        let end = range.get_end().get_file_location().offset;
                        let name = entity.get_name().unwrap_or_default();
                        ast.template_class_ranges.push((name, start, end));
                    }
                }
            }
            EntityKind::ClassTemplatePartialSpecialization => {
                if let Some(ci) = extract_class(&entity, &cpp_ranges) {
                    ast.classes.push(ci);
                }
            }
            EntityKind::FunctionDecl => {
                if let Some(fi) = extract_function(&entity, None, &cpp_ranges) {
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
                collect_namespace(&entity, &mut ast, &cpp_ranges);
            }
            EntityKind::LinkageSpec => {
                collect_linkage_spec(&entity, &mut ast, &cpp_ranges);
            }
            EntityKind::TypedefDecl => {
                collect_typedef(&entity, &mut ast);
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

fn collect_namespace(
    ns: &clang::Entity<'_>,
    ast: &mut CppAst,
    cpp_ranges: &[std::ops::Range<u32>],
) {
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
                if let Some(mut ci) = extract_class(&entity, cpp_ranges) {
                    // 命名空间前缀扁平化到类名
                    if !ns_name.is_empty() {
                        ci.name = format!("{}_{}", ns_name, ci.name);
                    }
                    ci.is_in_namespace = true;
                    ast.classes.push(ci);
                }
            }
            EntityKind::ClassTemplatePartialSpecialization => {
                if let Some(mut ci) = extract_class(&entity, cpp_ranges) {
                    if !ns_name.is_empty() {
                        ci.name = format!("{}_{}", ns_name, ci.name);
                    }
                    ci.is_in_namespace = true;
                    ast.classes.push(ci);
                }
            }
            EntityKind::FunctionDecl => {
                if let Some(fi) = extract_function(&entity, None, cpp_ranges) {
                    ast.functions.push(fi);
                }
            }
            EntityKind::EnumDecl => {
                if let Some(ei) = extract_enum(&entity) {
                    ast.enums.push(ei);
                }
            }
            EntityKind::Namespace => {
                collect_namespace(&entity, ast, cpp_ranges);
            }
            _ => {}
        }
    }
}

fn collect_linkage_spec(
    spec: &clang::Entity<'_>,
    ast: &mut CppAst,
    cpp_ranges: &[std::ops::Range<u32>],
) {
    for entity in spec.get_children() {
        if entity
            .get_location()
            .map(|l| l.is_in_system_header())
            .unwrap_or(true)
        {
            continue;
        }
        match entity.get_kind() {
            EntityKind::FunctionDecl => {
                if let Some(mut fi) = extract_function(&entity, None, cpp_ranges) {
                    fi.is_extern_c = true;
                    ast.functions.push(fi);
                }
            }
            EntityKind::EnumDecl => {
                if let Some(ei) = extract_enum(&entity) {
                    ast.enums.push(ei);
                }
            }
            EntityKind::TypedefDecl => {
                collect_typedef(&entity, ast);
            }
            // 仅收集有完整定义的 struct（跳过 `struct Foo;` 前向声明）
            EntityKind::StructDecl if entity.is_definition() => {
                if let Some(mut ci) = extract_class(&entity, cpp_ranges) {
                    ci.is_in_namespace = false;
                    ast.classes.push(ci);
                }
            }
            _ => {}
        }
    }
}

fn collect_typedef(entity: &clang::Entity<'_>, ast: &mut CppAst) {
    let Some(name) = entity.get_name() else { return };
    let Some(range) = entity.get_range() else { return };
    let start = range.get_start().get_file_location().offset;
    let end = range.get_end().get_file_location().offset;
    ast.typedefs.push((name, start, end));
}

fn extract_class(
    entity: &clang::Entity<'_>,
    cpp_ranges: &[std::ops::Range<u32>],
) -> Option<ClassInfo> {
    let name = entity.get_name()?;
    let is_struct = entity.get_kind() == EntityKind::StructDecl;
    let is_abstract = entity.is_abstract_record();

    // 判断该类是否定义在当前编译单元（shim cpp 文件）中，而非被 include 的头文件。
    //
    // libclang 解析预处理文件（`.cpp2rust`）时，所有实体的 `get_location()` 都返回
    // 物理文件（`.cpp2rust`）的字节偏移量，而非跟随行号标记的逻辑来源文件。
    // 因此不能用文件路径比较，而必须用字节偏移量与 `cpp_byte_ranges` 扫描结果对比：
    // 只有落在 shim cpp 内容区间（即 `.cpp` 行号标记之后、`.h` 标记之前的区域）
    // 的实体才认为来自当前文件。
    let is_from_current_file = entity_is_from_current_file(entity, cpp_ranges);

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
                    .map(|t| t.get_display_name())
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
        is_in_namespace: false,
        is_from_current_file,
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
        is_volatile: entity
            .get_type()
            .map(|t| {
                let display_name = t.get_display_name();
                // 方法类型显示名如 "volatile uint32_t () volatile"
                // 尾部 " volatile" 表示 this-volatile 修饰符（影响方法指针类型）
                display_name.trim_end().ends_with(") volatile")
                    || display_name.trim_end().ends_with(") volatile &")
                    || display_name.trim_end().ends_with(") volatile &&")
            })
            .unwrap_or(false),
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

fn extract_function(
    entity: &clang::Entity<'_>,
    friend_of: Option<&str>,
    cpp_ranges: &[std::ops::Range<u32>],
) -> Option<FunctionInfo> {
    let name = entity.get_name()?;
    // 跳过所有 C++ 操作符命名的自由函数（含 UDL operator""h 等），
    // 这类函数在 C 链接层无法直接表示，且生成的 Rust 名称不合法。
    if name.starts_with("operator") {
        return None;
    }
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

    // 判断函数声明/定义是否来自当前 .cpp 文件（而非被 include 的头文件）。
    let is_from_current_file = entity_is_from_current_file(entity, cpp_ranges);

    Some(FunctionInfo {
        name,
        return_type,
        params,
        is_inline: entity.is_inline_function(),
        is_variadic,
        is_extern_c: false,
        friend_of: friend_of.map(String::from),
        body_offset,
        is_from_current_file,
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

// ─────────────────────────────────────────────
//  行号标记扫描（用于 is_from_current_file 判断）
// ─────────────────────────────────────────────

/// 判断 clang 实体的起始偏移量是否落在 `cpp_ranges` 范围内，
/// 即实体是否来自当前 `.cpp` 文件本身（而非 `#include` 引入的头文件）。
fn entity_is_from_current_file(
    entity: &clang::Entity<'_>,
    cpp_ranges: &[std::ops::Range<u32>],
) -> bool {
    entity
        .get_range()
        .map(|r| {
            let offset = r.get_start().get_file_location().offset;
            cpp_ranges.iter().any(|range| range.contains(&offset))
        })
        .unwrap_or(false)
}

/// 扫描 `g++ -E` 生成的预处理文件内容，返回属于 `.cpp`/`.c` 文件（而非 `.h`/`.hpp` 头文件）
/// 内容的字节偏移量区间列表。
///
/// 原理：预处理文件中包含行号标记（linemarker），格式为
/// `# <行号> "<文件路径>" [标志]`，通过解析这些标记即可知道每段内容来自哪个原始文件。
/// 后缀为 `.h`/`.hpp` 的标记表示进入了头文件，后缀为 `.cpp`/`.c` 的标记表示回到了
/// 主 shim 文件；系统虚拟路径（`<built-in>`、`<command-line>` 等）则跳过。
pub fn cpp_byte_ranges(content: &str) -> Vec<std::ops::Range<u32>> {
    let mut ranges: Vec<std::ops::Range<u32>> = Vec::new();
    let mut in_cpp = false;
    let mut section_start: u32 = 0;
    let mut byte_pos: u32 = 0;

    for line in content.split('\n') {
        let line_byte_len = line.len() as u32 + 1; // +1 表示 '\n'

        let trimmed = line.trim_start();
        if trimmed.starts_with("# ") {
            if let Some(file_path) = parse_line_marker_file(trimmed) {
                // 过滤系统虚拟路径（<built-in>、<command-line> 等）
                let is_virtual = file_path.starts_with('<') || file_path.is_empty();
                // 头文件后缀
                let is_header = file_path.ends_with(".h")
                    || file_path.ends_with(".hpp")
                    || file_path.ends_with(".hh");
                // .cpp/.c 文件（即 shim cpp 自身）
                let is_cpp = !is_virtual && !is_header;

                match (in_cpp, is_cpp) {
                    (true, false) => {
                        // 离开 cpp 区间
                        ranges.push(section_start..byte_pos);
                        in_cpp = false;
                    }
                    (false, true) => {
                        // 进入 cpp 区间（行号标记行本身不算内容，从下一行开始）
                        in_cpp = true;
                        section_start = byte_pos + line_byte_len;
                    }
                    _ => {}
                }
            }
        }

        byte_pos += line_byte_len;
    }

    if in_cpp && section_start < byte_pos {
        ranges.push(section_start..byte_pos);
    }

    ranges
}

/// 从行号标记中提取文件路径。
/// 格式：`# <数字> "<路径>" [标志]`，返回引号中的路径部分。
fn parse_line_marker_file(line: &str) -> Option<&str> {
    // 跳过 "# " 前缀
    let rest = line[2..].trim_start();
    // 跳过数字
    let after_num = rest
        .trim_start_matches(|c: char| c.is_ascii_digit())
        .trim_start();
    // 必须以 '"' 开头
    if !after_num.starts_with('"') {
        return None;
    }
    let inner = &after_num[1..];
    let end = inner.find('"')?;
    Some(&inner[..end])
}
