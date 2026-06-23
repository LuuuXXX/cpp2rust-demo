//! C++ 信息提取器（Phase 3）
//!
//! 从 `CppAst` 和原始源信息提取 `FfiSpec` IR，供代码生成器使用。

pub mod type_mapper;

mod class_spec;
mod cpp_block;
mod dynamic_cast_spec;
mod hicc_direct;
mod ident_util;
mod lib_spec;
mod proxy_spec;
mod repr_c_spec;
mod shim_classifier;
mod source_reader;
mod template_spec;

use crate::ast_parser::{CppAst, FunctionInfo};
use crate::ffi_model::{ClassSpec, FfiSpec};
use std::fs;
use type_mapper::cpp_to_rust;

// 从子模块重导出公用符号，使 sub-module 的 `super::fn_name` 路径继续可用
pub use source_reader::read_source_includes;
use ident_util::{format_params_cpp, is_rust_keyword, sanitize_fn_name, sanitize_param_name};
use shim_classifier::{assign_associated_fns, classify_fn, classify_functions, ShimKind};
pub use type_mapper::{
    extract_range_text, normalize_ptr_spacing, strip_struct_class_keyword, strip_volatile,
};

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
    let source_bytes = match fs::read(&ast.file) {
        Ok(bytes) => bytes,
        Err(e) => {
            eprintln!(
                "warning: cpp2rust: failed to read source file '{}': {}",
                ast.file.display(),
                e
            );
            Vec::new()
        }
    };
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

    // ── hicc 直出（idiomatic 命名空间类）模式 ──────────────
    // 去 shim 核心：真实命名空间类 + make_unique 工厂直出，替代 extern-C opaque 桥接。
    // 仅当不存在任何 extern "C" 函数、且存在带公有构造的命名空间类时启用；
    // 现有 extern-C 示例不受影响，仍走下方旧路径。
    if hicc_direct::detect_idiomatic_mode(ast) {
        let cpp_block_lines = if let Some(hdr) = project_header {
            vec![format!("#include \"{}\"", hdr)]
        } else {
            // 无项目引号 include（如 header-only 库以系统 include 引入头文件时），
            // 将捕获到的系统 include 注入 hicc::cpp! 块，确保库类型在 hicc 编译
            // 生成的 C++ wrapper 中可见（如 magic_enum、toml++、fmtlib 等）。
            // 额外追加当前文件定义的命名空间类的内联声明，使 hicc 能解析
            // import_class! 绑定中引用的用户类类型，确保 cargo check 通过。
            let mut lines = system_includes.to_vec();
            lines.extend(hicc_direct::emit_current_file_class_decls(ast));
            lines
        };
        let class_specs = hicc_direct::build_hicc_direct_specs(ast);
        // 绑定命名空间自由函数（排除仅用于产生链接符号的 `<unit>_anchor` 锚点函数）
        let free_fns: Vec<&FunctionInfo> = functions
            .iter()
            .copied()
            .filter(|f| !f.name.ends_with("_anchor"))
            .collect();
        let class_names: Vec<&str> = ast.classes.iter().map(|c| c.name.as_str()).collect();
        let lib_spec = lib_spec::build_lib_spec_namespaced(&free_fns, unit_name, &class_names);
        return FfiSpec {
            unit_name: unit_name.to_string(),
            cpp_block_lines,
            class_specs,
            lib_spec,
            ..Default::default()
        };
    }

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
    // 始终调用 build_lib_spec_namespaced：其内部的 is_mappable_rust_type 过滤器会自动
    // 排除含 `::` 的命名空间类型（如 std::string*、example::OperationResult*），而
    // void* → *mut u8 等可映射类型则正常生成绑定。
    //
    // 命名空间自由函数（如模板示例的 `<unit>_ns::<unit>_anchor()`）必须以 `ns::name`
    // 限定其 C++ 签名：本路径的 hicc `cpp!` 块仅含 `#include`，不带 `using namespace`，
    // 裸函数名会在全局作用域解析失败（实测 024/028 生成产物 cargo build 报
    // “was not declared in this scope”）。限定仅对 `fi.namespace` 为 Some 的函数生效，
    // 旧式 extern-C 全局桥接函数（namespace 为 None）保持裸函数名不变。
    let lib_spec = {
        let class_names: Vec<&str> = ast.classes.iter().map(|c| c.name.as_str()).collect();
        lib_spec::build_lib_spec_namespaced(&functions, unit_name, &class_names)
    };

    let mut spec = FfiSpec {
        unit_name: unit_name.to_string(),
        cpp_block_lines,
        class_specs,
        lib_spec,
        ..Default::default()
    };

    // ── 模板类 / 模板函数规格（v6 Phase B）─────────
    // 构建模板 IR（开销极小），v7 起由生成器默认输出，
    // 不再依赖任何环境变量开关。
    let (template_classes, template_functions) = template_spec::build_template_specs(ast);
    spec.template_classes = template_classes;
    spec.template_functions = template_functions;
    spec.template_instances = template_spec::build_template_instances(ast);
    spec.template_factories =
        template_spec::build_template_factories(ast, &spec.template_instances);

    // ── @make_proxy 代理工厂规格（v6 Phase C）─────────
    // 构建代理工厂 IR（开销极小），v7 起由生成器默认输出，
    // 不再依赖任何环境变量开关。
    spec.proxy_factories = proxy_spec::build_proxy_factories(ast);

    // ── @dynamic_cast 下行转换规格（v6 Phase C（续））─────────
    // 构建 dynamic_cast IR（开销极小），v7 起由生成器默认输出，
    // 不再依赖任何环境变量开关。
    spec.dynamic_casts = dynamic_cast_spec::build_dynamic_casts(ast);

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

    // ── 头文件 POD 结构体 → #[repr(C)] 直出 ──────────────
    // 被 FFI 函数签名引用、但在头文件中有完整字段定义的纯数据结构（如 SAX 回调表
    // RapidJsonHandlerCallbacks）须以 #[repr(C)] Rust 结构体输出，而非不透明 import_class!，
    // 否则会与 hicc 的 MethodsType 特化冲突。命中后将其从 fwd_decls 移除，
    // 使 import_lib! 不再前向声明、跨模块前缀也不再生成不透明句柄。
    {
        let local_class_names: Vec<&str> = spec
            .class_specs
            .iter()
            .filter(|cs| !cs.is_empty())
            .map(|cs| cs.name.as_str())
            .collect();
        let (repr_c_structs, to_remove) = repr_c_spec::build_repr_c_structs(
            &ast.classes,
            &spec.lib_spec.fwd_decls,
            &local_class_names,
        );
        if !to_remove.is_empty() {
            spec.lib_spec.fwd_decls.retain(|decl| {
                let name = decl
                    .strip_prefix("class ")
                    .and_then(|s| s.strip_suffix(';'))
                    .map(str::trim)
                    .unwrap_or("");
                !to_remove.iter().any(|r| r == name)
            });
        }
        spec.repr_c_structs = repr_c_structs;
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
            if type_references_class(&fi.return_type, cn)
                || fi
                    .params
                    .iter()
                    .any(|p| type_references_class(&p.type_name, cn))
            {
                set.insert(cn.to_string());
            }
        }
    }
    set
}

