//! C++ 信息提取器（Phase 2）
//!
//! 从 `CppAst` 和原始源信息提取 `FfiSpec` IR，供代码生成器使用。

pub mod type_mapper;

use crate::ast_parser::{ClassInfo, CppAst, FieldInfo, FunctionInfo, MethodInfo, ParamInfo};
use crate::ffi_model::{ClassSpec, FfiSpec, FnBinding, LibSpec, MethodBinding, SelfKind};
use std::fs;
use type_mapper::{clean_type, cpp_to_rust, to_snake_case};

// ─────────────────────────────────────────────
//  公共入口
// ─────────────────────────────────────────────

/// 从 `CppAst` 提取 `FfiSpec`。
pub fn extract(
    ast: &CppAst,
    unit_name: &str,
    system_includes: &[String],
    project_header: Option<&str>,
) -> FfiSpec {
    let source_bytes = fs::read(&ast.file).unwrap_or_default();
    // has_any_classes：是否存在任何类（含命名空间类），用于 namespace_class_mode 检测
    let has_any_classes = !ast.classes.is_empty();
    // has_classes：是否存在非命名空间的物理类，用于决定 cpp! 块模式（project header vs inline class）
    let has_classes = ast.classes.iter().any(|c| !c.is_in_namespace);

    // 去重：对于同名函数，只保留一个（有 body_offset 的优先；否则 is_extern_c=false 优先）
    // 只纳入来自当前 .cpp 文件本身或显式 extern "C" 声明的函数，
    // 过滤掉通过 #include 引入的头文件内部函数（它们不应被导出为 FFI）。
    let eligible_functions: Vec<FunctionInfo> = ast
        .functions
        .iter()
        .filter(|f| f.is_from_current_file || f.is_extern_c)
        .cloned()
        .collect();
    let functions = dedup_functions(&eligible_functions);

    // ── 计算函数签名中引用的类名集合 ─────────────
    // 先检查 extern-C 函数，若无则检查所有符合条件的函数（有些 header 不用 extern "C" 包裹）
    let used_classes: std::collections::HashSet<String> = {
        let mut set = std::collections::HashSet::new();
        let all_cn: Vec<&str> = ast.classes.iter().map(|c| c.name.as_str()).collect();
        let candidate_fns: Vec<&FunctionInfo> = {
            let extern_c: Vec<&FunctionInfo> = eligible_functions
                .iter()
                .filter(|f| f.is_extern_c)
                .collect();
            if extern_c.is_empty() {
                eligible_functions.iter().collect()
            } else {
                extern_c
            }
        };
        for fi in &candidate_fns {
            for cn in &all_cn {
                if fi.return_type.contains(cn) || fi.params.iter().any(|p| p.type_name.contains(cn))
                {
                    set.insert(cn.to_string());
                }
            }
        }
        set
    };

    // ── 检测命名空间/opaque 类模式 ───────────────
    // 当且仅当：有类存在 AND 无类名出现在函数签名 AND 至少一个 extern-C 函数的参数/返回类型
    // 包含 `::` 或 `void*`（说明类通过命名空间限定类型或 opaque 指针暴露，hicc 无法处理）
    // 这影响 cpp! 块内容与 import_class! 生成：
    //   043: void* opaque 指针（命名空间类）→ cpp! 只 include 头文件，不生成 import_class!
    //   044: example::OperationResult* 命名空间类型指针 → 同样只 include 头文件
    //   028: int/double 原始类型（辅助类）→ cpp! 内联类定义，正常生成 import_class!
    // 注意：import_lib! 的生成不受此模式影响，始终由 build_lib_spec 决定（内部有类型过滤）
    let namespace_class_mode = has_any_classes && used_classes.is_empty() && {
        eligible_functions.iter().any(|f| {
            f.is_extern_c && {
                let rt = &f.return_type;
                rt.contains("::")
                    || rt.contains("void *")
                    || rt.contains("void*")
                    || f.params.iter().any(|p| {
                        let t = &p.type_name;
                        t.contains("::") || t.contains("void *") || t.contains("void*")
                    })
            }
        })
    };

    // ── hicc::cpp! 块内容 ──────────────────────
    let cpp_block_lines = if namespace_class_mode {
        // 命名空间类模式：只生成项目头文件 include，不内联类体
        if let Some(hdr) = project_header {
            vec![format!("#include \"{}\"", hdr)]
        } else {
            Vec::new()
        }
    } else {
        build_cpp_block(
            ast,
            &functions,
            &source_bytes,
            system_includes,
            project_header,
            has_classes,
        )
    };

    // ── import_class! 块列表 ──────────────────
    // 只为 extern-C 函数签名中明确引用的类生成 import_class!
    // 若 used_classes 为空（无类被引用），则不生成任何 import_class!
    let class_specs: Vec<ClassSpec> = if namespace_class_mode || used_classes.is_empty() {
        Vec::new()
    } else {
        ast.classes
            .iter()
            .filter(|c| !c.name.is_empty())
            .filter(|c| used_classes.contains(&c.name))
            .map(|ci| {
                build_class_spec(ci, &ast.classes).unwrap_or_else(|| ClassSpec {
                    name: ci.name.clone(),
                    methods: Vec::new(),
                    associated_fns: Vec::new(),
                    destroy_fn: None,
                    is_interface: false,
                })
            })
            .collect()
    };

    // ── import_lib! 块 ────────────────────────
    // 始终调用 build_lib_spec：其内部的 is_mappable_rust_type 过滤器会自动排除
    // 含 `::` 的命名空间类型（如 std::string*、example::OperationResult*），
    // 而 void* → *mut u8 等可映射类型则正常生成绑定。
    let lib_spec = {
        let class_names: Vec<&str> = ast.classes.iter().map(|c| c.name.as_str()).collect();
        build_lib_spec(&functions, unit_name, &class_names)
    };

    let mut spec = FfiSpec {
        unit_name: unit_name.to_string(),
        cpp_block_lines,
        class_specs,
        lib_spec,
    };

    // ── 后处理器 ──────────────────────────────
    crate::postprocessor::diamond_handler::apply(&mut spec, ast, &functions);
    crate::postprocessor::operator_handler::apply(&mut spec, ast, &functions);

    // ── 关联函数归属（ctor/dtor/factory → ClassSpec::associated_fns）──────
    // 将 import_lib! 中属于某个类的 ctor/dtor/StaticAccessor 函数
    // 移至对应 ClassSpec::associated_fns，使代码生成器可输出 class body 格式
    if !spec.class_specs.is_empty() {
        let class_names_owned: Vec<String> = ast.classes.iter().map(|c| c.name.clone()).collect();
        let class_names_ref: Vec<&str> = class_names_owned.iter().map(|s| s.as_str()).collect();
        assign_associated_fns(
            &mut spec.class_specs,
            &mut spec.lib_spec,
            &functions,
            &class_names_ref,
        );
    }

    spec
}

/// 去重：对于同名函数优先保留 body_offset 且 is_extern_c=false 的版本
fn dedup_functions<'a>(functions: &'a [FunctionInfo]) -> Vec<&'a FunctionInfo> {
    let mut map: std::collections::HashMap<&str, &'a FunctionInfo> =
        std::collections::HashMap::new();

    for fi in functions {
        let entry = map.entry(fi.name.as_str()).or_insert(fi);
        // 替换规则：有 body_offset 且不是 extern_c 的版本胜出
        let new_score = score(fi);
        let old_score = score(entry);
        if new_score > old_score {
            *entry = fi;
        }
    }

    // 按原始顺序输出
    let mut result: Vec<&'a FunctionInfo> = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for fi in functions {
        if !seen.contains(fi.name.as_str()) {
            if let Some(&best) = map.get(fi.name.as_str()) {
                result.push(best);
                seen.insert(fi.name.as_str());
            }
        }
    }
    result
}

fn score(fi: &FunctionInfo) -> u8 {
    match (fi.body_offset.is_some(), fi.is_extern_c) {
        (true, false) => 3, // best: has body, not extern_c
        (true, true) => 2,
        (false, false) => 1,
        (false, true) => 0, // worst: declaration in extern "C"
    }
}

// ─────────────────────────────────────────────
//  hicc::cpp! 块构建
// ─────────────────────────────────────────────

