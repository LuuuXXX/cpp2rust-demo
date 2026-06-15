//! C++ AST 解析 — Phase 2
//!
//! 使用 `clang` crate 解析 `g++ -E -C` 产出的 `.cpp2rust` 预处理文件。
//! 过滤系统头节点，只保留用户代码中的类、函数、枚举等声明。

mod collector;
pub mod range_scanner;

pub use range_scanner::{cpp_byte_ranges, user_content_byte_ranges, user_file_paths_from_content};

use anyhow::{anyhow, Result};
use clang::{Clang, EntityKind, Index, Language};
use std::path::{Path, PathBuf};

use crate::error::Cpp2RustError;
use collector::{
    collect_linkage_spec, collect_namespace, collect_typedef, extract_class, extract_function,
    extract_template_class, extract_template_function, is_noninline_fn_def,
};
use range_scanner::entity_is_from_current_file;

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
    /// 类的简单名（未做命名空间扁平化，如 `Counter`）。顶层类与 `name` 相同。
    /// 命名空间类的 `name` 仍保留旧式扁平化前缀（如 `class_basic_ns_Counter`）以兼容
    /// 旧路径的「被 extern-C 桥接引用的类」匹配逻辑；hicc 直出路径用本字段与 `namespace`
    /// 还原真实命名空间类名（`#[cpp(class = "ns::T")]`）。
    pub simple_name: String,
    /// 所属命名空间的 `::` 限定路径（如 `class_basic_ns` 或 `foo::bar`）。
    /// 顶层类为 `None`。用于 hicc 直出时绑定真实命名空间类（`#[cpp(class = "ns::T")]`）。
    pub namespace: Option<String>,
    /// 是否定义在当前被解析的 `.cpp2rust` 文件中（false 表示来自被 include 的头文件）
    pub is_from_current_file: bool,
}

