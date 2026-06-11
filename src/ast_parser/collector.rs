//! 类/函数/枚举收集逻辑 — 从 libclang 实体树提取结构化信息
//!
//! 供 `mod.rs` 中的 `parse_preprocessed` 调用。

use clang::{EntityKind, Language, TemplateArgument};

use super::range_scanner::{entity_is_from_current_file, entity_presumed_from_user_file};
use super::{
    BaseInfo, ClassInfo, CppAst, EnumInfo, EnumVariantInfo, FieldInfo, FunctionInfo, MethodInfo,
    ParamInfo, TemplateClassInfo, TemplateFunctionInfo,
};

/// 递归收集当前编译单元函数 / 方法体内**局部变量声明**的类型显示名（v6 Phase B 收尾）。
///
/// 用于模板实例化追踪：覆盖 `Stack<int> s;`、`Stack<int>* p = new Stack<int>();`
/// 等表达式级使用点（`auto p = new Stack<int>();` 会被 libclang 推导为
/// `Stack<int> *`，同样可被捕获）。
///
/// 遍历时跳过位于系统头的子树以限制成本；仅记录落在当前编译单元字节范围
/// （`cpp_ranges`）内的 `VarDecl`。函数参数为 `ParmDecl`、类字段为 `FieldDecl`，
/// 均不属于 `VarDecl`，因此不会在此重复收集（静态成员为 `VarDecl`，与字段来源
/// 重叠的部分由提取器去重）。
pub(super) fn collect_local_var_types(
    entity: &clang::Entity<'_>,
    cpp_ranges: &[std::ops::Range<u32>],
    out: &mut Vec<String>,
) {
    for child in entity.get_children() {
        // 跳过系统头中的子树，避免遍历庞大的标准库实体树
        if child
            .get_location()
            .map(|l| l.is_in_system_header())
            .unwrap_or(false)
        {
            continue;
        }
        if child.get_kind() == EntityKind::VarDecl
            && entity_is_from_current_file(&child, cpp_ranges)
        {
            if let Some(t) = child.get_type() {
                let name = t.get_display_name();
                if !name.is_empty() {
                    out.push(name);
                }
            }
        }
        collect_local_var_types(&child, cpp_ranges, out);
    }
}

pub(super) fn collect_namespace(
    ns: &clang::Entity<'_>,
    ast: &mut CppAst,
    cpp_ranges: &[std::ops::Range<u32>],
    user_ranges: &[std::ops::Range<u32>],
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
                    if fi.is_from_current_file {
                        ast.functions.push(fi);
                    }
                }
            }
            EntityKind::EnumDecl if entity_is_from_current_file(&entity, cpp_ranges) => {
                if let Some(ei) = extract_enum(&entity) {
                    ast.enums.push(ei);
                }
            }
            EntityKind::TypedefDecl => {
                collect_typedef(&entity, ast, user_ranges);
            }
            EntityKind::Namespace => {
                collect_namespace(&entity, ast, cpp_ranges, user_ranges);
            }
            _ => {}
        }
    }
}

/// 判断实体是否为"非内联函数**定义**"（不限语言）。
///
/// 系统头文件中不可能存在非内联的函数定义（只有声明和内联定义），
/// 因此对满足该条件的实体，应绕过 `is_in_system_header()` 检查，
/// 避免 Windows LLVM 17 上 `is_in_system_header()` 对用户代码误报的问题。
///
/// 不限语言：在 `.cpp` 文件中，`extern "C" {}` 块内或顶层的函数定义，
/// libclang 的 `get_language()` 可能返回 `CPlusPlus` 而非 `C`，
/// 因此不能依赖语言检查。
pub(super) fn is_noninline_fn_def(entity: &clang::Entity<'_>) -> bool {
    entity.get_kind() == EntityKind::FunctionDecl
        && entity.is_definition()
        && !entity.is_inline_function()
}