fn build_cpp_block(
    ast: &CppAst,
    functions: &[&FunctionInfo],
    source_bytes: &[u8],
    system_includes: &[String],
    project_header: Option<&str>,
    has_classes: bool,
) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();

    if !has_classes {
        // 函数-only：仅放项目头文件 include
        if let Some(hdr) = project_header {
            lines.push(format!("#include \"{}\"", hdr));
        }
        return lines;
    }

    // 有类：放系统 includes
    for inc in system_includes {
        lines.push(inc.clone());
    }
    if !system_includes.is_empty() {
        lines.push(String::new());
    }

    // 只内联来自当前文件的类，来自 include 头文件的类通过 #include 引入
    let local_classes: Vec<&ClassInfo> = ast
        .classes
        .iter()
        .filter(|c| c.is_from_current_file)
        .collect();

    // 当所有类均来自头文件（local_classes 为空）时：
    // - include 项目头文件以引入类型定义和函数声明
    // - 不再重复 emit 枚举（它们已包含在头文件中，重复会导致重复定义错误）
    // - 不再重复 emit shim 函数体（项目 .cpp 中已有定义，重复会导致 duplicate symbol 链接错误）
    let use_project_header = local_classes.is_empty() && project_header.is_some();
    if use_project_header {
        if let Some(hdr) = project_header {
            lines.push(format!("#include \"{}\"", hdr));
        }
    } else {
        // 枚举定义（在类定义之前；仅在不使用项目头文件时输出，避免重复定义）
        for en in &ast.enums {
            if en.name.is_empty() {
                continue;
            }
            lines.push(format!("enum {} {{", en.name));
            for v in &en.variants {
                lines.push(format!("    {} = {},", v.name, v.value));
            }
            lines.push("};".to_string());
            lines.push(String::new());
        }
    }

    // typedef 定义（在类定义之前；typedef 重复声明在 C++11 中合法，始终输出）
    for (_, start, end) in &ast.typedefs {
        let text = extract_range_text(source_bytes, *start, *end)
            .trim()
            .to_string();
        if text.is_empty() {
            continue;
        }
        let stmt = if text.ends_with(';') {
            text
        } else {
            format!("{};", text)
        };
        lines.push(stmt);
        lines.push(String::new());
    }

    // 模板类定义（在 typedef 之后、具体类之前）
    for (_, start, end) in &ast.template_class_ranges {
        let text = extract_range_text(source_bytes, *start, *end);
        let text = strip_preprocessor_markers(text.trim());
        let trimmed = text.trim();
        if !trimmed.is_empty() {
            // libclang 的 range end 指向末尾 `;` 位置，extract_range_text 用 [start..end]（不含 end），
            // 因此提取结果不含 `;`。若末尾是 `}` 则补全 `;`。
            let text_with_semi = if trimmed.ends_with('}') {
                format!("{};", trimmed)
            } else {
                trimmed.to_string()
            };
            for line in text_with_semi.lines() {
                lines.push(line.to_string());
            }
            lines.push(String::new());
        }
    }

    let class_names: Vec<&str> = ast.classes.iter().map(|c| c.name.as_str()).collect();

    // 判断是否使用分离风格（含虚函数的类）
    let use_separate_style = ast
        .classes
        .iter()
        .any(|c| c.methods.iter().any(|m| m.is_virtual));

    if use_separate_style {
        // 分离风格：先放所有类的声明，再放方法实现
        for ci in &local_classes {
            emit_class_decl(ci, source_bytes, &mut lines);
            lines.push(String::new());
        }
        // 方法定义（从源文件读取，跳过 = default / = delete 方法）
        for ci in &local_classes {
            for method in &ci.methods {
                if method.is_default {
                    continue;
                }
                if let Some((start, end)) = method.body_offset {
                    let text = extract_range_text(source_bytes, start, end);
                    let text = strip_preprocessor_markers(text.trim());
                    let trimmed = text.trim();
                    // 跳过不含函数体（= default / = delete）的情况
                    if trimmed.is_empty()
                        || (!trimmed.contains('{')
                            && (trimmed.contains("= default") || trimmed.contains("= delete")))
                    {
                        continue;
                    }
                    for line in trimmed.lines() {
                        lines.push(line.to_string());
                    }
                    lines.push(String::new());
                }
            }
        }
    } else {
        // 内联风格：类定义含内联方法体
        for ci in &local_classes {
            emit_class_inline(ci, source_bytes, &mut lines);
            lines.push(String::new());
            // 静态成员变量初始化
            for field in &ci.fields {
                if field.is_static {
                    if let Some(init) = find_static_init(source_bytes, &ci.name, &field.name) {
                        lines.push(init);
                        lines.push(String::new());
                    } else {
                        lines.push(format!(
                            "{} {}::{};",
                            clean_type(&field.type_name),
                            ci.name,
                            field.name
                        ));
                        lines.push(String::new());
                    }
                }
            }
        }
    }

    // Ctor/dtor/standalone shim 函数（含静态访问器）
    // 当使用 project_header 模式时，函数已通过头文件中的 extern "C" 声明引入，
    // 实现体在项目 .cpp 文件中，不需要（也不应）在 cpp! 块中重复定义。
    if !use_project_header {
        let shim_fns = classify_functions(functions, &class_names);
        for (fn_info, shim_kind) in &shim_fns {
            if !matches!(shim_kind, ShimKind::MethodAccessor) {
                if let Some((start, end)) = fn_info.body_offset {
                    let raw = extract_range_text(source_bytes, start, end);
                    let cleaned = clean_shim_text(&raw);
                    let cleaned = strip_preprocessor_markers(&cleaned);
                    let trimmed = cleaned.trim();
                    if !trimmed.is_empty() {
                        for line in trimmed.lines() {
                            lines.push(line.to_string());
                        }
                        lines.push(String::new());
                    }
                }
            }
        }
    }

    // 去掉末尾多余空行
    while lines.last().map(|l| l.is_empty()).unwrap_or(false) {
        lines.pop();
    }

    lines
}

/// 生成类前向声明（分离风格）
fn emit_class_decl(ci: &ClassInfo, source_bytes: &[u8], lines: &mut Vec<String>) {
    let keyword = if ci.is_struct { "struct" } else { "class" };
    let bases_str = format_bases(&ci.bases);
    lines.push(format!("{} {}{} {{", keyword, ci.name, bases_str));
    emit_fields_by_access(ci, source_bytes, lines);
    emit_method_decls(ci, lines);
    lines.push("};".to_string());
}

/// 生成内联类定义（简单类风格，方法含方法体）
fn emit_class_inline(ci: &ClassInfo, source_bytes: &[u8], lines: &mut Vec<String>) {
    let keyword = if ci.is_struct { "struct" } else { "class" };
    let bases_str = format_bases(&ci.bases);
    lines.push(format!("{} {}{} {{", keyword, ci.name, bases_str));

    emit_fields_by_access(ci, source_bytes, lines);

    // 对 public 方法去重：同名+同参数类型时优先保留有 body_offset 的版本，
    // 按原始顺序输出每个 (name, param_types) 键的第一次出现位置。
    let all_pub: Vec<&MethodInfo> = ci
        .methods
        .iter()
        .filter(|m| m.accessibility == "public")
        .collect();

    let mut seen_keys: std::collections::HashSet<(String, String)> =
        std::collections::HashSet::new();
    let mut pub_methods: Vec<&MethodInfo> = Vec::new();
    for m in &all_pub {
        let param_types: String = m
            .params
            .iter()
            .map(|p| p.type_name.as_str())
            .collect::<Vec<_>>()
            .join(",");
        let key = (m.name.clone(), param_types.clone());
        if seen_keys.insert(key) {
            // 若存在同签名的有-body 版本，则用它替代第一次出现的无-body 版本
            let best = all_pub
                .iter()
                .find(|x| {
                    x.name == m.name
                        && {
                            let xt: String = x
                                .params
                                .iter()
                                .map(|p| p.type_name.as_str())
                                .collect::<Vec<_>>()
                                .join(",");
                            xt == param_types
                        }
                        && x.body_offset.is_some()
                })
                .copied()
                .unwrap_or(m);
            pub_methods.push(best);
        }
    }

    if !pub_methods.is_empty() {
        let has_non_pub =
            !ci.fields.is_empty() && ci.fields.iter().any(|f| f.accessibility != "public");
        if has_non_pub || !ci.is_struct {
            lines.push("public:".to_string());
        }
        for method in &pub_methods {
            let text = build_inline_method_line(method, source_bytes, &ci.name);
            lines.push(format!("    {}", text));
        }
    }

    lines.push("};".to_string());
}

/// 格式化基类列表
fn format_bases(bases: &[crate::ast_parser::BaseInfo]) -> String {
    if bases.is_empty() {
        return String::new();
    }
    let b: Vec<String> = bases
        .iter()
        .map(|b| {
            let virt = if b.is_virtual { "virtual " } else { "" };
            format!("public {}{}", virt, clean_type(&b.name))
        })
        .collect();
    format!(" : {}", b.join(", "))
}