impl ClassInfo {
    /// 返回 C++ 端的 `::` 限定类名（如 `class_basic_ns::Counter`）。
    /// 顶层类返回简单名本身。
    pub fn qualified_name(&self) -> String {
        match &self.namespace {
            Some(ns) if !ns.is_empty() => format!("{}::{}", ns, self.simple_name),
            _ => self.simple_name.clone(),
        }
    }
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

/// 模板类声明（`ClassTemplate`）— v6 Phase A
///
/// 仅收集来自当前编译单元的模板类，用于在生成器侧（v7 起默认）
/// 输出泛型 `import_class!` 骨架。与 `template_class_ranges` 互补：
/// 后者保存源码文本（供内联到 `hicc::cpp!`），本结构保存结构化签名信息。
#[derive(Debug, Clone)]
pub struct TemplateClassInfo {
    pub name: String,
    /// 类型参数名（如 `["T", "Allocator"]`）
    pub type_params: Vec<String>,
    pub bases: Vec<BaseInfo>,
    pub methods: Vec<MethodInfo>,
    pub fields: Vec<FieldInfo>,
    /// 是否定义在当前被解析的 `.cpp2rust` 文件中
    pub is_from_current_file: bool,
}

/// 模板函数声明（`FunctionTemplate`）— v6 Phase A
#[derive(Debug, Clone)]
pub struct TemplateFunctionInfo {
    pub name: String,
    /// 类型参数名（如 `["T"]`）
    pub type_params: Vec<String>,
    pub return_type: String,
    pub params: Vec<ParamInfo>,
    /// 是否定义在当前被解析的 `.cpp2rust` 文件中
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
    /// 模板类结构化信息（v6 Phase A）
    pub template_classes: Vec<TemplateClassInfo>,
    /// 模板函数结构化信息（v6 Phase A）
    pub template_functions: Vec<TemplateFunctionInfo>,
    /// 当前编译单元函数 / 方法体内**局部变量声明**的类型显示名（v6 Phase B 收尾）。
    ///
    /// 用于模板实例化追踪：覆盖 `Stack<int> s;`、`Stack<int>* p = new Stack<int>();`
    /// 等表达式级使用点。v7 起在生成器侧默认被消费。
    pub local_var_types: Vec<String>,
}

// ─────────────────────────────────────────────
//  解析入口
// ─────────────────────────────────────────────

/// 判断顶层实体是否应被跳过（两遍扫描共用的过滤谓词）。
///
/// 跳过条件：
/// 1. 实体不是"非内联函数定义"（非内联定义不可能来自系统头，必须保留）；
/// 2. 且 `is_in_system_header()` 为 true，或位置不可用时使用 `skip_if_no_location` 默认值。
///
/// `skip_if_no_location` 对 `FunctionDecl` 的规则：非 C 语言时为 `true`（跳过无位置的
/// 系统 C++ 声明）；其他 kind 默认为 `true`。
fn should_skip_entity(entity: &clang::Entity<'_>, kind: EntityKind) -> bool {
    if is_noninline_fn_def(entity) {
        return false;
    }
    let skip_if_no_location = match kind {
        EntityKind::FunctionDecl => entity.get_language() != Some(Language::C),
        _ => true,
    };
    entity
        .get_location()
        .map(|l| l.is_in_system_header())
        .unwrap_or(skip_if_no_location)
}

/// 解析 `.cpp2rust` 预处理文件，返回结构化 AST。
///
/// 输入文件由 `g++ -E -C` 生成，扩展名为非标准的 `.cpp2rust`，
/// 因此必须通过 `-xc++` 告知 libclang 以 C++ 模式解析。
pub fn parse_preprocessed(file: &Path) -> Result<CppAst> {
    let clang = Clang::new().map_err(|e| Cpp2RustError::LibclangInit(e.to_string()))?;
    let index = Index::new(&clang, false, false);

    let tu = index
        .parser(file)
        .arguments(&["-xc++", "-std=c++17"])
        .parse()
        .map_err(|e| Cpp2RustError::ParseFailed(format!("{}: {:?}", file.display(), e)))?;

    // 扫描预处理文件中的行号标记，确定哪些字节范围属于 shim cpp 文件自身
    // （而非 include 进来的头文件）。libclang 对预处理文件始终返回物理文件路径，
    // 所以必须通过字节偏移量来区分来源。
    let file_content = std::fs::read_to_string(file).map_err(|e| {
        anyhow!(
            "failed to read {} for line marker scan: {}",
            file.display(),
            e
        )
    })?;
    let cpp_ranges = cpp_byte_ranges(&file_content);
    let user_ranges = user_content_byte_ranges(&file_content);
    let user_files = user_file_paths_from_content(&file_content);

    let mut ast = CppAst {
        file: file.to_path_buf(),
        classes: Vec::new(),
        functions: Vec::new(),
        enums: Vec::new(),
        typedefs: Vec::new(),
        template_class_ranges: Vec::new(),
        template_classes: Vec::new(),
        template_functions: Vec::new(),
        local_var_types: Vec::new(),
    };

    let root = tu.get_entity();

    // 第一遍：收集类/函数/枚举声明
    for entity in root.get_children() {
        let kind = entity.get_kind();

        // LinkageSpec（extern "C"/"C++" 块）与 UnexposedDecl 需要特殊处理：
        // 在 Windows LLVM 17 上，用户代码的 extern "C" 块（包括用户头文件中的声明块
        // 和 .cpp 文件中的定义块）的 is_in_system_header() 有时会错误地返回 true，
        // 若依赖该标志跳过，整个块及其内部的所有函数将被丢弃。
        // 另外，在 Windows LLVM 17 + MSVC 头文件环境下，extern "C" 块有时以
        // EntityKind::UnexposedDecl 形式出现（而非 LinkageSpec），需要同等处理。
        // 解决方案：对两种 kind 都调用 collect_linkage_spec，不做顶层过滤；
        // 内部的精细过滤由 collect_linkage_spec 自行处理（非内联定义不跳过）。
        if kind == EntityKind::LinkageSpec || kind == EntityKind::UnexposedDecl {
            collect_linkage_spec(&entity, &mut ast, &cpp_ranges, &user_ranges, &user_files);
            continue;
        }

        if should_skip_entity(&entity, kind) {
            continue;
        }
        match kind {
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
                    // v6 Phase A：额外提取模板类的结构化签名信息（供生成器开关使用）
                    if let Some(tc) = extract_template_class(&entity, &cpp_ranges) {
                        ast.template_classes.push(tc);
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
            EntityKind::FunctionTemplate => {
                // v6 Phase A：提取模板函数签名（仅当前编译单元）
                if let Some(tf) = extract_template_function(&entity, &cpp_ranges) {
                    ast.template_functions.push(tf);
                }
            }
            EntityKind::EnumDecl if entity_is_from_current_file(&entity, &cpp_ranges) => {
                if let Some(ei) = collector::extract_enum(&entity) {
                    ast.enums.push(ei);
                }
            }
            EntityKind::Namespace => {
                collect_namespace(&entity, &mut ast, &cpp_ranges, &user_ranges);
            }
            EntityKind::TypedefDecl => {
                collect_typedef(&entity, &mut ast, &user_ranges);
            }
            _ => {}
        }
    }

    // 第二遍：收集类外方法定义（带方法体）并更新 body_offset
    for entity in root.get_children() {
        let kind = entity.get_kind();

        // 与第一遍相同的过滤逻辑（通过 should_skip_entity 共用）。
        // LinkageSpec/UnexposedDecl 在第二遍不需要处理（方法定义由 semantic_parent 定位），
        // 但仍需保持与第一遍一致的非内联函数定义豁免。
        if kind != EntityKind::LinkageSpec
            && kind != EntityKind::UnexposedDecl
            && should_skip_entity(&entity, kind)
        {
            continue;
        }
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
                    .map(|a| {
                        a.get_type()
                            .map(|t| t.get_display_name())
                            .unwrap_or_default()
                    })
                    .collect();
                let param_count = def_param_types.len();
                if let Some(parent) = entity.get_semantic_parent() {
                    if let Some(class_name) = parent.get_name() {
                        if let Some(class) = ast.classes.iter_mut().find(|c| c.name == class_name) {
                            // 先按名称+参数类型精确匹配，再按名称+参数数量匹配，最后仅按名称匹配
                            let idx = class
                                .methods
                                .iter()
                                .position(|m| {
                                    m.name == method_name
                                        && m.params.len() == param_count
                                        && m.params
                                            .iter()
                                            .zip(def_param_types.iter())
                                            .all(|(p, t)| p.type_name == *t)
                                })
                                .or_else(|| {
                                    class.methods.iter().position(|m| {
                                        m.name == method_name && m.params.len() == param_count
                                    })
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

    // 第三遍（v6 Phase B 收尾）：收集函数 / 方法体内局部变量声明的类型，供模板实例化
    // 追踪使用（如 `Stack<int> s;`、`Stack<int>* p = new Stack<int>();`）。
    // 仅遍历当前编译单元的子树（跳过系统头），限制遍历成本；结果 v7 起在生成器侧默认被消费。
    let mut local_var_types = Vec::new();
    collector::collect_local_var_types(&root, &cpp_ranges, &mut local_var_types);
    ast.local_var_types = local_var_types;

    Ok(ast)
}

// ─────────────────────────────────────────────
//  调试输出
// ─────────────────────────────────────────────

impl CppAst {
    /// 以树形文本格式打印 AST 内容（`parse` 子命令使用）。
    pub fn print_tree(&self) {
        println!("File: {}", self.file.display());

        for class in &self.classes {
            let kind = if class.is_struct {
                "StructDecl"
            } else {
                "ClassDecl"
            };
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
                    println!(
                        "    - ParmDecl: {} : {}{}",
                        param.name, param.type_name, def
                    );
                }
            }
            for field in &class.fields {
                let tags = field_tags(field);
                println!(
                    "  - FieldDecl: {} : {}{}",
                    field.name, field.type_name, tags
                );
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
    if m.is_const {
        tags.push("const");
    }
    if m.is_virtual {
        tags.push("virtual");
    }
    if m.is_pure_virtual {
        tags.push("pure_virtual");
    }
    if m.is_static {
        tags.push("static");
    }
    if m.is_inline {
        tags.push("inline");
    }
    if tags.is_empty() {
        String::new()
    } else {
        format!(" [{}]", tags.join(", "))
    }
}

fn field_tags(f: &FieldInfo) -> String {
    let mut tags = Vec::new();
    if f.is_mutable {
        tags.push("mutable");
    }
    if f.is_static {
        tags.push("static");
    }
    if tags.is_empty() {
        String::new()
    } else {
        format!(" [{}]", tags.join(", "))
    }
}

fn func_tags(f: &FunctionInfo) -> String {
    let mut tags = Vec::new();
    if f.is_extern_c {
        tags.push("extern_c");
    }
    if f.is_inline {
        tags.push("inline");
    }
    if f.is_variadic {
        tags.push("variadic");
    }
    if let Some(ref cls) = f.friend_of {
        tags.push(cls.as_str());
    }
    if tags.is_empty() {
        String::new()
    } else {
        format!(" [{}]", tags.join(", "))
    }
}
