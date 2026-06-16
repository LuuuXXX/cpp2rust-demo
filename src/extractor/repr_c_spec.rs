//! 头文件 POD 结构体规格构建（`#[repr(C)]` 直出）
//!
//! 识别「被 FFI 函数签名引用、但在头文件中有完整字段定义的纯数据结构」（如 SAX 回调表
//! `RapidJsonHandlerCallbacks`），将其从不透明句柄（`import_class!`）路径分流，改以
//! `#[repr(C)]` Rust 结构体输出。否则 hicc 会为其生成 `MethodsType` 特化，与头文件中
//! 真实的 POD 定义冲突，导致 `cpp!` 块编译失败。

use super::type_mapper::{cpp_to_rust, to_snake_case};
use crate::ast_parser::{ClassInfo, FieldInfo};
use crate::ffi_model::ReprCStructSpec;

/// 从 `fwd_decl`（形如 `class Name;`）提取类型名。
fn fwd_decl_name(fwd_decl: &str) -> Option<&str> {
    fwd_decl
        .strip_prefix("class ")
        .and_then(|s| s.strip_suffix(';'))
        .map(str::trim)
}

/// 判断 `ci` 是否为「头文件中完整定义、可安全以 `#[repr(C)]` 输出」的 POD 结构体：
/// - 是 `struct`（而非 `class`）
/// - 来自被 include 的头文件（非当前 `.cpp`，故不会被内联到 `cpp!` 块）
/// - 有字段、无成员方法、无基类（纯数据布局）
fn is_header_pod_struct(ci: &ClassInfo) -> bool {
    ci.is_struct
        && !ci.is_from_current_file
        && !ci.fields.is_empty()
        && ci.methods.is_empty()
        && ci.bases.is_empty()
}

/// 将单个字段映射为 `(rust_name, rust_type)`；无法映射时返回 `None`（调用方据此放弃整个结构体）。
fn map_field(field: &FieldInfo) -> Option<(String, String)> {
    let rust_name = sanitize_field_name(&field.name);
    let rust_type = map_field_type(&field.type_name)?;
    Some((rust_name, rust_type))
}

/// 字段名转 snake_case，并对 Rust 关键字加 `r#` 前缀（保持可编译）。
fn sanitize_field_name(name: &str) -> String {
    let snake = to_snake_case(name);
    if is_rust_keyword(&snake) {
        format!("r#{}", snake)
    } else {
        snake
    }
}

fn is_rust_keyword(s: &str) -> bool {
    matches!(
        s,
        "as" | "break"
            | "const"
            | "continue"
            | "crate"
            | "else"
            | "enum"
            | "extern"
            | "false"
            | "fn"
            | "for"
            | "if"
            | "impl"
            | "in"
            | "let"
            | "loop"
            | "match"
            | "mod"
            | "move"
            | "mut"
            | "pub"
            | "ref"
            | "return"
            | "self"
            | "static"
            | "struct"
            | "super"
            | "trait"
            | "true"
            | "type"
            | "unsafe"
            | "use"
            | "where"
            | "while"
            | "async"
            | "await"
            | "dyn"
    )
}

/// 将字段的 C++ 类型映射为可放入 `#[repr(C)]` 结构体的 Rust 类型。
///
/// 函数指针字段（如 `int (*)(void*)`）映射为 `Option<unsafe extern "C" fn(...)>`：在 C 中
/// 这类字段可为 NULL，`Option<fn>` 的 null 指针优化布局与之一致，且更符合 Rust 习惯。
/// 其余字段走通用 [`cpp_to_rust`]。若映射结果仍含无法在 Rust FFI 中表达的内容
/// （命名空间限定符、空字符串、残留模板括号等）则返回 `None`。
fn map_field_type(cpp: &str) -> Option<String> {
    let mapped = cpp_to_rust(cpp);
    if !is_clean_rust_type(&mapped) {
        return None;
    }
    if cpp.contains("(*)") && mapped.starts_with("unsafe extern \"C\" fn") {
        Some(format!("Option<{}>", mapped))
    } else {
        Some(mapped)
    }
}

/// 校验映射结果是否为「干净」的 Rust 类型：非空，且不含命名空间限定符或未消解的 C++ 记号。
fn is_clean_rust_type(ty: &str) -> bool {
    !ty.is_empty()
        && !ty.contains("::")
        && !ty.contains('<')   // 模板/泛型（fn 指针已在上层用 Option 包裹，不经过这里判 '<'）
        && !ty.contains("struct ")
        && !ty.contains("class ")
        && !ty.contains("(*)")
}