/// 按访问控制分组输出字段
fn emit_fields_by_access(ci: &ClassInfo, source_bytes: &[u8], lines: &mut Vec<String>) {
    let accesses = ["private", "protected", "public"];
    for acc in accesses {
        let group: Vec<&FieldInfo> = ci
            .fields
            .iter()
            .filter(|f| f.accessibility == acc)
            .collect();
        if group.is_empty() {
            continue;
        }
        // class 默认 private，不需要 label；其他需要
        if acc != "private" {
            lines.push(format!("{}:", acc));
        }
        for field in &group {
            let field_text = emit_field_line(field, source_bytes);
            lines.push(format!("    {}", field_text));
        }
    }
}

fn emit_field_line(field: &FieldInfo, source_bytes: &[u8]) -> String {
    let static_kw = if field.is_static { "static " } else { "" };
    if let Some((start, end)) = field.field_offset {
        let raw = extract_range_text(source_bytes, start, end);
        let trimmed = raw.trim();
        if !trimmed.is_empty() {
            // 源文件文本已含 static 关键字，不再重复添加
            let cleaned = clean_shim_text(trimmed);
            return format!("{};", cleaned.trim());
        }
    }
    // 回退：构造
    let ty = clean_type(&field.type_name);
    format!("{}{} {};", static_kw, ty, field.name)
}

/// 输出方法声明（分离风格，无方法体）
fn emit_method_decls(ci: &ClassInfo, lines: &mut Vec<String>) {
    let pub_methods: Vec<&MethodInfo> = ci
        .methods
        .iter()
        .filter(|m| m.accessibility == "public")
        .collect();
    if pub_methods.is_empty() {
        return;
    }
    lines.push("public:".to_string());
    for method in pub_methods {
        let decl = build_method_decl(method);
        lines.push(format!("    {};", decl));
    }
}

/// 构建单个方法声明（无方法体）
fn build_method_decl(m: &MethodInfo) -> String {
    // 前缀修饰词
    let qualifier = if m.is_override {
        // overriding: no virtual prefix
        String::new()
    } else if m.is_pure_virtual || m.is_virtual {
        "virtual ".to_string()
    } else if m.is_static {
        "static ".to_string()
    } else {
        String::new()
    };

    let ret = if m.is_constructor || m.is_destructor {
        String::new()
    } else {
        format!("{} ", normalize_ptr_spacing(clean_type(&m.return_type)))
    };

    // 析构函数名：libclang 返回 "~ClassName"，直接使用
    let name = if m.is_destructor {
        if m.name.starts_with('~') {
            m.name.clone()
        } else {
            format!("~{}", m.name)
        }
    } else {
        m.name.clone()
    };

    let params = format_params_cpp(&m.params);
    let const_sfx = if m.is_const { " const" } else { "" };
    let volatile_sfx = if m.is_volatile { " volatile" } else { "" };
    let override_sfx = if m.is_override { " override" } else { "" };
    let pure_sfx = if m.is_pure_virtual && !m.is_override {
        " = 0"
    } else {
        ""
    };
    let default_sfx = if m.is_default { " = default" } else { "" };

    format!(
        "{}{}{}({}){}{}{}{}{}",
        qualifier, ret, name, params, const_sfx, volatile_sfx, override_sfx, pure_sfx, default_sfx
    )
}

/// 构建单行内联方法（内联风格）
fn build_inline_method_line(m: &MethodInfo, source_bytes: &[u8], class_name: &str) -> String {
    // = default 方法：直接用规范化声明，不从源码读取（避免 `= default {}` 等错误语法）
    if m.is_default {
        return format!("{};", build_method_decl(m));
    }

    if let Some((start, end)) = m.body_offset {
        let raw_text = extract_range_text(source_bytes, start, end);
        let stripped = strip_class_prefix(raw_text.trim(), class_name);
        let stripped = strip_preprocessor_markers(&stripped);
        let stripped = strip_method_volatile_qualifier(stripped.trim());
        // 去掉方法返回类型的 volatile 前缀（与 import_class! 中的 cpp_sig 保持一致）
        let stripped = if let Some(s) = stripped.strip_prefix("volatile ") {
            s.trim_start().to_string()
        } else {
            stripped
        };
        let stripped = stripped.trim().to_string();

        // 对静态方法：若提取文本未含 static，补加前缀
        if m.is_static && !stripped.trim_start().starts_with("static") {
            let s = format!("static {}", stripped);
            // 补全末尾分号（libclang range 有时不包含 `;`）
            return if !s.ends_with(';') && !s.ends_with('}') {
                format!("{};", s)
            } else {
                s
            };
        }
        // 补全末尾分号（libclang range 有时不包含 `;`）
        return if !stripped.ends_with(';') && !stripped.ends_with('}') {
            format!("{};", stripped)
        } else {
            stripped
        };
    }

    // 没有 body_offset → 生成 `= default;` 或 `{}`
    let decl = build_method_decl(m);
    if m.is_constructor || m.is_destructor {
        // is_default 时 build_method_decl 已含 " = default"，直接加分号；
        // 否则补充 " = default;"（用于无实现的普通 ctor/dtor）
        if m.is_default {
            format!("{};", decl)
        } else {
            format!("{} = default;", decl)
        }
    } else if m.is_pure_virtual {
        format!("{};", decl)
    } else {
        format!("{} {{}}", decl)
    }
}

/// 检测函数体文本是否为空（仅含 `{ }` 或 `: init_list {}`，大括号内无语句）
#[allow(dead_code)]
fn has_empty_body(text: &str) -> bool {
    if let Some(open) = text.rfind('{') {
        if let Some(close) = text.rfind('}') {
            if close > open {
                let inner = text[open + 1..close].trim();
                return inner.is_empty();
            }
        }
    }
    false
}

