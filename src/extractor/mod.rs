//! C++ 信息提取器（Phase 3）
//!
//! 从 `CppAst` 和原始源信息提取 `FfiSpec` IR，供代码生成器使用。

pub mod type_mapper;

mod class_spec;
mod cpp_block;
mod lib_spec;
mod template_spec;

use crate::ast_parser::{CppAst, FunctionInfo, ParamInfo};
use crate::ffi_model::{ClassSpec, FfiSpec};
use std::fs;
use type_mapper::{clean_type, cpp_to_rust, to_snake_case};

// ─────────────────────────────────────────────
//  公共入口
// ─────────────────────────────────────────────

/// 从 `CppAst` 提取 `FfiSpec`。
///
/// ## 参数说明
///
/// - `ast`：由 [`crate::ast_parser::parse_preprocessed`] 解析 `.cpp2rust` 预处理文件后得到的
///   结构化 AST，包含类/函数/枚举信息及源文件路径。
///
/// - `unit_name`：编译单元名称（如 `"class_basic"`），用于设置 `import_lib!` 的链接库名称。
///
/// - `system_includes`：系统头文件 `#include` 行列表（如 `["#include <cstdint>"]`），
///   由 [`crate::commands::init::read_source_includes`] 从预处理文件的 `# N "filename"` 行标记
///   中扫描系统头路径获得，插入到生成的 `hicc::cpp!` 块顶部。
///
/// - `project_header`：用户项目头文件路径（相对路径，如 `"include/mylib.h"`）；
///   命名空间类模式下生成 `#include "project_header"` 而非内联类体，
///   普通模式下为 `None`。由 [`crate::commands::init::read_source_includes`] 识别项目头并传入。
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
    //
    // 第三条件：非内联函数且有定义体（body_offset）。
    // 背景：在 Windows LLVM 17 上，extern "C" 函数有时以顶层 FunctionDecl 出现（而非
    // LinkageSpec 的子节点），导致 is_extern_c=false；同时 get_file_location().offset 可能
    // 返回原始源文件偏移量而非预处理文件偏移量，导致 is_from_current_file=false。
    // 对于有函数体（is_definition=true）的非内联函数，无论以上两个标志是否正确，
    // 均应将其视为待导出函数。头文件中的函数声明无 body_offset，不受此条影响。
    let eligible_functions: Vec<FunctionInfo> = ast
        .functions
        .iter()
        .filter(|f| {
            f.is_from_current_file || f.is_extern_c || (f.body_offset.is_some() && !f.is_inline)
        })
        .cloned()
        .collect();
    let functions = dedup_functions(&eligible_functions);

    let used_classes = compute_used_classes(&ast.classes, &eligible_functions);
    let namespace_class_mode =
        detect_namespace_mode(has_any_classes, &used_classes, &eligible_functions);

    // ── hicc::cpp! 块内容 ──────────────────────
    let cpp_block_lines = if namespace_class_mode {
        // 命名空间类模式：只生成项目头文件 include，不内联类体
        if let Some(hdr) = project_header {
            vec![format!("#include \"{}\"", hdr)]
        } else {
            Vec::new()
        }
    } else {
        cpp_block::build_cpp_block(
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
        // 导出类名列表：只有在 used_classes 中的类才会真正生成 import_class! 块，
        // 因此只有这些名称才能在方法绑定的类型映射中被视为合法的 FFI 类型。
        let exported_class_names: Vec<&str> = ast
            .classes
            .iter()
            .filter(|c| !c.name.is_empty() && used_classes.contains(&c.name))
            .map(|c| c.name.as_str())
            .collect();
        ast.classes
            .iter()
            .filter(|c| !c.name.is_empty())
            .filter(|c| used_classes.contains(&c.name))
            .map(|ci| {
                class_spec::build_class_spec(ci, &ast.classes, &exported_class_names)
                    .unwrap_or_else(|| ClassSpec {
                        name: ci.name.clone(),
                        ..Default::default()
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
        lib_spec::build_lib_spec(&functions, unit_name, &class_names)
    };

    let mut spec = FfiSpec {
        unit_name: unit_name.to_string(),
        cpp_block_lines,
        class_specs,
        lib_spec,
        ..Default::default()
    };

    // ── 模板类 / 模板函数规格（v6 Phase B）─────────
    // 始终构建（开销极小），但仅在生成器侧 CPP2RUST_GEN_TEMPLATES 开启时输出，
    // 因此默认产物逐字节不变。
    let (template_classes, template_functions) = template_spec::build_template_specs(ast);
    spec.template_classes = template_classes;
    spec.template_functions = template_functions;

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

/// 计算函数签名中引用的类名集合。
///
/// 先检查 extern-C 函数，若无则检查所有符合条件的函数（有些 header 不用 extern "C" 包裹）。
fn compute_used_classes(
    classes: &[crate::ast_parser::ClassInfo],
    eligible_functions: &[FunctionInfo],
) -> std::collections::HashSet<String> {
    let mut set = std::collections::HashSet::new();
    let all_cn: Vec<&str> = classes.iter().map(|c| c.name.as_str()).collect();
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
            if fi.return_type.contains(cn) || fi.params.iter().any(|p| p.type_name.contains(cn)) {
                set.insert(cn.to_string());
            }
        }
    }
    set
}

/// 检测命名空间/opaque 类模式。
///
/// 当且仅当：有类存在 AND 无类名出现在函数签名 AND 至少一个 extern-C 函数的参数/返回类型
/// 包含 `::` 或 `void*`（说明类通过命名空间限定类型或 opaque 指针暴露，hicc 无法处理）。
/// 这影响 cpp! 块内容与 import_class! 生成：
///   043: void* opaque 指针（命名空间类）→ cpp! 只 include 头文件，不生成 import_class!
///   044: example::OperationResult* 命名空间类型指针 → 同样只 include 头文件
///   028: int/double 原始类型（辅助类）→ cpp! 内联类定义，正常生成 import_class!
/// 注意：import_lib! 的生成不受此模式影响，始终由 build_lib_spec 决定（内部有类型过滤）
fn detect_namespace_mode(
    has_any_classes: bool,
    used_classes: &std::collections::HashSet<String>,
    eligible_functions: &[FunctionInfo],
) -> bool {
    has_any_classes
        && used_classes.is_empty()
        && eligible_functions.iter().any(|f| {
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
}

/// 去重：
/// - 以 `(name, param_types_joined)` 为键，对具有相同名称**且**相同参数类型签名的函数去重，
///   保留 score 最高的版本（有 body_offset 且非 extern_c 的版本胜出）。
/// - 具有相同名称但不同参数类型签名的函数（C++ 重载）分别保留，
///   下游代码负责为其生成带数字后缀的不同 Rust 名称。
fn dedup_functions<'a>(functions: &'a [FunctionInfo]) -> Vec<&'a FunctionInfo> {
    // 键：(函数名, 参数类型字符串拼接)
    let mut map: std::collections::HashMap<(&str, String), &'a FunctionInfo> =
        std::collections::HashMap::new();

    for fi in functions {
        let sig_key = fi
            .params
            .iter()
            .map(|p| p.type_name.as_str())
            .collect::<Vec<_>>()
            .join(",");
        let entry = map.entry((fi.name.as_str(), sig_key.clone())).or_insert(fi);
        let new_score = score(fi);
        let old_score = score(entry);
        if new_score > old_score {
            *entry = fi;
        }
    }

    // 按原始顺序输出，同一签名键只出现一次
    let mut result: Vec<&'a FunctionInfo> = Vec::new();
    let mut seen: std::collections::HashSet<(&str, String)> = std::collections::HashSet::new();
    for fi in functions {
        let sig_key = fi
            .params
            .iter()
            .map(|p| p.type_name.as_str())
            .collect::<Vec<_>>()
            .join(",");
        if seen.insert((fi.name.as_str(), sig_key.clone())) {
            if let Some(&best) = map.get(&(fi.name.as_str(), sig_key)) {
                result.push(best);
            }
        }
    }
    result
}

fn score(fi: &FunctionInfo) -> u8 {
    match (fi.body_offset.is_some(), fi.is_extern_c) {
        (true, false) => 3, // 最优：有函数体且非 extern_c
        (true, true) => 2,
        (false, false) => 1,
        (false, true) => 0, // 最差：extern "C" 中的声明
    }
}

// ─────────────────────────────────────────────
//  hicc::cpp! 块构建
// ─────────────────────────────────────────────
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

    // 识别构造函数命名模式（使用原始大小写以正确处理驼峰变体）：
    //   foo_new          — ends_with("_new")
    //   foo_new_variant  — contains("_new_")
    //   foo_newCamelCase — _new 后紧跟大写字母（驼峰，如 foo_newWithSize）
    let name_has_new = fi.name == "new"
        || fi.name.ends_with("_new")
        || fi.name.contains("_new_")
        || fi.name.find("_new").map_or(false, |p| {
            fi.name
                .get(p + 4..)
                .map_or(false, |rest| rest.starts_with(|c: char| c.is_uppercase()))
        });

    if ret_is_class_ptr && name_has_new {
        return ShimKind::Ctor;
    }
    if first_param_is_class_ptr
        && (name_lower.ends_with("_delete")
            || name_lower.ends_with("_deleter")
            || name_lower == "delete"
            || name_lower.ends_with("_free")
            || name_lower == "free"
            || name_lower.ends_with("_destroy")
            || name_lower == "destroy"
            || name_lower.ends_with("_release")
            || name_lower == "release")
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
    // C 函数指针映射结果：`unsafe extern "C" fn(...)` 始终合法
    if rust_ty.starts_with("unsafe extern") {
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
        // 双重指针：*mut *mut T 或 *const *const T（深度限为 2）
        if let Some(inner2) = inner
            .strip_prefix("*mut ")
            .or_else(|| inner.strip_prefix("*const "))
        {
            if inner2 == "i8" || inner2 == "u8" {
                return true;
            }
            if PRIMITIVES.contains(&inner2) {
                return true;
            }
            return class_names.contains(&inner2);
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

/// 从 C++ 类型字符串中删除作为类型限定符出现的 `struct ` 和 `class ` 关键字。
///
/// 不同平台/版本的 libclang 对同一类型的 elaborated type 处理方式不同：
///   - Linux libclang：`const struct MyClass *`（保留 struct 关键字）
///   - Windows LLVM 17：`const MyClass *`（省略 struct 关键字）
///
/// 本函数统一删除这些类型限定符，确保跨平台生成的 cpp_sig 一致。
/// 仅删除 `struct ` / `class `（关键字后跟空格），且前一字符必须为非标识符字符
/// （空格、`(`、`*` 等），以避免误删标识符中包含的 "struct"/"class" 子串（如
/// `my_struct_type` 中的 `struct`）。
fn strip_struct_class_keyword(ty: &str) -> String {
    let bytes = ty.as_bytes();
    let mut result = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        // 判断当前位置是否为单词边界（前一字符不是标识符字符）
        let is_boundary = i == 0 || {
            let prev = bytes[i - 1];
            !prev.is_ascii_alphanumeric() && prev != b'_'
        };
        if is_boundary {
            if bytes[i..].starts_with(b"struct ") {
                i += 7; // "struct ".len()
                continue;
            }
            if bytes[i..].starts_with(b"class ") {
                i += 6; // "class ".len()
                continue;
            }
        }
        result.push(bytes[i]);
        i += 1;
    }
    // SAFETY: 仅删除 ASCII 子序列，剩余字节仍为有效 UTF-8
    String::from_utf8(result).unwrap_or_else(|_| ty.to_string())
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
    use super::class_spec::build_class_spec;
    use super::class_spec::build_method_binding;
    use super::cpp_block::clean_shim_text;
    use super::lib_spec::build_lib_spec;
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

    /// 返回类型为 C 函数指针的函数现在应映射为 `unsafe extern "C" fn(...)`，出现在 fn_bindings 中
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
            fb.ret_type
                .as_deref()
                .unwrap_or("")
                .starts_with("unsafe extern"),
            "返回类型应映射为 unsafe extern \"C\" fn(...)，实际：{:?}",
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
            mb.params[0].1.starts_with("unsafe extern"),
            "参数类型应映射为 unsafe extern \"C\" fn(...)，实际：{}",
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
            mb.ret_type
                .as_deref()
                .unwrap_or("")
                .starts_with("unsafe extern"),
            "返回类型应映射为 unsafe extern \"C\" fn(...)，实际：{:?}",
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
        // 第二个参数类型应为 unsafe extern "C" fn(...)
        assert!(
            fb.params[1].1.starts_with("unsafe extern"),
            "第二个参数类型应映射为 unsafe extern \"C\" fn(...)，实际：{}",
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
        assert!(
            !mb.has_fn_ptr_param,
            "普通方法的 has_fn_ptr_param 应为 false"
        );
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
            is_mappable_rust_type(r#"unsafe extern "C" fn(i32, i32) -> i32"#, &[]),
            "C 函数指针映射结果应合法"
        );
        assert!(
            is_mappable_rust_type(r#"unsafe extern "C" fn(i32)"#, &[]),
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

    // ── build_class_spec 未知类型过滤测试 ────────────────────────────

    use crate::ast_parser::ClassInfo;

    fn make_class(name: &str, methods: Vec<MethodInfo>) -> ClassInfo {
        ClassInfo {
            name: name.to_string(),
            is_struct: false,
            is_abstract: false,
            template_args: vec![],
            bases: vec![],
            methods,
            fields: vec![],
            is_in_namespace: false,
            is_from_current_file: true,
        }
    }

    /// build_class_spec 应过滤含未知参数类型（如 pugixml 的 char_t）的方法
    #[test]
    fn build_class_spec_filters_method_with_unknown_param_type() {
        let m_bad = make_method("write", "void", &["char_t"]);
        let m_good = make_method("get_value", "int", &[]);
        let ci = make_class("MyWriter", vec![m_bad, m_good]);
        let all = vec![ci.clone()];
        let exported = &["MyWriter"];
        let spec = build_class_spec(&ci, &all, exported).expect("类应有方法");
        assert_eq!(spec.methods.len(), 1, "含 char_t 参数的方法应被过滤");
        assert_eq!(spec.methods[0].rust_name, "get_value");
    }

    /// build_class_spec 应过滤含未知返回类型的方法
    #[test]
    fn build_class_spec_filters_method_with_unknown_return_type() {
        let m_bad = make_method("get_context", "xpath_context", &[]);
        let m_good = make_method("get_count", "int", &[]);
        let ci = make_class("XPathExpr", vec![m_bad, m_good]);
        let all = vec![ci.clone()];
        let exported = &["XPathExpr"];
        let spec = build_class_spec(&ci, &all, exported).expect("类应有方法");
        assert_eq!(spec.methods.len(), 1, "返回未知类型的方法应被过滤");
        assert_eq!(spec.methods[0].rust_name, "get_count");
    }

    /// build_class_spec 对已知类型的方法不应过滤
    #[test]
    fn build_class_spec_keeps_method_with_known_types() {
        let m = make_method("process", "int", &["int", "const char *"]);
        let ci = make_class("Processor", vec![m]);
        let all = vec![ci.clone()];
        let exported = &["Processor"];
        let spec = build_class_spec(&ci, &all, exported).expect("类应有方法");
        assert_eq!(spec.methods.len(), 1, "普通方法不应被过滤");
    }

    /// build_class_spec 方法参数为其他已知类的指针时应保留
    #[test]
    fn build_class_spec_keeps_method_with_known_class_ptr_param() {
        let m = make_method("attach", "void", &["Node *"]);
        let node_class = make_class("Node", vec![]);
        let ci = make_class("Document", vec![m]);
        let all = vec![node_class, ci.clone()];
        let exported = &["Node", "Document"];
        let spec = build_class_spec(&ci, &all, exported).expect("类应有方法");
        assert_eq!(spec.methods.len(), 1, "参数为已知类指针的方法不应被过滤");
    }

    // ── classify_fn 单元测试 ─────────────────────────────────────────

    fn make_fn_with_params(name: &str, return_type: &str, params: &[(&str, &str)]) -> FunctionInfo {
        FunctionInfo {
            name: name.to_string(),
            return_type: return_type.to_string(),
            params: params
                .iter()
                .map(|(pname, ptype)| ParamInfo {
                    name: pname.to_string(),
                    type_name: ptype.to_string(),
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

    #[test]
    fn classify_fn_ctor_with_underscore_new() {
        // foo_new 返回类指针 → Ctor
        let fi = make_fn_with_params("foo_new", "Foo *", &[]);
        assert_eq!(classify_fn(&fi, &["Foo"]), ShimKind::Ctor);
    }

    #[test]
    fn classify_fn_ctor_exact_new() {
        // 裸 "new" 返回类指针 → Ctor
        let fi = make_fn_with_params("new", "Foo *", &[]);
        assert_eq!(classify_fn(&fi, &["Foo"]), ShimKind::Ctor);
    }

    #[test]
    fn classify_fn_no_false_ctor_renew() {
        // renew 不含 "_new" 且 name != "new" → 不应被识别为 Ctor
        let fi = make_fn_with_params("renew", "Foo *", &[]);
        // 没有 self 参数，不是 MethodAccessor；类名前缀也不匹配 → Standalone
        assert_ne!(classify_fn(&fi, &["Foo"]), ShimKind::Ctor);
    }

    #[test]
    fn classify_fn_dtor_with_underscore_delete() {
        let fi = make_fn_with_params("foo_delete", "void", &[("self", "Foo *")]);
        assert_eq!(classify_fn(&fi, &["Foo"]), ShimKind::Dtor);
    }

    #[test]
    fn classify_fn_dtor_exact_free() {
        let fi = make_fn_with_params("free", "void", &[("self", "Foo *")]);
        assert_eq!(classify_fn(&fi, &["Foo"]), ShimKind::Dtor);
    }

    #[test]
    fn classify_fn_no_false_dtor_predelete() {
        // predelete 不含 "_delete" 且 name != "delete" → 不应被识别为 Dtor
        let fi = make_fn_with_params("predelete", "void", &[("self", "Foo *")]);
        // 第一个参数名为 self，是 MethodAccessor
        assert_eq!(classify_fn(&fi, &["Foo"]), ShimKind::MethodAccessor);
    }

    // ── 回归测试：Ctor/Dtor 变体命名（修复 ends_with 过窄匹配） ─────────

    #[test]
    fn classify_fn_ctor_snake_variant() {
        // foo_new_from / foo_new_with_size 等 snake_case 变体 → Ctor
        let fi = make_fn_with_params("int_array5_new_from", "IntArray5 *", &[]);
        assert_eq!(classify_fn(&fi, &["IntArray5"]), ShimKind::Ctor);
    }

    #[test]
    fn classify_fn_ctor_camel_variant() {
        // foo_newCopy / foo_newWithSize 等驼峰变体（_new 后紧跟大写字母）→ Ctor
        let fi = make_fn_with_params("buffer_newCopy", "Buffer *", &[]);
        assert_eq!(classify_fn(&fi, &["Buffer"]), ShimKind::Ctor);
    }

    #[test]
    fn classify_fn_ctor_camel_with_size() {
        let fi = make_fn_with_params("buffer_newWithSize", "Buffer *", &[]);
        assert_eq!(classify_fn(&fi, &["Buffer"]), ShimKind::Ctor);
    }

    #[test]
    fn classify_fn_no_false_ctor_newest() {
        // get_newest_item 的 _new 后跟小写字母（非 _ / 大写）→ 不是 Ctor
        let fi = make_fn_with_params("get_newest_item", "Foo *", &[]);
        assert_ne!(classify_fn(&fi, &["Foo"]), ShimKind::Ctor);
    }

    #[test]
    fn classify_fn_dtor_deleter_suffix() {
        // refcounted_file_deleter 等以 _deleter 结尾的函数 → Dtor
        let fi = make_fn_with_params(
            "refcounted_file_deleter",
            "void",
            &[("self", "FileHandle *")],
        );
        assert_eq!(classify_fn(&fi, &["FileHandle"]), ShimKind::Dtor);
    }

    #[test]
    fn classify_fn_method_accessor_self_param() {
        // 第一参数名为 self，类型为类指针 → MethodAccessor
        let fi = make_fn_with_params("foo_get_value", "int", &[("self", "Foo *")]);
        assert_eq!(classify_fn(&fi, &["Foo"]), ShimKind::MethodAccessor);
    }

    #[test]
    fn classify_fn_method_accessor_this_param() {
        let fi = make_fn_with_params("foo_compute", "int", &[("this", "Foo *")]);
        assert_eq!(classify_fn(&fi, &["Foo"]), ShimKind::MethodAccessor);
    }

    #[test]
    fn classify_fn_static_accessor() {
        // 函数名以类名小写前缀开头，且无 self 类指针参数 → StaticAccessor
        let fi = make_fn_with_params("foo_version", "int", &[]);
        assert_eq!(classify_fn(&fi, &["Foo"]), ShimKind::StaticAccessor);
    }

    #[test]
    fn classify_fn_standalone() {
        // 不符合任何特定模式 → Standalone
        let fi = make_fn_with_params("utility_helper", "int", &[]);
        assert_eq!(classify_fn(&fi, &["Foo"]), ShimKind::Standalone);
    }

    // ── dedup_functions 单元测试 ──────────────────────────────────────

    fn make_fn_scored(
        name: &str,
        body_offset: Option<(u32, u32)>,
        is_extern_c: bool,
    ) -> FunctionInfo {
        FunctionInfo {
            name: name.to_string(),
            return_type: "void".to_string(),
            params: vec![],
            is_inline: false,
            is_variadic: false,
            is_extern_c,
            friend_of: None,
            body_offset,
            is_from_current_file: true,
        }
    }

    #[test]
    fn dedup_functions_keeps_unique() {
        let funcs = vec![
            make_fn_scored("alpha", None, true),
            make_fn_scored("beta", None, true),
        ];
        let result = dedup_functions(&funcs);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn dedup_functions_body_offset_wins() {
        // score(body=Some, extern_c=false)=3 优于 score(body=None, extern_c=true)=0
        let funcs = vec![
            make_fn_scored("foo", None, true),            // score 0
            make_fn_scored("foo", Some((10, 20)), false), // score 3
        ];
        let result = dedup_functions(&funcs);
        assert_eq!(result.len(), 1);
        assert!(result[0].body_offset.is_some(), "有函数体的版本应胜出");
        assert!(!result[0].is_extern_c, "非 extern_c 版本应胜出");
    }

    #[test]
    fn dedup_functions_preserves_original_order() {
        // 去重后按第一次出现顺序排列
        let funcs = vec![
            make_fn_scored("beta", None, true),
            make_fn_scored("alpha", None, true),
            make_fn_scored("beta", Some((5, 10)), false), // beta 的更好版本
        ];
        let result = dedup_functions(&funcs);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].name, "beta");
        assert_eq!(result[1].name, "alpha");
        assert!(
            result[0].body_offset.is_some(),
            "beta 应选取 body_offset 版本"
        );
    }

    // ── T2: dedup_functions 函数重载场景测试 ──────────────────────────────────

    /// 同名但参数类型不同的函数（C++ 重载）应各自保留
    #[test]
    fn dedup_functions_preserves_overloads() {
        // 构造 get_value() 和 get_value(int) 两个重载
        let fi_no_param = make_fn("get_value", "int", &[]);
        let fi_one_param = make_fn("get_value", "int", &["int"]);
        let funcs = vec![fi_no_param, fi_one_param];
        let result = dedup_functions(&funcs);
        // 两个重载应都保留
        assert_eq!(
            result.len(),
            2,
            "同名不同签名的重载函数应各自保留，dedup 后应有 2 条"
        );
    }

    /// 同名且参数完全相同的函数仅保留一条（去重）
    #[test]
    fn dedup_functions_deduplicates_exact_same_sig() {
        let fi1 = make_fn("set_value", "void", &["int"]);
        let fi2 = make_fn("set_value", "void", &["int"]);
        let funcs = vec![fi1, fi2];
        let result = dedup_functions(&funcs);
        assert_eq!(result.len(), 1, "完全相同签名的函数应去重为 1 条");
    }

    /// 重载函数在 build_lib_spec 中应获得不同的 rust_name（通过 _1 后缀区分）
    #[test]
    fn build_lib_spec_overload_gets_suffix() {
        let fi_a = make_fn("get_value", "int", &[]);
        let fi_b = make_fn("get_value", "int", &["int"]);
        let funcs = vec![&fi_a, &fi_b];
        let spec = build_lib_spec(&funcs, "test", &[]);
        // 两个重载函数都应生成绑定
        assert_eq!(spec.fn_bindings.len(), 2, "两个重载函数都应生成绑定");
        // 它们的 rust_name 应不同（第二个加了 _1 后缀）
        let names: Vec<&str> = spec
            .fn_bindings
            .iter()
            .map(|fb| fb.rust_name.as_str())
            .collect();
        assert_ne!(names[0], names[1], "重载函数的 rust_name 应不同");
        // 后缀形如 get_value_1
        assert!(
            names.iter().any(|&n| n == "get_value"),
            "应有 get_value（第一个）"
        );
        assert!(
            names.iter().any(|&n| n == "get_value_1"),
            "应有 get_value_1（第二个重载）"
        );
    }

    // ── T6: is_mappable_rust_type 双重指针测试 ────────────────────────────────

    /// 双重字符指针（char**）应通过合法性检查
    #[test]
    fn is_mappable_rust_type_double_char_ptr() {
        assert!(
            is_mappable_rust_type("*mut *mut i8", &[]),
            "*mut *mut i8 (char**) 应合法"
        );
        assert!(
            is_mappable_rust_type("*mut *const i8", &[]),
            "*mut *const i8 (const char**) 应合法"
        );
        assert!(
            is_mappable_rust_type("*mut *mut u8", &[]),
            "*mut *mut u8 (void**) 应合法"
        );
    }

    /// 双重原始类型指针（int**）应通过合法性检查
    #[test]
    fn is_mappable_rust_type_double_primitive_ptr() {
        assert!(
            is_mappable_rust_type("*mut *mut i32", &[]),
            "*mut *mut i32 (int**) 应合法"
        );
        assert!(
            is_mappable_rust_type("*const *const f64", &[]),
            "*const *const f64 (const double* const*) 应合法"
        );
    }

    /// 三重指针不应通过合法性检查（深度限为 2）
    #[test]
    fn is_mappable_rust_type_triple_ptr_is_invalid() {
        assert!(
            !is_mappable_rust_type("*mut *mut *mut i8", &[]),
            "三重指针应非法（深度超限）"
        );
    }

    // ── T1: collect_namespace 三方库函数渗透回归测试 ──────────────────────────

    /// 模拟三方库命名空间函数场景：
    /// is_from_current_file=false 且 is_extern_c=false 的函数不应出现在 fn_bindings 中。
    ///
    /// 背景：`collect_namespace` 修复前，`is_extern_c=true` 的误标函数会通过
    /// `eligible_functions` 过滤器进入 FFI 绑定；修复后，collector 在 push 前检查
    /// `is_from_current_file`，此测试验证该防线生效。
    #[test]
    fn eligible_functions_excludes_non_current_file_fn() {
        // 模拟一个来自三方头文件的函数（既非当前文件，也非 extern C，无函数体）
        let third_party_fn = FunctionInfo {
            name: "clzll".to_string(),
            return_type: "int".to_string(),
            params: vec![ParamInfo {
                name: "x".to_string(),
                type_name: "unsigned long long".to_string(),
                has_default: false,
            }],
            is_inline: false,
            is_variadic: false,
            is_extern_c: false,
            friend_of: None,
            body_offset: None,
            is_from_current_file: false,
        };
        // 正常的当前文件函数
        let current_fn = make_fn("my_func", "int", &["int"]);

        // 通过 dedup_functions 直接检验 eligible_functions 逻辑：
        // 模拟 extract() 中的过滤器
        let all_fns = vec![third_party_fn.clone(), current_fn.clone()];
        let eligible: Vec<&FunctionInfo> = all_fns
            .iter()
            .filter(|f| {
                f.is_from_current_file || f.is_extern_c || (f.body_offset.is_some() && !f.is_inline)
            })
            .collect();

        // clzll 应被过滤掉
        assert!(
            !eligible.iter().any(|f| f.name == "clzll"),
            "来自三方库的命名空间函数不应出现在 eligible_functions 中"
        );
        // 当前文件函数应保留
        assert!(
            eligible.iter().any(|f| f.name == "my_func"),
            "当前文件函数应保留在 eligible_functions 中"
        );
    }
}