/// 为头文件 POD 结构体构建 `#[repr(C)]` 规格列表，并返回应从 `fwd_decls` 中移除的类型名。
///
/// 仅处理出现在 `fwd_decls`（即确被 FFI 函数签名引用）、且**未**生成 `import_class!` 块
/// （不在 `local_class_names` 中）的头文件 POD 结构体。任一字段无法映射时跳过该结构体，
/// 退回原有的不透明句柄路径，避免生成无法编译的代码。
pub(super) fn build_repr_c_structs(
    classes: &[ClassInfo],
    fwd_decls: &[String],
    local_class_names: &[&str],
) -> (Vec<ReprCStructSpec>, Vec<String>) {
    let mut specs = Vec::new();
    let mut to_remove = Vec::new();

    for fwd_decl in fwd_decls {
        let name = match fwd_decl_name(fwd_decl) {
            Some(n) if !n.is_empty() => n,
            _ => continue,
        };
        if local_class_names.contains(&name) {
            continue;
        }
        let ci = match classes
            .iter()
            .find(|c| c.name == name && is_header_pod_struct(c))
        {
            Some(c) => c,
            None => continue,
        };

        let mapped: Option<Vec<(String, String)>> = ci.fields.iter().map(map_field).collect();
        let fields = match mapped {
            Some(f) => f,
            None => continue, // 任一字段不可映射：保留不透明句柄路径
        };

        specs.push(ReprCStructSpec {
            name: name.to_string(),
            fields,
        });
        to_remove.push(name.to_string());
    }

    (specs, to_remove)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast_parser::FieldInfo;

    fn field(name: &str, ty: &str) -> FieldInfo {
        FieldInfo {
            name: name.to_string(),
            type_name: ty.to_string(),
            is_mutable: false,
            is_static: false,
            accessibility: "public".to_string(),
            field_offset: None,
        }
    }

    fn pod_struct(name: &str, fields: Vec<FieldInfo>) -> ClassInfo {
        ClassInfo {
            name: name.to_string(),
            is_struct: true,
            is_abstract: false,
            template_args: vec![],
            bases: vec![],
            methods: vec![],
            fields,
            is_in_namespace: false,
            simple_name: name.to_string(),
            namespace: None,
            is_from_current_file: false,
        }
    }

    #[test]
    fn maps_callback_table_with_fn_ptr_fields() {
        let ci = pod_struct(
            "RapidJsonHandlerCallbacks",
            vec![
                field("user_data", "void *"),
                field("on_null", "int (*)(void *)"),
                field("on_bool", "int (*)(void *, int)"),
            ],
        );
        let fwd = vec!["class RapidJsonHandlerCallbacks;".to_string()];
        let (specs, removed) = build_repr_c_structs(&[ci], &fwd, &[]);
        assert_eq!(removed, vec!["RapidJsonHandlerCallbacks".to_string()]);
        assert_eq!(specs.len(), 1);
        let s = &specs[0];
        assert_eq!(
            s.fields[0],
            ("user_data".to_string(), "*mut u8".to_string())
        );
        assert_eq!(
            s.fields[1],
            (
                "on_null".to_string(),
                "Option<unsafe extern \"C\" fn(*mut u8) -> i32>".to_string()
            )
        );
        assert_eq!(
            s.fields[2],
            (
                "on_bool".to_string(),
                "Option<unsafe extern \"C\" fn(*mut u8, i32) -> i32>".to_string()
            )
        );
    }

    #[test]
    fn skips_struct_not_in_fwd_decls() {
        let ci = pod_struct("Unreferenced", vec![field("x", "int")]);
        let (specs, removed) = build_repr_c_structs(&[ci], &[], &[]);
        assert!(specs.is_empty());
        assert!(removed.is_empty());
    }

    #[test]
    fn skips_local_handle_with_import_class() {
        let ci = pod_struct("Handle", vec![field("x", "int")]);
        let fwd = vec!["class Handle;".to_string()];
        let (specs, removed) = build_repr_c_structs(&[ci], &fwd, &["Handle"]);
        assert!(specs.is_empty());
        assert!(removed.is_empty());
    }

    #[test]
    fn skips_struct_from_current_file() {
        let mut ci = pod_struct("LocalPod", vec![field("x", "int")]);
        ci.is_from_current_file = true;
        let fwd = vec!["class LocalPod;".to_string()];
        let (specs, removed) = build_repr_c_structs(&[ci], &fwd, &[]);
        assert!(specs.is_empty());
        assert!(removed.is_empty());
    }

    #[test]
    fn skips_struct_with_methods() {
        let mut ci = pod_struct("NotPod", vec![field("x", "int")]);
        ci.methods.push(crate::ast_parser::MethodInfo {
            name: "foo".to_string(),
            return_type: "void".to_string(),
            params: vec![],
            is_const: false,
            is_volatile: false,
            is_virtual: false,
            is_pure_virtual: false,
            is_static: false,
            is_constructor: false,
            is_destructor: false,
            is_inline: false,
            accessibility: "public".to_string(),
            body_offset: None,
            is_override: false,
            is_default: false,
        });
        let fwd = vec!["class NotPod;".to_string()];
        let (specs, removed) = build_repr_c_structs(&[ci], &fwd, &[]);
        assert!(specs.is_empty());
        assert!(removed.is_empty());
    }

    #[test]
    fn skips_struct_with_unmappable_field() {
        let ci = pod_struct("HasNamespacedField", vec![field("v", "std::string")]);
        let fwd = vec!["class HasNamespacedField;".to_string()];
        let (specs, removed) = build_repr_c_structs(&[ci], &fwd, &[]);
        assert!(specs.is_empty());
        assert!(removed.is_empty());
    }
}
