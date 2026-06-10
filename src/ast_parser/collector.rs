//! 类/函数/枚举收集逻辑 — 从 libclang 实体树提取结构化信息
//!
//! 供 `mod.rs` 中的 `parse_preprocessed` 调用。

use clang::{EntityKind, Language};

use super::range_scanner::{entity_is_from_current_file, entity_presumed_from_user_file};
use super::{
    BaseInfo, ClassInfo, CppAst, EnumInfo, EnumVariantInfo, FieldInfo, FunctionInfo, MethodInfo,
    ParamInfo,
};

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
                    // 只收集来自当前 .cpp 文件的命名空间函数；
                    // 第三方库头文件中的内部实现函数（如 rapidjson::internal::clzll）
                    // 不应作为 FFI 导出，且其函数体引用了库内部类型，内联会导致编译失败。
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
                collect_typedef(&entity, ast, cpp_ranges);
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
                collect_typedef(&entity, ast, cpp_ranges);
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
    cpp_ranges: &[std::ops::Range<u32>],
) {
    // 只收集来自当前 .cpp 文件（cpp_ranges）的 typedef，
    // 排除通过 #include 引入的系统/第三方头文件中的 typedef（避免类型污染）。
    // 例如 rapidjson::internal::BoolType 等内部 typedef 落在第三方头文件范围内，
    // 若被收集并输出到 cpp! 块全局作用域，会因命名空间不在作用域内而报编译错误。
    if !entity_is_from_current_file(entity, cpp_ranges) {
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

    // enum class（scoped enum）通过检查子节点中不含有同名符号来判断
    // libclang 2.x 没有直接的 is_scoped_enum API；用 display_name 含 "::" 来近似
    let is_scoped = entity
        .get_children()
        .first()
        .and_then(|c| c.get_name())
        .map(|cn| cn.starts_with(&name))
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