/// 过滤预处理器行号标记，如 `# 26 "file.cpp" 3 4`
fn strip_preprocessor_markers(text: &str) -> String {
    text.lines()
        .filter(|line| {
            let t = line.trim();
            if !t.starts_with('#') {
                return true;
            }
            let rest = t[1..].trim_start();
            // 丢弃 `# <数字> "file"` 形式的行号标记
            !rest.starts_with(|c: char| c.is_ascii_digit())
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// 清理 shim 函数文本：去除类型前的 `struct ` / `class ` 关键字前缀。
/// 例：`struct Foo*` → `Foo*`，`class Bar*` → `Bar*`。
/// 先清理 `struct`，再清理 `class`，顺序不影响结果但保持确定性。
fn clean_shim_text(text: &str) -> String {
    let after_struct = clean_shim_keyword(text, "struct ");
    clean_shim_keyword(&after_struct, "class ")
}

/// 从文本中去除独立出现的 C++ 关键字前缀（`struct ` 或 `class `），
/// 只去除出现在行首、空白符、括号或逗号之后的实例，保留标识符中间的情况。
fn clean_shim_keyword(text: &str, keyword: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut rest = text;
    let kw_len = keyword.len();
    while let Some(pos) = rest.find(keyword) {
        result.push_str(&rest[..pos]);
        let prev_ok =
            pos == 0 || matches!(rest.as_bytes()[pos - 1], b' ' | b'\n' | b'\t' | b'(' | b',');
        // 无论是否跳过，都推进指针以防止死循环
        rest = &rest[pos + kw_len..];
        if !prev_ok {
            result.push_str(keyword);
        }
    }
    result.push_str(rest);
    result
}

// ─────────────────────────────────────────────
//  import_class! 块
// ─────────────────────────────────────────────

fn build_class_spec(ci: &ClassInfo, all_classes: &[ClassInfo]) -> Option<ClassSpec> {
    // 收集本类的 public 非 ctor/dtor 方法（跳过 operator 重载和 Rust 关键字方法名）
    let own_methods: Vec<&MethodInfo> = ci
        .methods
        .iter()
        .filter(|m| {
            !m.is_constructor && !m.is_destructor && m.accessibility == "public" && !m.is_static
        })
        .filter(|m| !m.name.starts_with("operator") && !is_rust_keyword(&to_snake_case(&m.name)))
        .collect();

    // 收集所有基类的 public 方法（递归，保持顺序）
    let inherited = collect_inherited_methods(ci, all_classes);

    // 合并：继承方法 + 本类覆盖/新增方法
    // 规则：如果本类有同名方法（override），用本类的；否则用继承的
    let own_names: std::collections::HashSet<&str> =
        own_methods.iter().map(|m| m.name.as_str()).collect();

    let mut methods: Vec<MethodBinding> = Vec::new();

    // 先放继承来的（本类未覆盖的，同样跳过 operator 和 Rust 关键字方法名）
    for im in &inherited {
        if !own_names.contains(im.name.as_str())
            && !im.name.starts_with("operator")
            && !is_rust_keyword(&to_snake_case(&im.name))
        {
            if let Some(mb) = build_method_binding(im) {
                methods.push(mb);
            }
        }
    }

    // 再放本类的方法（按原始顺序：覆盖的和新增的）
    for m in &own_methods {
        if let Some(mb) = build_method_binding(m) {
            methods.push(mb);
        }
    }

    if methods.is_empty() {
        return None;
    }

    // 检测纯虚接口类：所有 public 非 ctor/dtor 方法（含继承）均为纯虚
    let is_interface = !own_methods.is_empty()
        && own_methods.iter().all(|m| m.is_pure_virtual)
        && inherited.iter().all(|m| m.is_pure_virtual);

    Some(ClassSpec {
        name: ci.name.clone(),
        methods,
        associated_fns: Vec::new(),
        destroy_fn: None,
        is_interface,
    })
}

/// 递归收集所有基类的 public 非 ctor/dtor 方法（不含静态方法）
fn collect_inherited_methods<'a>(
    ci: &ClassInfo,
    all_classes: &'a [ClassInfo],
) -> Vec<&'a MethodInfo> {
    let mut result: Vec<&'a MethodInfo> = Vec::new();
    let mut seen: std::collections::HashSet<&str> = std::collections::HashSet::new();

    for base in &ci.bases {
        let base_name = clean_type(&base.name).to_string();
        if let Some(base_ci) = all_classes.iter().find(|c| c.name == base_name) {
            // 先递归收集基类的基类
            let grand_inherited = collect_inherited_methods(base_ci, all_classes);
            for m in grand_inherited {
                if seen.insert(m.name.as_str()) {
                    result.push(m);
                }
            }
            // 再收集本基类的方法
            for m in base_ci.methods.iter().filter(|m| {
                !m.is_constructor && !m.is_destructor && m.accessibility == "public" && !m.is_static
            }) {
                if seen.insert(m.name.as_str()) {
                    result.push(m);
                }
            }
        }
    }
    result
}

fn build_method_binding(m: &MethodInfo) -> Option<MethodBinding> {
    // hicc 不支持 volatile this 限定的方法（方法指针类型不匹配），跳过
    if m.is_volatile {
        return None;
    }
    // C++ 成员函数指针无法映射为有效 Rust FFI 类型，跳过
    if m.params.iter().any(|p| p.type_name.contains("::*)"))
        || m.return_type.contains("::*)")
    {
        return None;
    }
    let rust_name = sanitize_fn_name(&m.name);
    let self_kind = if m.is_const {
        SelfKind::Ref
    } else {
        SelfKind::RefMut
    };

    let params: Vec<(String, String)> = m
        .params
        .iter()
        .enumerate()
        .map(|(i, p)| (sanitize_param_name(&p.name, i), cpp_to_rust(&p.type_name)))
        .collect();

    let ret_type = ret_type_from_cpp(&m.return_type);

    // 检测参数或返回类型是否含 C 函数指针，用于生成 cpp2rust-todo[FP] 注释
    let has_fn_ptr_param = m.params.iter().any(|p| p.type_name.contains("(*)"))
        || m.return_type.contains("(*)");

    // C++ 方法签名：含参数名（若 AST 有）、剥除参数 volatile、指针紧贴类型
    // 返回类型 volatile 和方法 this-volatile 均需保留，供 hicc 编译时方法指针类型检查
    let param_types: Vec<String> = m
        .params
        .iter()
        .map(|p| {
            let ty = normalize_ptr_spacing(strip_volatile(clean_type(&p.type_name)));
            // C++ 签名中：有名字则 "type name"，无名则仅 "type"
            if !p.name.is_empty() && p.name != "_" {
                format!("{} {}", ty, p.name)
            } else {
                ty.to_string()
            }
        })
        .collect();
    let ret_clean = normalize_ptr_spacing(strip_volatile(clean_type(&m.return_type)));
    let cv_suffix = match (m.is_const, m.is_volatile) {
        (true, true) => " const volatile",
        (true, false) => " const",
        (false, true) => " volatile",
        (false, false) => "",
    };
    let cpp_sig = if m.return_type.is_empty() || m.return_type == "void" {
        format!("void {}({}){}", m.name, param_types.join(", "), cv_suffix)
    } else {
        format!(
            "{} {}({}){}",
            ret_clean,
            m.name,
            param_types.join(", "),
            cv_suffix
        )
    };

    Some(MethodBinding {
        cpp_sig,
        rust_name,
        self_kind,
        params,
        ret_type,
        has_fn_ptr_param,
    })
}

// ─────────────────────────────────────────────
//  import_lib! 块
// ─────────────────────────────────────────────

fn build_lib_spec(functions: &[&FunctionInfo], unit_name: &str, class_names: &[&str]) -> LibSpec {
    let shims = classify_functions(functions, class_names);
    let fn_bindings: Vec<FnBinding> = shims
        .iter()
        .filter(|(_, k)| !matches!(k, ShimKind::MethodAccessor))
        .filter(|(fi, _)| !fi.is_variadic)
        .filter(|(fi, _)| !fi.name.starts_with("operator"))
        // C++ 成员函数指针（如 `int (Cls::*)() const`）无法映射为有效 Rust FFI 类型，跳过整个函数
        .filter(|(fi, _)| !fi.params.iter().any(|p| p.type_name.contains("::*)")))
        // 返回类型含 C++ 成员函数指针语法，同样无法映射为有效 Rust FFI 类型，跳过整个函数
        .filter(|(fi, _)| !fi.return_type.contains("::*)"))
        // 参数或返回类型经 cpp_to_rust 映射后仍是无法在 Rust FFI 中使用的类型
        // （如未声明的 C 类型 FILE、未知 C++ 类型 MessageMap、含命名空间的 std::string 等），
        // 跳过整个函数以避免生成无法编译的绑定代码
        .filter(|(fi, _)| {
            fi.params
                .iter()
                .all(|p| is_mappable_rust_type(&cpp_to_rust(&p.type_name), class_names))
                && is_mappable_rust_type(&cpp_to_rust(&fi.return_type), class_names)
        })
        .map(|(fi, _)| build_fn_binding(fi, class_names))
        .collect();

    // 前向声明：只包含在函数签名中实际引用的类（按原始顺序）
    let used_classes: std::collections::HashSet<&str> = fn_bindings
        .iter()
        .flat_map(|fb| {
            class_names.iter().filter(move |cn| {
                fb.cpp_sig.contains(*cn)
                    || fb.params.iter().any(|(_, t)| t.contains(*cn))
                    || fb
                        .ret_type
                        .as_ref()
                        .map(|r| r.contains(*cn))
                        .unwrap_or(false)
            })
        })
        .copied()
        .collect();
    let fwd_decls: Vec<String> = class_names
        .iter()
        .filter(|cn| used_classes.contains(**cn))
        .map(|n| format!("class {};", n))
        .collect();

    // link_name 只取路径末段（文件名），避免将模块路径（如 "unittest/documenttest"）
    // 直接用作链接库名导致 hicc-build 无法找到对应的编译产物。
    // 使用 std::path::Path::file_name() 而非手动拆分 '/'，跨平台更安全。
    let link_name = std::path::Path::new(unit_name)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or(unit_name)
        .to_string();

    LibSpec {
        link_name,
        fwd_decls,
        fn_bindings,
    }
}

fn build_fn_binding(fi: &FunctionInfo, class_names: &[&str]) -> FnBinding {
    let rust_name = sanitize_fn_name(&fi.name);
    let params: Vec<(String, String)> = fi
        .params
        .iter()
        .enumerate()
        .map(|(i, p)| {
            (
                sanitize_param_name(&p.name, i),
                cpp_to_rust(&p.type_name),
            )
        })
        .collect();

    let ret_type = ret_type_from_cpp(&fi.return_type);

    // unsafe: 参数中有裸指针（*mut T 或 *const i8），或返回值为裸 C 字符串
    // 例外：*mut ClassType 且返回值是原始类型（i8/u8/i16/u16/i32/u32/i64/u64/f32/f64/bool/isize/usize）
    //        且参数不含 volatile 限定 → NOT unsafe
    let primitive_ret = ret_type
        .as_deref()
        .map(|r| {
            matches!(
                r,
                "i8" | "u8"
                    | "i16"
                    | "u16"
                    | "i32"
                    | "u32"
                    | "i64"
                    | "u64"
                    | "f32"
                    | "f64"
                    | "bool"
                    | "isize"
                    | "usize"
            )
        })
        .unwrap_or(false);
    let has_volatile_param = fi
        .params
        .iter()
        .any(|p| p.type_name.split_whitespace().any(|w| w == "volatile"));
    let is_unsafe = params.iter().any(|(_, t)| {
        if t == "*const i8" {
            return true;
        }
        if t.starts_with("Option<unsafe extern") {
            return true; // C 函数指针参数：需要 unsafe
        }
        if let Some(inner) = t.strip_prefix("*mut ") {
            let is_class = class_names.contains(&inner);
            // volatile 限定的类指针参数不能享受 primitive_ret 豁免：仍标记为 unsafe
            if is_class && primitive_ret && !has_volatile_param {
                return false;
            }
            return true;
        }
        false
    }) || ret_type
        .as_deref()
        .is_some_and(|r| r == "*const i8" || r == "*mut i8" || r.starts_with("Option<unsafe extern"));

    // 检测参数或返回类型是否含 C 函数指针，用于生成 cpp2rust-todo[FP] 注释
    let has_fn_ptr_param = fi.params.iter().any(|p| p.type_name.contains("(*)"))
        || fi.return_type.contains("(*)");

    // 构造 C++ 函数签名：只有当参数类型为已知类的指针时才保留参数名，
    // 但 self/this/thiz 等接收者惯用名除外（这些参数在 C 签名中通常省略参数名）
    let param_parts: Vec<String> = fi
        .params
        .iter()
        .map(|p| {
            let ty = normalize_ptr_spacing(clean_type(&p.type_name));
            let is_class_ptr = class_names.iter().any(|cn| p.type_name.contains(cn));
            let is_self_name = matches!(p.name.as_str(), "self" | "this" | "thiz");
            if is_class_ptr && !p.name.is_empty() && p.name != "_" && !is_self_name {
                format!("{} {}", ty, p.name)
            } else {
                ty
            }
        })
        .collect();

    let ret_clean = if fi.return_type.is_empty() || fi.return_type == "void" {
        "void".to_string()
    } else {
        normalize_ptr_spacing(clean_type(&fi.return_type))
    };

    // 无参数时：extern_c → "(void)"，否则 "()"
    let params_str = if param_parts.is_empty() {
        if fi.is_extern_c {
            "void".to_string()
        } else {
            String::new()
        }
    } else {
        param_parts.join(", ")
    };

    let cpp_sig = format!("{} {}({})", ret_clean, fi.name, params_str);

    FnBinding {
        cpp_sig,
        rust_name,
        params,
        ret_type,
        is_unsafe,
        has_fn_ptr_param,
    }
}

// ─────────────────────────────────────────────
//  Shim 分类
// ─────────────────────────────────────────────

#[derive(Debug, PartialEq)]
pub(crate) enum ShimKind {
    Ctor,
    Dtor,
    MethodAccessor,
    Standalone,
    StaticAccessor,
}

fn classify_functions<'a>(
    functions: &[&'a FunctionInfo],
    class_names: &[&str],
) -> Vec<(&'a FunctionInfo, ShimKind)> {
    functions
        .iter()
        .map(|fi| (*fi, classify_fn(fi, class_names)))
        .collect()
}

fn classify_fn(fi: &FunctionInfo, class_names: &[&str]) -> ShimKind {
    let name_lower = fi.name.to_lowercase();

    let ret_is_class_ptr = class_names.iter().any(|cn| {
        let r = &fi.return_type;
        r.contains(&format!("{} *", cn))
            || r.contains(&format!("{}*", cn))
            || r.contains(&format!("{} &", cn))
    });

    let first_param_is_class_ptr = fi
        .params
        .first()
        .map(|p| {
            class_names.iter().any(|cn| {
                let ty = &p.type_name;
                ty.contains(&format!("{} *", cn))
                    || ty.contains(&format!("{}*", cn))
                    || ty.contains(&format!("{} &", cn))
            })
        })
        .unwrap_or(false);

    if ret_is_class_ptr && (name_lower.contains("_new") || name_lower.ends_with("new")) {
        return ShimKind::Ctor;
    }
    if first_param_is_class_ptr
        && (name_lower.contains("_delete")
            || name_lower.ends_with("delete")
            || name_lower.contains("_free")
            || name_lower.ends_with("free")
            || name_lower.contains("_destroy")
            || name_lower.ends_with("destroy")
            || name_lower.contains("_release")
            || name_lower.ends_with("release"))
    {
        return ShimKind::Dtor;
    }
    // 只有当第一个参数是类指针且参数名为约定的 self/this/thiz（表示对象接收者）时，
    // 才归类为 MethodAccessor（会被跳过，不出现在 import_lib/import_class 中）。
    // 若第一个参数名是其他名称（如 other/src/input），则该参数只是普通的类指针参数，
    // 函数应归类为 Standalone，出现在 import_lib 中。
    let first_param_name_is_self = fi
        .params
        .first()
        .map(|p| matches!(p.name.as_str(), "self" | "this" | "thiz"))
        .unwrap_or(false);
    // volatile 限定的指针参数无法作为 hicc 类方法接收者，应归为 Standalone
    let first_param_is_volatile = fi
        .params
        .first()
        .map(|p| p.type_name.split_whitespace().any(|w| w == "volatile"))
        .unwrap_or(false);
    if first_param_is_class_ptr && first_param_name_is_self && !first_param_is_volatile {
        return ShimKind::MethodAccessor;
    }

    let is_static_accessor = class_names.iter().any(|cn| {
        let prefix = format!("{}_", cn.to_lowercase());
        name_lower.starts_with(&prefix)
    }) && !first_param_is_class_ptr;

    if is_static_accessor {
        ShimKind::StaticAccessor
    } else {
        ShimKind::Standalone
    }
}

// ─────────────────────────────────────────────
//  辅助工具
// ─────────────────────────────────────────────

/// 将 C++ 返回类型字符串转换为 Rust `Option<String>`（`None` 表示 void 或空）。
///
/// 统一用于 `build_method_binding` 和 `build_fn_binding`，消除重复判断逻辑。
fn ret_type_from_cpp(s: &str) -> Option<String> {
    if s.is_empty() || s == "void" {
        return None;
    }
    let rt = cpp_to_rust(s);
    if rt.is_empty() {
        None
    } else {
        Some(rt)
    }
}

/// 判断经 `cpp_to_rust` 映射后的 Rust 类型在 FFI 上下文中是否合法可用。
///
/// 合法类型包括：
/// - 空字符串（void 返回值）
/// - Rust 原始类型（i8/u8/i16/u16/i32/u32/i64/u64/f32/f64/bool/isize/usize）
/// - `*const i8` / `*mut i8` / `*const u8` / `*mut u8`（C 字符串或 void 指针）
/// - `*mut T` / `*const T`（T 为 `class_names` 中的已知类或原始类型）
/// - `&T` / `&mut T`（T 为 `class_names` 中的已知类或原始类型）
///
/// 以下情况为非法（会导致生成的 Rust 代码无法编译）：
/// - 含 `::` 的 C++ 命名空间类型（如 `std::string`）
/// - 未声明的 C 类型（如 `FILE`，展开为 `*mut FILE`）
/// - 未知 C++ 类型（如 `MessageMap`、`ValueType`、`SchemaDocument`）
fn is_mappable_rust_type(rust_ty: &str, class_names: &[&str]) -> bool {
    if rust_ty.is_empty() {
        return true; // void 返回值
    }
    // C 函数指针映射结果：`Option<unsafe extern "C" fn(...)>` 始终合法
    if rust_ty.starts_with("Option<unsafe extern") {
        return true;
    }
    // 含 :: 的路径表达式（如 std::string）在 FFI 类型位置非法
    if rust_ty.contains("::") {
        return false;
    }
    const PRIMITIVES: &[&str] = &[
        "i8", "u8", "i16", "u16", "i32", "u32", "i64", "u64", "f32", "f64", "bool", "isize",
        "usize",
    ];
    if PRIMITIVES.contains(&rust_ty) {
        return true;
    }
    // 裸指针：*mut T 或 *const T
    if let Some(inner) = rust_ty
        .strip_prefix("*mut ")
        .or_else(|| rust_ty.strip_prefix("*const "))
    {
        // 字节指针/C 字符串指针始终合法
        if inner == "i8" || inner == "u8" {
            return true;
        }
        // 指向原始类型的指针（如 *mut i32）合法
        if PRIMITIVES.contains(&inner) {
            return true;
        }
        // 指向已知类的指针合法
        return class_names.contains(&inner);
    }
    // 引用：&T 或 &mut T
    if let Some(inner) = rust_ty
        .strip_prefix("&mut ")
        .or_else(|| rust_ty.strip_prefix("&"))
    {
        if PRIMITIVES.contains(&inner) {
            return true;
        }
        return class_names.contains(&inner);
    }
    false
}

/// 从源文件字节数组中读取范围文本
pub(crate) fn extract_range_text(source_bytes: &[u8], start: u32, end: u32) -> String {
    let s = start as usize;
    let e = (end as usize).min(source_bytes.len());
    if s >= e {
        return String::new();
    }
    String::from_utf8_lossy(&source_bytes[s..e]).to_string()
}

/// 从方法定义文本中去除 `ClassName::` 前缀（只替换第一次出现）
fn strip_class_prefix(text: &str, class_name: &str) -> String {
    let prefix = format!("{}::", class_name);
    if let Some(pos) = text.find(&prefix) {
        let mut result = text.to_string();
        result.replace_range(pos..pos + prefix.len(), "");
        result
    } else {
        text.to_string()
    }
}

/// 判断是否为 Rust 关键字（Rust 2021 严格关键字 + 保留关键字）。
///
/// 用于参数名、函数名、方法名的消歧处理，防止生成的 Rust 代码出现关键字冲突。
fn is_rust_keyword(s: &str) -> bool {
    matches!(
        s,
        // 严格关键字（Rust 2021）
        "as" | "async" | "await" | "break" | "const" | "continue" | "crate" | "dyn"
        | "else" | "enum" | "extern" | "false" | "fn" | "for" | "if" | "impl" | "in"
        | "let" | "loop" | "match" | "mod" | "move" | "mut" | "pub" | "ref" | "return"
        | "self" | "Self" | "static" | "struct" | "super" | "trait" | "true" | "type"
        | "union" | "unsafe" | "use" | "where" | "while"
        // 保留关键字
        | "abstract" | "become" | "box" | "do" | "final" | "gen" | "macro" | "override"
        | "priv" | "try" | "typeof" | "unsized" | "virtual" | "yield"
    )
}

/// 参数名称清理（避免 Rust 关键字）
fn sanitize_param_name(name: &str, idx: usize) -> String {
    match name {
        "" | "_" => format!("arg{}", idx),
        _ if is_rust_keyword(name) => format!("{}_", name),
        _ => name.to_string(),
    }
}

/// 函数/方法名清理：先转 snake_case，再对关键字加 `_` 后缀。
///
/// 用于 `build_method_binding` 和 `build_fn_binding` 生成 `rust_name`，
/// 确保结果不与 Rust 关键字冲突。
fn sanitize_fn_name(name: &str) -> String {
    let snake = to_snake_case(name);
    if is_rust_keyword(&snake) {
        format!("{}_", snake)
    } else {
        snake
    }
}

/// 格式化 C++ 参数列表字符串
fn format_params_cpp(params: &[ParamInfo]) -> String {
    params
        .iter()
        .map(|p| {
            let ty = normalize_ptr_spacing(clean_type(&p.type_name));
            if p.name.is_empty() || p.name == "_" {
                ty.to_string()
            } else {
                format!("{} {}", ty, p.name)
            }
        })
        .collect::<Vec<_>>()
        .join(", ")
}

/// 在源文件中查找静态成员变量初始化语句
fn find_static_init(source_bytes: &[u8], class_name: &str, field_name: &str) -> Option<String> {
    let marker = format!("{}::{}", class_name, field_name);
    let text = std::str::from_utf8(source_bytes).ok()?;
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.contains(&marker) && !trimmed.starts_with("//") && trimmed.ends_with(';') {
            return Some(clean_shim_text(trimmed));
        }
    }
    None
}