/// 判断类型字符串是否引用了指定类名（词边界匹配，避免 "Foo" 误匹配 "FooBar"）。
///
/// 匹配规则：类名后必须紧跟 `*`、`&`、` `、`>` 或字符串末尾，
/// 且前面必须是非标识符字符（空格、`*`、`<` 或字符串开头）。
fn type_references_class(ty: &str, class_name: &str) -> bool {
    let cn_bytes = class_name.as_bytes();
    let ty_bytes = ty.as_bytes();
    let cn_len = cn_bytes.len();
    let mut i = 0;
    while i + cn_len <= ty_bytes.len() {
        if ty_bytes[i..].starts_with(cn_bytes) {
            let prefix_ok = i == 0 || {
                let prev = ty_bytes[i - 1];
                !prev.is_ascii_alphanumeric() && prev != b'_'
            };
            let suffix_ok = i + cn_len == ty_bytes.len() || {
                let next = ty_bytes[i + cn_len];
                next == b'*' || next == b'&' || next == b' ' || next == b'>'
            };
            if prefix_ok && suffix_ok {
                return true;
            }
        }
        i += 1;
    }
    false
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
                let rt = normalize_ptr_spacing(f.return_type.as_str());
                rt.contains("::")
                    || rt.contains("void*")
                    || f.params.iter().any(|p| {
                        let t = normalize_ptr_spacing(p.type_name.as_str());
                        t.contains("::") || t.contains("void*")
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
    // 预先计算每个函数的签名键，避免在两次循环中重复拼接
    let keyed: Vec<(&FunctionInfo, String)> = functions
        .iter()
        .map(|fi| {
            let sig_key = fi
                .params
                .iter()
                .map(|p| p.type_name.as_str())
                .collect::<Vec<_>>()
                .join(",");
            (fi, sig_key)
        })
        .collect();

    // 键：(函数名, 参数类型字符串拼接)
    let mut map: std::collections::HashMap<(&str, &str), &'a FunctionInfo> =
        std::collections::HashMap::new();

    for (fi, sig_key) in &keyed {
        let entry = map.entry((fi.name.as_str(), sig_key.as_str())).or_insert(fi);
        let new_score = score(fi);
        let old_score = score(entry);
        if new_score > old_score {
            *entry = fi;
        }
    }

    // 按原始顺序输出，同一签名键只出现一次
    let mut result: Vec<&'a FunctionInfo> = Vec::new();
    let mut seen: std::collections::HashSet<(&str, &str)> = std::collections::HashSet::new();
    for (fi, sig_key) in &keyed {
        if seen.insert((fi.name.as_str(), sig_key.as_str())) {
            if let Some(&best) = map.get(&(fi.name.as_str(), sig_key.as_str())) {
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
//  辅助工具（类型映射）
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
            namespace: None,
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
            is_ref_qualified: false,
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

    // ── lib_spec 去重逻辑测试 ─────────────────────────────────────────

    /// `struct T*` 与 `T*` 规范化后 cpp_sig 相同时只生成一条绑定（cpp_sig 去重）
    #[test]
    fn build_lib_spec_dedup_struct_prefix() {
        // 两个函数仅在参数类型写法上不同（"struct Foo*" vs "Foo*"），
        // build_fn_binding 会规范化为相同 cpp_sig，应只保留一条绑定
        let fi1 = make_fn("foo_run", "void", &["struct Foo*"]);
        let fi2 = make_fn("foo_run", "void", &["Foo*"]);
        let funcs = vec![&fi1, &fi2];
        let spec = build_lib_spec(&funcs, "test", &["Foo"]);
        assert_eq!(
            spec.fn_bindings.len(),
            1,
            "struct Foo* 与 Foo* 规范化后 cpp_sig 相同，应只生成一条绑定"
        );
    }

    /// C++ 重载（同名不同参数）应生成 `foo` / `foo_1` / `foo_2` 三条绑定
    #[test]
    fn build_lib_spec_overload_suffix() {
        let fi0 = make_fn("compute", "int", &[]);
        let fi1 = make_fn("compute", "int", &["int"]);
        let fi2 = make_fn("compute", "int", &["int", "int"]);
        let funcs = vec![&fi0, &fi1, &fi2];
        let spec = build_lib_spec(&funcs, "test", &[]);
        assert_eq!(spec.fn_bindings.len(), 3, "三个不同签名的重载各应生成一条绑定");
        // 第一个无后缀
        assert_eq!(spec.fn_bindings[0].rust_name, "compute");
        // 第二个追加 _1
        assert_eq!(spec.fn_bindings[1].rust_name, "compute_1");
        // 第三个追加 _2
        assert_eq!(spec.fn_bindings[2].rust_name, "compute_2");
    }

    /// 同名同参数签名的两条函数经 dedup_functions 只保留一条
    #[test]
    fn dedup_functions_same_sig_keeps_one() {
        let fi1 = FunctionInfo {
            name: "foo".to_string(),
            return_type: "void".to_string(),
            params: vec![ParamInfo {
                name: "x".to_string(),
                type_name: "int".to_string(),
                has_default: false,
            }],
            is_inline: false,
            is_variadic: false,
            is_extern_c: false,
            friend_of: None,
            body_offset: Some((0, 0)),
            is_from_current_file: true,
            namespace: None,
        };
        let fi2 = FunctionInfo {
            name: "foo".to_string(),
            return_type: "void".to_string(),
            params: vec![ParamInfo {
                name: "x".to_string(),
                type_name: "int".to_string(),
                has_default: false,
            }],
            is_inline: false,
            is_variadic: false,
            is_extern_c: true, // 低 score
            friend_of: None,
            body_offset: None,
            is_from_current_file: true,
            namespace: None,
        };
        let fns = vec![fi1, fi2];
        let result = dedup_functions(&fns);
        assert_eq!(result.len(), 1, "同名同签名只保留一条");
        // score 高者（body_offset=Some, is_extern_c=false）胜出
        assert!(!result[0].is_extern_c, "应保留 score 更高（非 extern-C）的版本");
    }

    /// 同名不同参数签名的两条函数各自保留
    #[test]
    fn dedup_functions_different_sig_both_kept() {
        let fi1 = make_fn("bar", "void", &["int"]);
        let fi2 = make_fn("bar", "void", &["float"]);
        let fns = vec![fi1, fi2];
        let result = dedup_functions(&fns);
        assert_eq!(result.len(), 2, "同名不同签名（重载）应各自保留");
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
            simple_name: name.to_string(),
            is_struct: false,
            is_abstract: false,
            template_args: vec![],
            bases: vec![],
            methods,
            fields: vec![],
            is_in_namespace: false,
            namespace: None,
            is_from_current_file: true,
            is_local_project: true,
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
            namespace: None,
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
            namespace: None,
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
        assert!(names.contains(&"get_value"), "应有 get_value（第一个）");
        assert!(
            names.contains(&"get_value_1"),
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

    /// type_references_class 词边界匹配测试
    #[test]
    fn type_references_class_exact_match() {
        assert!(type_references_class("Foo *", "Foo"));
        assert!(type_references_class("Foo*", "Foo"));
        assert!(type_references_class("Foo &", "Foo"));
        assert!(type_references_class("Foo", "Foo"));
        assert!(type_references_class("const Foo *", "Foo"));
        assert!(type_references_class("Foo*", "Foo"));
    }

    #[test]
    fn type_references_class_no_false_positive() {
        assert!(!type_references_class("FooBar *", "Foo"));
        assert!(!type_references_class("Foo2 *", "Foo"));
        assert!(!type_references_class("MyFooBar *", "Foo"));
        assert!(!type_references_class("aFoo *", "Foo"));
    }

    #[test]
    fn type_references_class_template_context() {
        assert!(type_references_class("Stack<Foo>", "Foo"));
        assert!(type_references_class("Stack<Foo*>", "Foo"));
        assert!(!type_references_class("Stack<FooBar>", "Foo"));
    }

    /// compute_used_classes 不再被子串误匹配（FooBar 不被 Foo 误匹配）
    #[test]
    fn compute_used_classes_no_substring_false_positive() {
        let classes = vec![ClassInfo {
            simple_name: "Foo".to_string(),
            name: "Foo".to_string(),
            is_struct: false,
            is_abstract: false,
            template_args: vec![],
            bases: vec![],
            methods: vec![],
            fields: vec![],
            is_in_namespace: false,
            namespace: None,
            is_from_current_file: true,
            is_local_project: true,
        }];
        let fi = FunctionInfo {
            name: "get_bar".to_string(),
            return_type: "FooBar *".to_string(),
            params: vec![],
            is_inline: false,
            is_variadic: false,
            is_extern_c: true,
            friend_of: None,
            body_offset: None,
            is_from_current_file: true,
            namespace: None,
        };
        let result = compute_used_classes(&classes, &[fi]);
        assert!(
            result.is_empty(),
            "FooBar* 不应被类名 'Foo' 子串匹配到，但结果为 {:?}",
            result
        );
    }

    /// compute_used_classes 正确匹配 "Foo *"
    #[test]
    fn compute_used_classes_correct_match() {
        let classes = vec![ClassInfo {
            simple_name: "Foo".to_string(),
            name: "Foo".to_string(),
            is_struct: false,
            is_abstract: false,
            template_args: vec![],
            bases: vec![],
            methods: vec![],
            fields: vec![],
            is_in_namespace: false,
            namespace: None,
            is_from_current_file: true,
            is_local_project: true,
        }];
        let fi = FunctionInfo {
            name: "foo_new".to_string(),
            return_type: "Foo *".to_string(),
            params: vec![],
            is_inline: false,
            is_variadic: false,
            is_extern_c: true,
            friend_of: None,
            body_offset: None,
            is_from_current_file: true,
            namespace: None,
        };
        let result = compute_used_classes(&classes, &[fi]);
        assert!(
            result.contains("Foo"),
            "Foo* 应被类名 'Foo' 匹配到，但结果为 {:?}",
            result
        );
    }

    // ── T1: collect_namespace 三方库函数渗透回归测试（续）──────────────────────

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
            namespace: None,
        };
        // 正常的当前文件函数
        let current_fn = make_fn("my_func", "int", &["int"]);

        // 通过 dedup_functions 直接检验 eligible_functions 逻辑：
        // 模拟 extract() 中的过滤器
        let all_fns = [third_party_fn.clone(), current_fn.clone()];
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

    // ── detect_namespace_mode 场景测试 ──────────────────────────────────────

    fn make_fn_extern_c(name: &str, return_type: &str, param_types: &[&str]) -> FunctionInfo {
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
            namespace: None,
        }
    }

    #[test]
    fn detect_namespace_mode_false_when_no_classes() {
        // has_any_classes=false → 无论 eligible_functions 内容如何，均返回 false
        let fi = make_fn_extern_c("foo", "void*", &[]);
        let result = detect_namespace_mode(false, &Default::default(), &[fi]);
        assert!(!result);
    }

    #[test]
    fn detect_namespace_mode_false_when_used_classes_nonempty() {
        // used_classes 非空时不触发命名空间模式
        let fi = make_fn_extern_c("foo", "void*", &[]);
        let used = std::collections::HashSet::from(["MyClass".to_string()]);
        let result = detect_namespace_mode(true, &used, &[fi]);
        assert!(!result);
    }

    #[test]
    fn detect_namespace_mode_true_on_void_ptr_return() {
        // 返回类型为 `void*`（规范化后）→ 触发命名空间模式
        let fi = make_fn_extern_c("get_ctx", "void *", &[]);
        let result = detect_namespace_mode(true, &Default::default(), &[fi]);
        assert!(result, "返回 void* 的 extern-C 函数应触发命名空间模式");
    }

    #[test]
    fn detect_namespace_mode_true_on_namespaced_return_type() {
        // 返回类型含 `::` → 触发命名空间模式
        let fi = make_fn_extern_c("make_it", "std::string", &[]);
        let result = detect_namespace_mode(true, &Default::default(), &[fi]);
        assert!(result, "返回含 :: 类型的 extern-C 函数应触发命名空间模式");
    }

    #[test]
    fn detect_namespace_mode_true_on_void_ptr_param() {
        // 参数类型规范化后含 `void*` → 触发命名空间模式
        let fi = make_fn_extern_c("process", "int", &["void *"]);
        let result = detect_namespace_mode(true, &Default::default(), &[fi]);
        assert!(result, "参数含 void* 的 extern-C 函数应触发命名空间模式");
    }

    #[test]
    fn detect_namespace_mode_false_non_extern_c() {
        // 函数非 extern-C，不触发命名空间模式
        let mut fi = make_fn_extern_c("foo", "void*", &[]);
        fi.is_extern_c = false;
        let result = detect_namespace_mode(true, &Default::default(), &[fi]);
        assert!(!result, "非 extern-C 函数不应触发命名空间模式");
    }

    #[test]
    fn detect_namespace_mode_false_plain_types() {
        // 普通类型（int、MyClass*）不触发命名空间模式
        let fi = make_fn_extern_c("do_thing", "MyClass*", &["int"]);
        let result = detect_namespace_mode(true, &Default::default(), &[fi]);
        assert!(!result, "普通类类型不应触发命名空间模式");
    }
}
