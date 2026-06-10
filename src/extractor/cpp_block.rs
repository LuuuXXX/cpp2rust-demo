//! cpp! ブロック構築（Phase 3）
//!
//! ClassInfo と FunctionInfo から hicc の `cpp! { ... }` ブロックのソース行を生成する。

use super::type_mapper::clean_type;
use super::{classify_functions, extract_range_text, format_params_cpp, normalize_ptr_spacing, ShimKind};
use crate::ast_parser::{ClassInfo, CppAst, FieldInfo, FunctionInfo, MethodInfo};

pub(super) fn build_cpp_block(
    ast: &CppAst,
    functions: &[&FunctionInfo],
    source_bytes: &[u8],
    system_includes: &[String],
    project_header: Option<&str>,
    has_classes: bool,
    extra_local_includes: &[String],
) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();

    if !has_classes {
        // 函数-only（或仅含模板类）：放项目头文件 include + 额外本地头文件 + using namespace 指令
        if let Some(hdr) = project_header {
            lines.push(format!("#include \"{}\"", hdr));
        }
        for hdr in extra_local_includes {
            lines.push(format!("#include \"{}\"", hdr));
        }
        for ns in &ast.using_namespaces {
            lines.push(ns.clone());
        }
        while lines.last().map(|l| l.is_empty()).unwrap_or(false) {
            lines.pop();
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
    // - 项目头文件已提供完整类型定义，不重复输出 typedef/枚举/类体/shim 函数。
    //   若继续输出 typedef，可能引入仅在原 .cpp 额外 include（如 internal_handles.h）
    //   中才可用的 C++ 模板类型，导致 hicc::cpp! 块缺少对应头文件而报编译错误。
    let use_project_header = local_classes.is_empty() && project_header.is_some();
    if use_project_header {
        if let Some(hdr) = project_header {
            lines.push(format!("#include \"{}\"", hdr));
        }
        // 包含额外本地头文件（如 internal_handles.h），为 import_class! 提供完整类型定义。
        // 这些头文件自带 #pragma once 保护，与项目头文件的前向声明不会冲突。
        for hdr in extra_local_includes {
            lines.push(format!("#include \"{}\"", hdr));
        }
        // .cpp 文件中的 using namespace 指令（如 using namespace rapidjson;）。
        // 对于将结构体定义在内部头文件（如 internal_handles.h）中的文件，
        // 这些指令可能已由 extra_local_includes 间接引入，但显式输出无害（C++ 允许重复）。
        for ns in &ast.using_namespaces {
            lines.push(ns.clone());
        }
        // 去掉末尾多余空行后直接返回：不输出 typedef/类定义/shim 函数
        while lines.last().map(|l| l.is_empty()).unwrap_or(false) {
            lines.pop();
        }
        return lines;
    }

    // 以下仅在有 local_classes 需要内联时执行 ─────────────────────────────

    // 注入额外的本地头文件（如 rapidjson/writer.h），定义内联类体所需的 C++ 类型
    // （例如 Writer<StringBuffer>、PrettyWriter<StringBuffer>）。
    // 不含 project_header 本身，因为其中的不透明 typedef 和 extern "C" 声明
    // 与内联类体共存时不会产生歧义（C++ 允许 typedef struct X X 后再定义 struct X）。
    for hdr in extra_local_includes {
        lines.push(format!("#include \"{}\"", hdr));
    }
    if !extra_local_includes.is_empty() {
        lines.push(String::new());
    }

    // using namespace 指令（来自 .cpp 文件自身，在 #include 之后、类定义之前）
    for ns in &ast.using_namespaces {
        lines.push(ns.clone());
    }
    if !ast.using_namespaces.is_empty() {
        lines.push(String::new());
    }

    // 枚举定义（在类定义之前；仅在内联模式下输出，避免与项目头文件重复定义）
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

    // typedef 定义（在类定义之前；仅在内联模式下输出）
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

    // Ctor/dtor/standalone shim 函数（含静态访问器）内联到 cpp! 块中。
    // 此处只有 local_classes 非空时才会执行（use_project_header=true 时已提前返回），
    // 因此不存在与项目 .cpp 中已有实现重复定义的问题。
    let mut shim_fns = classify_functions(functions, &class_names);
    // 按函数体在源文件中的字节偏移升序排序，确保静态辅助函数在调用它的外部函数之前输出，
    // 避免因 ast.functions 中 extern "C" 声明排在前面而造成前向引用编译错误。
    shim_fns.sort_by_key(|(fi, _)| fi.body_offset.map(|(s, _)| s).unwrap_or(u32::MAX));
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
        // override 方法：不加 virtual 前缀
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

/// 过滤预处理器行号标记，如 `# 26 "file.cpp" 3 4`
pub(super) fn strip_preprocessor_markers(text: &str) -> String {
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
pub(super) fn clean_shim_text(text: &str) -> String {
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