/// 规范化 C++ 类型中的指针空格：`T *` → `T*`，`const T *` → `const T*`
pub fn normalize_ptr_spacing(ty: &str) -> String {
    let mut result = String::with_capacity(ty.len());
    let mut chars = ty.chars().peekable();
    while let Some(c) = chars.next() {
        // 跳过 '*' 前的空格，避免 byte-level 迭代在 UTF-8 多字节字符时出错
        if c == ' ' && chars.peek() == Some(&'*') {
            continue;
        }
        result.push(c);
    }
    result
}

/// 剥除 C++ 类型的 `volatile` 前缀（volatile 在 C++ 方法签名中不影响 FFI）
fn strip_volatile(ty: &str) -> &str {
    ty.strip_prefix("volatile ").map(str::trim).unwrap_or(ty)
}

/// 剥除方法声明中尾部的 volatile 修饰符（位于 `)` 和 `{` 之间）。
/// 例：`volatile uint32_t readStatus() volatile { … }` → `volatile uint32_t readStatus() { … }`
fn strip_method_volatile_qualifier(text: &str) -> String {
    // 找到第一个 `{`，只处理之前的声明部分
    if let Some(brace) = text.find('{') {
        let decl = &text[..brace];
        // 找到最后一个 `)` 之后的修饰部分（const/volatile/noexcept 等）
        if let Some(last_paren) = decl.rfind(')') {
            let suffix = &decl[last_paren + 1..];
            if !suffix.contains("volatile") {
                return text.to_string(); // 无需修改
            }
            // 只去掉 "volatile" 词，保留其他字符（包括空格）
            let cleaned = suffix.replace(" volatile", "").replace("volatile ", "");
            return format!("{}{}{}", &decl[..=last_paren], cleaned, &text[brace..]);
        }
    }
    text.to_string()
}