pub(super) fn collect_linkage_spec(
    spec: &clang::Entity<'_>,
    ast: &mut CppAst,
    cpp_ranges: &[std::ops::Range<u32>],
    user_ranges: &[std::ops::Range<u32>],
    user_files: &std::collections::HashSet<String>,
) {
    for entity in spec.get_children() {
        // 过滤：跳过来自系统头的实体，但有两个关键例外：
        //
        // 例外 1：非内联函数**定义**不可能真正来自系统头文件，直接保留。
        //
        // 例外 2：用双重检查判断是否为用户代码：
        //   a) 字节范围检查（user_ranges）：基于 linemarker flag-3 扫描
        //   b) 路径检查（user_files + get_presumed_location()）：通过原始源文件路径判断
        // 任一为 true 即视为用户代码。这样即使一种方式因平台差异失效，另一种仍可救援。
        let from_user_range = entity
            .get_range()
            .map(|r| {
                let offset = r.get_start().get_file_location().offset;
                user_ranges.iter().any(|range| range.contains(&offset))
            })
            .unwrap_or(true);
        let from_user_file = entity_presumed_from_user_file(&entity, user_files);
        if !is_noninline_fn_def(&entity) && !from_user_range && !from_user_file {
            continue;
        }
        match entity.get_kind() {
            EntityKind::FunctionDecl => {
                if let Some(mut fi) = extract_function(&entity, None, cpp_ranges) {
                    fi.is_extern_c = true;
                    ast.functions.push(fi);
                }
            }
            EntityKind::EnumDecl if entity_is_from_current_file(&entity, cpp_ranges) => {
                if let Some(ei) = extract_enum(&entity) {
                    ast.enums.push(ei);
                }
            }
            EntityKind::TypedefDecl => {
                collect_typedef(&entity, ast, user_ranges);
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

pub(super) fn collect_typedef(
    entity: &clang::Entity<'_>,
    ast: &mut CppAst,
    user_ranges: &[std::ops::Range<u32>],
) {
    // 收集来自用户代码区间（.cpp 文件本身及用户头文件）的 typedef，
    // 排除通过 #include 引入的系统/第三方头文件中的 typedef（避免系统类型污染）。
    if !entity_is_from_current_file(entity, user_ranges) {
        return;
    }
    let Some(name) = entity.get_name() else {
        return;
    };
    let Some(range) = entity.get_range() else {
        return;
    };
    let start = range.get_start().get_file_location().offset;
    let end = range.get_end().get_file_location().offset;
    ast.typedefs.push((name, start, end));
}

/// 将 libclang 的模板实参（[`TemplateArgument`]）转为干净的类型显示名。
///
/// 主要用于显式实例化（如 `template class Matrix<int>;`，在 AST 中表现为带模板实参的
/// `ClassDecl`）的实参提取。`TemplateArgument::Type(t)` 取 `t.get_display_name()`
/// 得到 `int` / `double` 等可直接复用的类型名。
///
/// 非类型实参（如整型常量 `template<int N>` 的 `N`）暂回退到 libclang 的调试表示。
/// **该回退字符串并非合法 C++ 类型/表达式**；下游实例化别名提取（`template_spec`）
/// 主要面向类型实参，对该回退串会按「类类型」处理并附 `cpp2rust-todo[TMPL]` 提示用户
/// 补全（且仅在 `CPP2RUST_GEN_TEMPLATES` 开关开启时输出）。完整的非类型模板参数支持
/// 留待后续阶段扩展。
fn template_arg_display(arg: &TemplateArgument<'_>) -> String {
    match arg {
        TemplateArgument::Type(t) => t.get_display_name(),
        other => format!("{:?}", other),
    }
}

pub(super) fn extract_class(
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
            template_args.push(template_arg_display(arg));
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
            EntityKind::Method | EntityKind::Constructor | EntityKind::Destructor => {
                if let Some(mi) = extract_method(&child) {
                    methods.push(mi);
                }
            }
            EntityKind::FieldDecl | EntityKind::VarDecl => {
                let is_static = child.get_kind() == EntityKind::VarDecl;
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
                    is_mutable: !is_static && child.is_mutable(),
                    is_static,
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

/// 收集模板实体（`ClassTemplate` / `FunctionTemplate`）的类型参数名。
///
/// 遍历直接子节点，提取 `TemplateTypeParameter` / `NonTypeTemplateParameter` /
/// `TemplateTemplateParameter` 的名称（如 `T`、`Allocator`、`N`）。
fn collect_template_params(entity: &clang::Entity<'_>) -> Vec<String> {
    entity
        .get_children()
        .iter()
        .filter(|c| {
            matches!(
                c.get_kind(),
                EntityKind::TemplateTypeParameter
                    | EntityKind::NonTypeTemplateParameter
                    | EntityKind::TemplateTemplateParameter
            )
        })
        .filter_map(|c| c.get_name())
        .filter(|n| !n.is_empty())
        .collect()
}

/// 提取模板类（`ClassTemplate`）的结构化信息 — v6 Phase A。
///
/// 复用 `extract_method` 收集成员方法，并通过 [`collect_template_params`] 获取
/// 泛型参数名。仅用于生成器侧的泛型骨架输出（受 `CPP2RUST_GEN_TEMPLATES` 开关控制），
/// 不影响默认产物。
pub(super) fn extract_template_class(
    entity: &clang::Entity<'_>,
    cpp_ranges: &[std::ops::Range<u32>],
) -> Option<TemplateClassInfo> {
    let name = entity.get_name()?;
    let is_from_current_file = entity_is_from_current_file(entity, cpp_ranges);
    let type_params = collect_template_params(entity);

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
            EntityKind::Method | EntityKind::Constructor | EntityKind::Destructor => {
                if let Some(mi) = extract_method(&child) {
                    methods.push(mi);
                }
            }
            EntityKind::FieldDecl | EntityKind::VarDecl => {
                let is_static = child.get_kind() == EntityKind::VarDecl;
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
                    is_mutable: !is_static && child.is_mutable(),
                    is_static,
                    accessibility,
                    field_offset,
                });
            }
            _ => {}
        }
    }

    Some(TemplateClassInfo {
        name,
        type_params,
        bases,
        methods,
        fields,
        is_from_current_file,
    })
}

/// 提取模板函数（`FunctionTemplate`）的结构化信息 — v6 Phase A。
pub(super) fn extract_template_function(
    entity: &clang::Entity<'_>,
    cpp_ranges: &[std::ops::Range<u32>],
) -> Option<TemplateFunctionInfo> {
    let name = entity.get_name()?;
    // 与全局函数一致：跳过操作符模板（无法表示为合法 Rust 名称）
    if name.starts_with("operator") {
        return None;
    }
    let return_type = entity
        .get_result_type()
        .map(|t| t.get_display_name())
        .unwrap_or_default();
    // FunctionTemplate 不通过 get_arguments() 暴露参数，需遍历 ParmDecl 子节点。
    let params: Vec<ParamInfo> = entity
        .get_children()
        .iter()
        .filter(|c| c.get_kind() == EntityKind::ParmDecl)
        .map(|arg| {
            let name = arg.get_name().unwrap_or_else(|| "_".to_string());
            let type_name = arg
                .get_type()
                .map(|t| t.get_display_name())
                .unwrap_or_default();
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
        .collect();
    let type_params = collect_template_params(entity);
    let is_from_current_file = entity_is_from_current_file(entity, cpp_ranges);

    Some(TemplateFunctionInfo {
        name,
        type_params,
        return_type,
        params,
        is_from_current_file,
    })
}

pub(super) fn extract_method(entity: &clang::Entity<'_>) -> Option<MethodInfo> {
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
        is_override: !entity
            .get_overridden_methods()
            .unwrap_or_default()
            .is_empty(),
        is_default: entity.is_defaulted(),
    })
}

pub(super) fn extract_function(
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

    let is_variadic = entity.get_type().map(|t| t.is_variadic()).unwrap_or(false);

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

    // 通过 libclang 语言标记检测 C 链接（extern "C"）。
    // 这比仅依赖父节点为 LinkageSpec 更可靠：在 Windows LLVM 17 上，
    // extern "C" 函数有时作为顶层 FunctionDecl 出现而非 LinkageSpec 的子节点，
    // 此时父节点法会遗漏这些函数。collect_linkage_spec 路径也会显式覆盖此字段。
    let is_extern_c = entity.get_language() == Some(Language::C);

    Some(FunctionInfo {
        name,
        return_type,
        params,
        is_inline: entity.is_inline_function(),
        is_variadic,
        is_extern_c,
        friend_of: friend_of.map(String::from),
        body_offset,
        is_from_current_file,
    })
}

pub(super) fn extract_params(entity: &clang::Entity<'_>) -> Vec<ParamInfo> {
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

pub(super) fn extract_enum(entity: &clang::Entity<'_>) -> Option<EnumInfo> {
    let name = entity.get_name()?;
    let underlying_type = entity
        .get_enum_underlying_type()
        .map(|t| t.get_display_name())
        .unwrap_or_else(|| "int".to_string());

    // enum class（scoped enum）检测：
    // libclang 对 scoped enum 的变体不附加枚举名前缀（变体名就是简单名称），
    // 而对 unscoped enum，libclang 返回的变体名同样不带前缀（只是纯变体名）。
    // 区分方式：检查 entity 的 DisplayName 是否以 "enum " 开头（unscoped）
    // 还是 "enum class " 或 "enum struct " 开头（scoped）。
    // 若 display_name 不可用，回退为启发式：第一个变体名以枚举名开头视为 unscoped。
    let is_scoped = {
        let display = entity.get_display_name().unwrap_or_default();
        if display.starts_with("enum class ") || display.starts_with("enum struct ") {
            true
        } else if display.starts_with("enum ") {
            false
        } else {
            // 回退启发式：scoped enum 的变体名不以枚举名开头
            !entity
                .get_children()
                .first()
                .and_then(|c| c.get_name())
                .map(|cn| cn.starts_with(&name))
                .unwrap_or(false)
        }
    };

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
