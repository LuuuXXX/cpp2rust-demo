//! C++ 信息提取器（Phase 2）
//!
//! 从 `CppAst` 和原始源信息提取 `FfiSpec` IR，供代码生成器使用。

pub mod class_extractor;
pub mod enum_extractor;
pub mod function_extractor;
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
    let has_classes = !ast.classes.is_empty();

    // 去重：对于同名函数，只保留一个（有 body_offset 的优先；否则 is_extern_c=false 优先）
    let functions = dedup_functions(&ast.functions);

    // ── hicc::cpp! 块内容 ──────────────────────
    let cpp_block_lines = build_cpp_block(
        ast,
        &functions,
        &source_bytes,
        system_includes,
        project_header,
        has_classes,
    );

    // ── import_class! 块列表 ──────────────────
    let class_specs = ast
        .classes
        .iter()
        .filter(|c| !c.name.is_empty())
        .filter_map(|ci| build_class_spec(ci, &ast.classes))
        .collect();

    // ── import_lib! 块 ────────────────────────
    let class_names: Vec<&str> = ast.classes.iter().map(|c| c.name.as_str()).collect();
    let lib_spec = build_lib_spec(&functions, unit_name, &class_names);

    FfiSpec {
        unit_name: unit_name.to_string(),
        cpp_block_lines,
        class_specs,
        lib_spec,
    }
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
        (true, false) => 3,  // best: has body, not extern_c
        (true, true) => 2,
        (false, false) => 1,
        (false, true) => 0,  // worst: declaration in extern "C"
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

    // 判断是否使用分离风格（含虚函数的类）
    let use_separate_style = ast.classes.iter().any(|c| c.methods.iter().any(|m| m.is_virtual));

    let class_names: Vec<&str> = ast.classes.iter().map(|c| c.name.as_str()).collect();

    if use_separate_style {
        // 分离风格：先放所有类的声明，再放方法实现
        for ci in &ast.classes {
            emit_class_decl(ci, source_bytes, &mut lines);
            lines.push(String::new());
        }
        // 方法定义（从源文件读取）
        for ci in &ast.classes {
            for method in &ci.methods {
                if let Some((start, end)) = method.body_offset {
                    let text = extract_range_text(source_bytes, start, end);
                    let text = text.trim();
                    if !text.is_empty() {
                        for line in text.lines() {
                            lines.push(line.to_string());
                        }
                        lines.push(String::new());
                    }
                }
            }
        }
    } else {
        // 内联风格：类定义含内联方法体
        for ci in &ast.classes {
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

    // Ctor/dtor shim 函数（含静态访问器）
    let shim_fns = classify_functions(functions, &class_names);
    for (fn_info, shim_kind) in &shim_fns {
        if matches!(shim_kind, ShimKind::Ctor | ShimKind::Dtor | ShimKind::StaticAccessor) {
            if let Some((start, end)) = fn_info.body_offset {
                let raw = extract_range_text(source_bytes, start, end);
                let cleaned = clean_shim_text(&raw);
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

    let pub_methods: Vec<&MethodInfo> = ci
        .methods
        .iter()
        .filter(|m| m.accessibility == "public")
        .collect();

    if !pub_methods.is_empty() {
        let has_non_pub = !ci.fields.is_empty()
            && ci.fields.iter().any(|f| f.accessibility != "public");
        if has_non_pub || (!ci.fields.is_empty() && !ci.is_struct) {
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
        let group: Vec<&FieldInfo> = ci.fields.iter().filter(|f| f.accessibility == acc).collect();
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
            // 去掉 struct 前缀，加上 static 关键字（如果需要）
            let cleaned = clean_shim_text(trimmed);
            return format!("{}{};", static_kw, cleaned.trim());
        }
    }
    // 回退：构造
    let ty = clean_type(&field.type_name);
    format!("{}{} {};", static_kw, ty, field.name)
}

/// 输出方法声明（分离风格，无方法体）
fn emit_method_decls(ci: &ClassInfo, lines: &mut Vec<String>) {
    let pub_methods: Vec<&MethodInfo> =
        ci.methods.iter().filter(|m| m.accessibility == "public").collect();
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
    let override_sfx = if m.is_override { " override" } else { "" };
    let pure_sfx = if m.is_pure_virtual && !m.is_override { " = 0" } else { "" };

    format!("{}{}{}({}){}{}{}", qualifier, ret, name, params, const_sfx, override_sfx, pure_sfx)
}

/// 构建单行内联方法（内联风格）
fn build_inline_method_line(m: &MethodInfo, source_bytes: &[u8], class_name: &str) -> String {
    if let Some((start, end)) = m.body_offset {
        let raw_text = extract_range_text(source_bytes, start, end);
        let stripped = strip_class_prefix(raw_text.trim(), class_name);
        return stripped;
    }

    // 没有 body_offset → 生成 `= default;` 或 `{}`
    let decl = build_method_decl(m);
    if m.is_constructor || m.is_destructor {
        format!("{} = default;", decl)
    } else if m.is_pure_virtual {
        format!("{};", decl)
    } else {
        format!("{} {{}}", decl)
    }
}

/// 清理 shim 函数文本中的 `struct ClassName*` → `ClassName*`
fn clean_shim_text(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let bytes = text.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if i + 7 <= bytes.len() && &bytes[i..i + 7] == b"struct " {
            let prev_ok = i == 0
                || bytes[i - 1] == b' '
                || bytes[i - 1] == b'\n'
                || bytes[i - 1] == b'\t'
                || bytes[i - 1] == b'('
                || bytes[i - 1] == b',';
            if prev_ok {
                i += 7;
                continue;
            }
        }
        result.push(bytes[i] as char);
        i += 1;
    }
    result
}

// ─────────────────────────────────────────────
//  import_class! 块
// ─────────────────────────────────────────────

fn build_class_spec(ci: &ClassInfo, all_classes: &[ClassInfo]) -> Option<ClassSpec> {
    // 收集本类的 public 非 ctor/dtor 方法
    let own_methods: Vec<&MethodInfo> = ci
        .methods
        .iter()
        .filter(|m| !m.is_constructor && !m.is_destructor && m.accessibility == "public" && !m.is_static)
        .collect();

    // 收集所有基类的 public 方法（递归，保持顺序）
    let inherited = collect_inherited_methods(ci, all_classes);

    // 合并：继承方法 + 本类覆盖/新增方法
    // 规则：如果本类有同名方法（override），用本类的；否则用继承的
    let own_names: std::collections::HashSet<&str> =
        own_methods.iter().map(|m| m.name.as_str()).collect();

    let mut methods: Vec<MethodBinding> = Vec::new();

    // 先放继承来的（本类未覆盖的）
    for im in &inherited {
        if !own_names.contains(im.name.as_str()) {
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

    Some(ClassSpec { name: ci.name.clone(), methods })
}

/// 递归收集所有基类的 public 非 ctor/dtor 方法（不含静态方法）
fn collect_inherited_methods<'a>(ci: &ClassInfo, all_classes: &'a [ClassInfo]) -> Vec<&'a MethodInfo> {
    let mut result: Vec<&'a MethodInfo> = Vec::new();
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();

    for base in &ci.bases {
        let base_name = clean_type(&base.name).to_string();
        if let Some(base_ci) = all_classes.iter().find(|c| c.name == base_name) {
            // 先递归收集基类的基类
            let grand_inherited = collect_inherited_methods(base_ci, all_classes);
            for m in grand_inherited {
                if !seen.contains(&m.name) {
                    seen.insert(m.name.clone());
                    result.push(m);
                }
            }
            // 再收集本基类的方法
            for m in base_ci.methods.iter().filter(|m| {
                !m.is_constructor
                    && !m.is_destructor
                    && m.accessibility == "public"
                    && !m.is_static
            }) {
                if !seen.contains(&m.name) {
                    seen.insert(m.name.clone());
                    result.push(m);
                }
            }
        }
    }
    result
}

fn build_method_binding(m: &MethodInfo) -> Option<MethodBinding> {
    let rust_name = to_snake_case(&m.name);
    let self_kind = if m.is_const { SelfKind::Ref } else { SelfKind::RefMut };

    let params: Vec<(String, String)> = m
        .params
        .iter()
        .map(|p| (sanitize_param_name(&p.name), cpp_to_rust(&p.type_name)))
        .collect();

    let ret_type = if m.return_type.is_empty() || m.return_type == "void" {
        None
    } else {
        Some(cpp_to_rust(&m.return_type))
    };

    // C++ 方法签名：type-only 参数，指针紧贴类型
    let param_types: Vec<String> = m
        .params
        .iter()
        .map(|p| normalize_ptr_spacing(clean_type(&p.type_name)))
        .collect();
    let ret_clean = normalize_ptr_spacing(clean_type(&m.return_type));
    let const_suffix = if m.is_const { " const" } else { "" };
    let cpp_sig = if m.return_type.is_empty() || m.return_type == "void" {
        format!("void {}({}){}", m.name, param_types.join(", "), const_suffix)
    } else {
        format!("{} {}({}){}", ret_clean, m.name, param_types.join(", "), const_suffix)
    };

    Some(MethodBinding { cpp_sig, rust_name, self_kind, params, ret_type })
}

// ─────────────────────────────────────────────
//  import_lib! 块
// ─────────────────────────────────────────────

fn build_lib_spec(functions: &[&FunctionInfo], unit_name: &str, class_names: &[&str]) -> LibSpec {
    let fwd_decls: Vec<String> =
        class_names.iter().map(|n| format!("class {};", n)).collect();

    let shims = classify_functions(functions, class_names);
    let fn_bindings: Vec<FnBinding> = shims
        .iter()
        .filter(|(_, k)| !matches!(k, ShimKind::MethodAccessor))
        .map(|(fi, _)| build_fn_binding(fi))
        .collect();

    LibSpec { link_name: unit_name.to_string(), fwd_decls, fn_bindings }
}

fn build_fn_binding(fi: &FunctionInfo) -> FnBinding {
    let rust_name = to_snake_case(&fi.name);
    let params: Vec<(String, String)> = fi
        .params
        .iter()
        .map(|p| (sanitize_param_name(&p.name), cpp_to_rust(&p.type_name)))
        .collect();

    let ret_type = if fi.return_type.is_empty() || fi.return_type == "void" {
        None
    } else {
        let rt = cpp_to_rust(&fi.return_type);
        if rt.is_empty() { None } else { Some(rt) }
    };

    // unsafe: 参数中有 *mut T 类型
    let is_unsafe = params.iter().any(|(_, t)| t.starts_with("*mut "));

    // 构造 C++ 函数签名（类型后紧贴 *，保留参数名）
    let param_parts: Vec<String> = fi
        .params
        .iter()
        .map(|p| {
            let ty = normalize_ptr_spacing(clean_type(&p.type_name));
            if !p.name.is_empty() && p.name != "_" {
                format!("{} {}", ty, p.name)
            } else {
                ty.to_string()
            }
        })
        .collect();

    let ret_clean = if fi.return_type.is_empty() || fi.return_type == "void" {
        "void".to_string()
    } else {
        normalize_ptr_spacing(clean_type(&fi.return_type)).to_string()
    };

    // 无参数时：extern_c → "(void)"，否则 "()"
    let params_str = if param_parts.is_empty() {
        if fi.is_extern_c { "void".to_string() } else { String::new() }
    } else {
        param_parts.join(", ")
    };

    let cpp_sig = format!("{} {}({})", ret_clean, fi.name, params_str);

    FnBinding { cpp_sig, rust_name, params, ret_type, is_unsafe }
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
    functions.iter().map(|fi| (*fi, classify_fn(fi, class_names))).collect()
}

fn classify_fn(fi: &FunctionInfo, class_names: &[&str]) -> ShimKind {
    let name_lower = fi.name.to_lowercase();

    let ret_is_class_ptr = class_names.iter().any(|cn| {
        let r = &fi.return_type;
        r.contains(&format!("{} *", cn))
            || r.contains(&format!("{}*", cn))
            || r.contains(&format!("{} &", cn))
    });

    let first_param_is_class_ptr = fi.params.first().map(|p| {
        class_names.iter().any(|cn| {
            let ty = &p.type_name;
            ty.contains(&format!("{} *", cn))
                || ty.contains(&format!("{}*", cn))
                || ty.contains(&format!("{} &", cn))
        })
    }).unwrap_or(false);

    if ret_is_class_ptr && (name_lower.contains("_new") || name_lower.ends_with("new")) {
        return ShimKind::Ctor;
    }
    if first_param_is_class_ptr
        && (name_lower.contains("_delete") || name_lower.ends_with("delete"))
    {
        return ShimKind::Dtor;
    }
    if first_param_is_class_ptr {
        return ShimKind::MethodAccessor;
    }

    let is_static_accessor = class_names.iter().any(|cn| {
        let prefix = format!("{}_", cn.to_lowercase());
        name_lower.starts_with(&prefix)
    }) && !first_param_is_class_ptr;

    if is_static_accessor { ShimKind::StaticAccessor } else { ShimKind::Standalone }
}

// ─────────────────────────────────────────────
//  辅助工具
// ─────────────────────────────────────────────

/// 从源文件字节数组中读取范围文本
pub(crate) fn extract_range_text(source_bytes: &[u8], start: u32, end: u32) -> String {
    let s = start as usize;
    let e = (end as usize).min(source_bytes.len());
    if s >= e { return String::new(); }
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

/// 参数名称清理（避免 Rust 关键字）
fn sanitize_param_name(name: &str) -> String {
    match name {
        "self" => "self_".to_string(),
        "type" => "type_".to_string(),
        "fn" => "fn_".to_string(),
        "loop" => "loop_".to_string(),
        "move" => "move_".to_string(),
        "" | "_" => "arg".to_string(),
        _ => name.to_string(),
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
    let bytes = ty.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b' ' && i + 1 < bytes.len() && bytes[i + 1] == b'*' {
            i += 1;
            continue;
        }
        result.push(bytes[i] as char);
        i += 1;
    }
    result
}

/// 读取原始 .cpp 和 .h 文件的 include 行
///
/// 返回 (system_includes, project_header)
/// system_includes 顺序：header 优先，然后 cpp 中新增
pub fn read_source_includes(cpp_path: &std::path::Path) -> (Vec<String>, Option<String>) {
    let cpp_content = fs::read_to_string(cpp_path).unwrap_or_default();

    // 尝试找到对应的 .h 文件
    let h_path = cpp_path.with_extension("h");
    let h_content = fs::read_to_string(&h_path).unwrap_or_default();

    let mut system: Vec<String> = Vec::new();
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut project: Option<String> = None;

    // 读取 .h 文件中的系统 include（优先放入）
    let mut in_cpp_section = false;
    for line in h_content.lines() {
        let t = line.trim();
        // 粗略检测 #ifdef __cplusplus 之后的部分（C++ 专用区域）
        if t == "#ifdef __cplusplus" || t.starts_with("#if defined(__cplusplus") {
            in_cpp_section = true;
        } else if t == "#endif" {
            // 不重置 in_cpp_section，因为有多个 #endif
        }
        if let Some(rest) = t.strip_prefix("#include ") {
            let rest = rest.trim();
            if rest.starts_with('<') {
                let inc = format!("#include {}", rest);
                if seen.insert(inc.clone()) {
                    system.push(inc);
                }
            }
        }
    }

    // 读取 .cpp 文件中的 include
    for line in cpp_content.lines() {
        let t = line.trim();
        if let Some(rest) = t.strip_prefix("#include ") {
            let rest = rest.trim();
            if rest.starts_with('<') {
                let inc = format!("#include {}", rest);
                if seen.insert(inc.clone()) {
                    system.push(inc);
                }
            } else if rest.starts_with('"') {
                let hdr = rest.trim_matches('"');
                if project.is_none() {
                    project = Some(hdr.to_string());
                }
            }
        }
    }

    (system, project)
}