/// 读取原始 .cpp 和 .h 文件的 include 行
///
/// 返回 (system_includes, project_header)
/// 顺序规则：
///   1. header-only includes（只在头文件中出现、不在 .cpp 中出现）按头文件顺序排前
///   2. cpp includes（.cpp 中出现的系统 include）按 .cpp 文件中出现的顺序排后
///
/// 头文件扩展名按 `.h` → `.hpp` → `.hxx` 顺序探测，取第一个存在的文件，
/// 以便兼容同时使用 `.hpp`（如 rapidjson、Eigen）的项目。
pub fn read_source_includes(cpp_path: &std::path::Path) -> (Vec<String>, Option<String>) {
    let cpp_content = fs::read_to_string(cpp_path).unwrap_or_default();

    // 按优先级探测对应头文件（.h → .hpp → .hxx）
    let h_content = ["h", "hpp", "hxx"]
        .iter()
        .map(|ext| cpp_path.with_extension(ext))
        .find_map(|p| fs::read_to_string(&p).ok())
        .unwrap_or_default();

    let mut project: Option<String> = None;

    // 收集头文件中的系统 include（保序）
    let h_includes: Vec<String> = h_content
        .lines()
        .filter_map(|line| {
            let t = line.trim();
            let rest = t.strip_prefix("#include ")?;
            let rest = rest.trim();
            if rest.starts_with('<') {
                Some(format!("#include {}", rest))
            } else {
                None
            }
        })
        .collect();
    // 收集 .cpp 中的系统 include（保序）
    let mut cpp_includes: Vec<String> = Vec::new();
    let mut cpp_seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    for line in cpp_content.lines() {
        let t = line.trim();
        if let Some(rest) = t.strip_prefix("#include ") {
            let rest = rest.trim();
            if rest.starts_with('<') {
                let inc = format!("#include {}", rest);
                if cpp_seen.insert(inc.clone()) {
                    cpp_includes.push(inc);
                }
            } else if rest.starts_with('"') {
                let hdr = rest.trim_matches('"');
                if project.is_none() {
                    project = Some(hdr.to_string());
                }
            }
        }
    }
    // 合并：header-only 优先（按头文件顺序），然后 cpp 中的按顺序
    let mut system: Vec<String> = Vec::new();
    let mut seen: std::collections::HashSet<&str> = std::collections::HashSet::new();

    // 1. header-only includes
    for inc in &h_includes {
        if !cpp_seen.contains(inc) && seen.insert(inc.as_str()) {
            system.push(inc.clone());
        }
    }

    // 2. cpp includes（按 cpp 文件顺序，含同时出现在头文件中的）
    for inc in &cpp_includes {
        if seen.insert(inc.as_str()) {
            system.push(inc.clone());
        }
    }

    (system, project)
}

// ─────────────────────────────────────────────
//  关联函数归属
// ─────────────────────────────────────────────

/// 将 LibSpec::fn_bindings 中属于某个类的 ctor/dtor/StaticAccessor 函数
/// 移至对应 ClassSpec::associated_fns，使代码生成器可输出 class body 格式。
///
/// 匹配规则：函数名前缀与类名匹配（如 `counter_new` → 归属 `Counter`）；
/// 仅处理 `ShimKind::Ctor`、`ShimKind::Dtor`、`ShimKind::StaticAccessor`。
/// 不属于任何已知类（或类无对应 ClassSpec）的函数保留在 fn_bindings 中。
fn assign_associated_fns(
    class_specs: &mut [crate::ffi_model::ClassSpec],
    lib_spec: &mut crate::ffi_model::LibSpec,
    functions: &[&FunctionInfo],
    class_names: &[&str],
) {
    // 预先分类所有 shim 函数
    let shims = classify_functions(functions, class_names);

    // 建立 rust_name → ShimKind 映射（去重；同名取第一个）
    let mut kind_map: std::collections::HashMap<String, &ShimKind> =
        std::collections::HashMap::new();
    for (fi, kind) in &shims {
        kind_map.entry(to_snake_case(&fi.name)).or_insert(kind);
    }

    // 预先构建 rust_name → FunctionInfo 映射，避免在循环中重复计算 to_snake_case
    let fn_by_rust_name: std::collections::HashMap<String, &FunctionInfo> = functions
        .iter()
        .map(|fi| (to_snake_case(&fi.name), *fi))
        .collect();

    let mut remaining = Vec::new();
    for fb in lib_spec.fn_bindings.drain(..) {
        let kind = kind_map.get(&fb.rust_name).copied();
        let should_move = matches!(
            kind,
            Some(ShimKind::Ctor | ShimKind::Dtor | ShimKind::StaticAccessor)
        );

        if should_move {
            // 通过函数签名中的类型（返回类型 / 第一个参数类型）确定归属类。
            // 这比名称前缀匹配更可靠，可正确处理 RapidJsonBigIntegerHandle 这类
            // 类名与函数名前缀不一致的情况。
            let matching_function = fn_by_rust_name.get(&fb.rust_name).copied();
            let owning: Option<&str> = matching_function.and_then(|fi| {
                if matches!(kind, Some(ShimKind::Ctor)) {
                    // Ctor：返回类型中含类名（优先最长匹配，避免子串误匹配）
                    class_names
                        .iter()
                        .filter(|cn| fi.return_type.contains(*cn))
                        .max_by_key(|cn| cn.len())
                        .copied()
                } else if matches!(kind, Some(ShimKind::Dtor)) {
                    // Dtor：第一个参数类型含类名（优先最长匹配，避免子串误匹配）
                    fi.params.first().and_then(|p| {
                        class_names
                            .iter()
                            .filter(|cn| p.type_name.contains(*cn))
                            .max_by_key(|cn| cn.len())
                            .copied()
                    })
                } else {
                    // StaticAccessor：退回名称前缀匹配
                    class_names
                        .iter()
                        .filter(|cn| {
                            let prefix = format!("{}_", cn.to_lowercase());
                            fb.rust_name.starts_with(&prefix)
                        })
                        .max_by_key(|cn| cn.len())
                        .copied()
                }
            });

            if let Some(cn) = owning {
                if let Some(cs) = class_specs.iter_mut().find(|c| c.name == cn) {
                    // Dtor：记录 destroy_fn 名称（不放入 associated_fns，dtor 不在 Rust 端显式调用）
                    if matches!(kind, Some(ShimKind::Dtor)) {
                        cs.destroy_fn = Some(fb.rust_name.clone());
                    } else {
                        cs.associated_fns.push(fb);
                    }
                    continue;
                }
            }
        }
        remaining.push(fb);
    }
    lib_spec.fn_bindings = remaining;

    // 确保有 associated_fns 的类在 fwd_decls 中有前向声明
    for cs in class_specs.iter() {
        if !cs.associated_fns.is_empty() {
            let decl = format!("class {};", cs.name);
            if !lib_spec.fwd_decls.contains(&decl) {
                lib_spec.fwd_decls.push(decl);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast_parser::{FunctionInfo, MethodInfo, ParamInfo};

    #[test]
    fn clean_shim_text_removes_struct_prefix() {
        assert_eq!(clean_shim_text("struct Foo* foo_new()"), "Foo* foo_new()");
        assert_eq!(
            clean_shim_text("void foo_delete(struct Foo* self)"),
            "void foo_delete(Foo* self)"
        );
    }

    #[test]
    fn clean_shim_text_removes_class_prefix() {
        assert_eq!(clean_shim_text("class Bar* bar_new()"), "Bar* bar_new()");
        assert_eq!(
            clean_shim_text("void bar_free(class Bar* self)"),
            "void bar_free(Bar* self)"
        );
    }

    #[test]
    fn clean_shim_text_preserves_embedded_keywords() {
        // "struct" 出现在单词中间时不应被去掉
        assert_eq!(clean_shim_text("restructure()"), "restructure()");
        // "class" 出现在单词中间时不应被去掉
        assert_eq!(clean_shim_text("declassify()"), "declassify()");
    }

    // ── 函数指针过滤回归测试 ──────────────────────────────────────

    fn make_fn(name: &str, return_type: &str, param_types: &[&str]) -> FunctionInfo {
        FunctionInfo {
            name: name.to_string(),
            return_type: return_type.to_string(),
            params: param_types
                .iter()
                .enumerate()
                .map(|(i, t)| ParamInfo {
                    name: format!("arg{}", i),
                    type_name: t.to_string(),
                    has_default: false,
                })
                .collect(),
            is_inline: false,
            is_variadic: false,
            is_extern_c: true,
            friend_of: None,
            body_offset: None,
            is_from_current_file: true,
        }
    }

    fn make_method(name: &str, return_type: &str, param_types: &[&str]) -> MethodInfo {
        MethodInfo {
            name: name.to_string(),
            return_type: return_type.to_string(),
            params: param_types
                .iter()
                .enumerate()
                .map(|(i, t)| ParamInfo {
                    name: format!("arg{}", i),
                    type_name: t.to_string(),
                    has_default: false,
                })
                .collect(),
            is_const: false,
            is_volatile: false,
            is_virtual: false,
            is_pure_virtual: false,
            is_static: false,
            is_constructor: false,
            is_destructor: false,
            is_inline: true,
            accessibility: "public".to_string(),
            body_offset: None,
            is_override: false,
            is_default: false,
        }
    }

    /// 返回类型为 C 函数指针的函数现在应映射为 Option<unsafe extern "C" fn(...)>，出现在 fn_bindings 中
    #[test]
    fn build_lib_spec_maps_fn_ptr_return_type() {
        let fi = make_fn("get_callback", "int (*)(int)", &[]);
        let funcs = vec![&fi];
        let spec = build_lib_spec(&funcs, "test", &[]);
        assert_eq!(
            spec.fn_bindings.len(),
            1,
            "返回 C 函数指针的函数应生成绑定，但未出现在 fn_bindings 中"
        );
        let fb = &spec.fn_bindings[0];
        assert!(
            fb.ret_type.as_deref().unwrap_or("").starts_with("Option<unsafe extern"),
            "返回类型应映射为 Option<unsafe extern \"C\" fn(...)>，实际：{:?}",
            fb.ret_type
        );
        assert!(fb.has_fn_ptr_param, "has_fn_ptr_param 应为 true");
    }

    /// 返回类型为 C++ 成员函数指针的函数不应出现在 import_lib! 中
    #[test]
    fn build_lib_spec_filters_member_fn_ptr_return_type() {
        let fi = make_fn("get_method_ptr", "int (Cls::*)(int) const", &[]);
        let funcs = vec![&fi];
        let spec = build_lib_spec(&funcs, "test", &[]);
        assert!(
            spec.fn_bindings.is_empty(),
            "返回 C++ 成员函数指针的函数应被过滤，但仍出现在 fn_bindings 中"
        );
    }

    /// 参数含 C 函数指针的方法现在应生成 MethodBinding（不再跳过）
    #[test]
    fn build_method_binding_maps_fn_ptr_param() {
        let m = make_method("set_handler", "void", &["int (*)(int)"]);
        let binding = build_method_binding(&m);
        assert!(
            binding.is_some(),
            "含 C 函数指针参数的方法应生成绑定，但返回了 None"
        );
        let mb = binding.unwrap();
        assert!(mb.has_fn_ptr_param, "has_fn_ptr_param 应为 true");
        assert!(
            mb.params[0].1.starts_with("Option<unsafe extern"),
            "参数类型应映射为 Option<unsafe extern \"C\" fn(...)>，实际：{}",
            mb.params[0].1
        );
    }

    /// 返回类型为 C 函数指针的方法现在应生成 MethodBinding（不再跳过）
    #[test]
    fn build_method_binding_maps_fn_ptr_return_type() {
        let m = make_method("get_handler", "int (*)(int)", &[]);
        let binding = build_method_binding(&m);
        assert!(
            binding.is_some(),
            "返回 C 函数指针的方法应生成绑定，但返回了 None"
        );
        let mb = binding.unwrap();
        assert!(mb.has_fn_ptr_param, "has_fn_ptr_param 应为 true");
        assert!(
            mb.ret_type.as_deref().unwrap_or("").starts_with("Option<unsafe extern"),
            "返回类型应映射为 Option<unsafe extern \"C\" fn(...)>，实际：{:?}",
            mb.ret_type
        );
    }

    /// 返回类型为 C++ 成员函数指针的方法不应出现在 import_class! 中
    #[test]
    fn build_method_binding_filters_member_fn_ptr_return_type() {
        let m = make_method("get_method_ptr", "int (Cls::*)()", &[]);
        assert!(
            build_method_binding(&m).is_none(),
            "返回 C++ 成员函数指针的方法应返回 None，但未被过滤"
        );
    }

    /// 参数含 C 函数指针的函数应出现在 fn_bindings 中，且标记 is_unsafe 和 has_fn_ptr_param
    #[test]
    fn build_lib_spec_maps_fn_ptr_param() {
        let fi = make_fn("apply_op", "int", &["int", "int (*)(int, int)"]);
        let funcs = vec![&fi];
        let spec = build_lib_spec(&funcs, "test", &[]);
        assert_eq!(
            spec.fn_bindings.len(),
            1,
            "含 C 函数指针参数的函数应生成绑定，但未出现在 fn_bindings 中"
        );
        let fb = &spec.fn_bindings[0];
        assert!(fb.is_unsafe, "含函数指针参数的函数应标记 is_unsafe");
        assert!(fb.has_fn_ptr_param, "has_fn_ptr_param 应为 true");
        // 第三个参数类型应为 Option<unsafe extern "C" fn(...)>
        assert!(
            fb.params[1].1.starts_with("Option<unsafe extern"),
            "第二个参数类型应映射为 Option<unsafe extern \"C\" fn(...)>，实际：{}",
            fb.params[1].1
        );
    }

    /// C++ 成员函数指针参数应继续被过滤（不在 fn_bindings 中）
    #[test]
    fn build_lib_spec_still_filters_member_fn_ptr_param() {
        let fi = make_fn("set_handler", "void", &["int (Cls::*)(int)"]);
        let funcs = vec![&fi];
        let spec = build_lib_spec(&funcs, "test", &[]);
        assert!(
            spec.fn_bindings.is_empty(),
            "含 C++ 成员函数指针参数的函数应被过滤，但仍出现在 fn_bindings 中"
        );
    }

    /// C++ 成员函数指针参数的方法应继续被过滤（返回 None）
    #[test]
    fn build_method_binding_still_filters_member_fn_ptr_param() {
        let m = make_method("set_method_ptr", "void", &["int (Cls::*)(int)"]);
        assert!(
            build_method_binding(&m).is_none(),
            "含 C++ 成员函数指针参数的方法应返回 None，但未被过滤"
        );
    }

    /// 普通方法中 has_fn_ptr_param 应为 false
    #[test]
    fn build_method_binding_has_fn_ptr_param_false_for_normal_method() {
        let m = make_method("get_value", "int", &["int"]);
        let mb = build_method_binding(&m).expect("普通方法应生成绑定");
        assert!(!mb.has_fn_ptr_param, "普通方法的 has_fn_ptr_param 应为 false");
    }

    /// 普通函数（无函数指针）不应被过滤
    #[test]
    fn build_lib_spec_keeps_normal_fn() {
        let fi = make_fn("get_value", "int", &["int"]);
        let funcs = vec![&fi];
        let spec = build_lib_spec(&funcs, "test", &[]);
        assert_eq!(
            spec.fn_bindings.len(),
            1,
            "普通函数不应被过滤，但从 fn_bindings 中消失"
        );
    }

    /// 普通方法（无函数指针）不应被过滤
    #[test]
    fn build_method_binding_keeps_normal_method() {
        let m = make_method("get_value", "int", &["int"]);
        assert!(
            build_method_binding(&m).is_some(),
            "普通方法不应被过滤，但返回了 None"
        );
    }

    // ── is_mappable_rust_type 单元测试 ─────────────────────────────

    #[test]
    fn is_mappable_rust_type_primitives() {
        for ty in &[
            "i8", "u8", "i32", "u32", "i64", "f64", "bool", "isize", "usize",
        ] {
            assert!(is_mappable_rust_type(ty, &[]), "原始类型 {} 应合法", ty);
        }
    }

    #[test]
    fn is_mappable_rust_type_void() {
        assert!(is_mappable_rust_type("", &[]), "空字符串（void）应合法");
    }

    #[test]
    fn is_mappable_rust_type_c_string_ptrs() {
        assert!(is_mappable_rust_type("*const i8", &[]), "*const i8 应合法");
        assert!(is_mappable_rust_type("*mut i8", &[]), "*mut i8 应合法");
        assert!(
            is_mappable_rust_type("*mut u8", &[]),
            "*mut u8（void*）应合法"
        );
        assert!(is_mappable_rust_type("*const u8", &[]), "*const u8 应合法");
    }

    #[test]
    fn is_mappable_rust_type_known_class_ptr() {
        let classes = &["MyClass"];
        assert!(
            is_mappable_rust_type("*mut MyClass", classes),
            "*mut 已知类 应合法"
        );
        assert!(
            is_mappable_rust_type("&mut MyClass", classes),
            "&mut 已知类 应合法"
        );
        assert!(
            is_mappable_rust_type("&MyClass", classes),
            "& 已知类 应合法"
        );
    }

    #[test]
    fn is_mappable_rust_type_unknown_type_is_invalid() {
        assert!(
            !is_mappable_rust_type("FILE", &[]),
            "未知裸类型 FILE 应非法"
        );
        assert!(
            !is_mappable_rust_type("*mut FILE", &[]),
            "*mut FILE（未知类）应非法"
        );
        assert!(
            !is_mappable_rust_type("&mut MessageMap", &[]),
            "&mut 未知类 应非法"
        );
        assert!(
            !is_mappable_rust_type("SchemaDocument", &[]),
            "未知裸类型 SchemaDocument 应非法"
        );
    }

    #[test]
    fn is_mappable_rust_type_namespace_is_invalid() {
        assert!(
            !is_mappable_rust_type("std::string", &[]),
            "含命名空间 std::string 应非法"
        );
    }

    /// 含未知参数类型的函数不应出现在 import_lib! 中
    #[test]
    fn build_lib_spec_filters_unknown_param_type() {
        // FILE 是 C 标准类型，不在 class_names 中，无法映射为合法 Rust 类型
        let fi = make_fn("open_encoded_file", "void", &["const char *", "FILE *"]);
        let funcs = vec![&fi];
        let spec = build_lib_spec(&funcs, "test", &[]);
        assert!(
            spec.fn_bindings.is_empty(),
            "含未知参数类型（FILE *）的函数应被过滤"
        );
    }

    /// 含未知返回类型的函数不应出现在 import_lib! 中
    #[test]
    fn build_lib_spec_filters_unknown_return_type() {
        let fi = make_fn("return_schema_doc", "SchemaDocument", &[]);
        let funcs = vec![&fi];
        let spec = build_lib_spec(&funcs, "test", &[]);
        assert!(
            spec.fn_bindings.is_empty(),
            "含未知返回类型（SchemaDocument）的函数应被过滤"
        );
    }

    /// 含命名空间返回类型的函数不应出现在 import_lib! 中
    #[test]
    fn build_lib_spec_filters_namespace_return_type() {
        let fi = make_fn("get_string", "std::string", &[]);
        let funcs = vec![&fi];
        let spec = build_lib_spec(&funcs, "test", &[]);
        assert!(
            spec.fn_bindings.is_empty(),
            "含命名空间返回类型（std::string）的函数应被过滤"
        );
    }


    #[test]
    fn is_mappable_rust_type_fn_ptr() {
        assert!(
            is_mappable_rust_type(
                r#"Option<unsafe extern "C" fn(i32, i32) -> i32>"#,
                &[]
            ),
            "C 函数指针映射结果应合法"
        );
        assert!(
            is_mappable_rust_type(
                r#"Option<unsafe extern "C" fn(i32)>"#,
                &[]
            ),
            "C 函数指针（无返回类型）映射结果应合法"
        );
    }

    /// 参数为已知类引用的函数应保留在 import_lib! 中
    #[test]
    fn build_lib_spec_keeps_known_class_ref_param() {
        let fi = make_fn("process", "int", &["MyClass &"]);
        let funcs = vec![&fi];
        let spec = build_lib_spec(&funcs, "test", &["MyClass"]);
        assert_eq!(spec.fn_bindings.len(), 1, "参数为已知类引用的函数应保留");
    }
}
