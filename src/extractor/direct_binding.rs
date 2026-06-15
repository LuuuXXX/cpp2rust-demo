//! 直接绑定模式判定（P1）
//!
//! 判定 C++ 项目应使用 direct 还是 shim 绑定模式：
//! - 无类项目 → [`BindingMode::Shim`]（保守，向后兼容）
//! - 存在任何 extern-C 函数返回类指针或首参为类指针 → [`BindingMode::Shim`]
//!   （项目显然有 C 包装层；覆盖命名规范的 `counter_*` 和不规范的 `file_*` 对 `FileHandle`）
//! - 否则（无任何函数接触类指针）→ [`BindingMode::Direct`]（纯 C++ 项目）
//!
//! Direct 模式下：每个有非静态方法的类对应一个 `hicc::make_unique<T>` 工厂，
//! `destroy_fn` 为 `None`（hicc 默认 `delete`），方法直接通过 `#[cpp(method = "...")]` 暴露。

use super::class_spec;
use super::type_mapper::{clean_type, cpp_to_rust, to_snake_case};
use crate::ast_parser::{ClassInfo, FunctionInfo, MethodInfo};
use crate::ffi_model::{BindingMode, ClassSpec, FnBinding, LibSpec};

/// 判定一个编译单元应使用 shim 还是 direct 绑定模式。
///
/// 判定流程：
/// 1. 无类项目 → `Shim`（保守，向后兼容）。
/// 2. 若**任何** extern-C 函数返回类指针或首参为类指针 → 项目显然有 C 包装层 → `Shim`。
///    这覆盖了命名规范的 `counter_*`/`file_handle_*` 与命名不规范的 `file_*`（对 FileHandle）。
/// 3. 否则（无任何函数接触类指针）→ 纯 C++ 项目，无 shim → `Direct`。
///
/// 第 2 步是关键的「保守信号」：只要项目里出现「extern-C 函数 + 类指针参数/返回值」
/// 的组合，就认为该项目为 C-API 风格，沿用 shim 流程，避免误把 file_open/file_close
/// 这类不规范的 C 包装识别为 direct 模式而丢失自定义 deleter。
pub fn classify(classes: &[ClassInfo], functions: &[FunctionInfo]) -> BindingMode {
    if classes.is_empty() {
        return BindingMode::Shim;
    }

    let class_names: Vec<&str> = classes.iter().map(|c| c.name.as_str()).collect();

    if has_any_class_pointer_or_ref_function(&class_names, functions) {
        return BindingMode::Shim;
    }

    BindingMode::Direct
}

/// 检查是否存在任何 extern-C 函数：返回类型或首参类型为某个类的**指针或引用**。
///
/// 这是「shim 信号」：只要项目中存在此类函数，就认为该项目为 C-API 风格，
/// 应使用 shim 模式（保守，向后兼容）。
///
/// 注意：返回裸类名（如 `Point`）的函数不算 shim 信号——在 Direct 模式下，
/// `std::make_unique<T>()` 返回 owned T，C++ 自由函数也可以返回类对象。
/// 只有指针/引用形式（如 `Counter*`、`const FileHandle*`）才表示项目提供了
/// C ABI shim 包装层。
fn has_any_class_pointer_or_ref_function(class_names: &[&str], functions: &[FunctionInfo]) -> bool {
    functions.iter().any(|fi| {
        if class_names
            .iter()
            .any(|cn| type_references_class_ptr_or_ref(&fi.return_type, cn))
        {
            return true;
        }
        if let Some(first_param) = fi.params.first() {
            if class_names
                .iter()
                .any(|cn| type_references_class_ptr_or_ref(&first_param.type_name, cn))
            {
                return true;
            }
        }
        false
    })
}

/// 检查 C++ 类型字符串是否引用了给定类的**指针或引用**形式。
///
/// 与 `type_references_class` 的区别：仅匹配 `Foo*` / `Foo&` / `const Foo*` / `struct Foo*`
/// 等指针/引用形式，**不**匹配裸类名 `Foo`（Direct 模式下工厂和自由函数可返回 owned T）。
fn type_references_class_ptr_or_ref(type_str: &str, class_name: &str) -> bool {
    let cleaned = type_str
        .replace("const ", "")
        .replace("volatile ", "")
        .replace("struct ", "")
        .replace("class ", "");
    // 按非字母数字下划线字符分词
    let tokens: Vec<&str> = cleaned
        .split(|c: char| !c.is_alphanumeric() && c != '_')
        .collect();
    // 查找 class_name token，然后检查其后面是否紧跟 * 或 & （指针或引用）
    for (i, tok) in tokens.iter().enumerate() {
        if *tok == class_name {
            // 检查 class_name 后面是否有 * 或 & 符号
            // 在 cleaned 字符串中定位 tok 的位置，查看紧跟的字符
            let after_class = cleaned.find(class_name).and_then(|pos| {
                let after = &cleaned[pos + class_name.len()..];
                // 去除空格后检查首字符
                after.trim_start().chars().next()
            });
            if let Some(next_char) = after_class {
                if next_char == '*' || next_char == '&' {
                    return true;
                }
            }
            // 也检查 &class_name（引用前缀）
            if i > 0 && tokens[i - 1] == "&" {
                return true;
            }
        }
    }
    false
}

// ─────────────────────────────────────────────────────────────────
//  Direct 模式 IR 构建（P1.3）
// ─────────────────────────────────────────────────────────────────

/// 判断类名是否为内部实现类（不应导出为 FFI 绑定）。
///
/// 内部类通常以 `detail_`、`internal_`、`dtoa_impl_` 等前缀开头，
/// 或名称含 `_impl`、`_helper`、`_fn`、`_tag`、`_cond`、`_base` 等后缀。
/// 这些类是库内部实现细节，不应暴露给 Rust 用户。
fn is_internal_class(name: &str) -> bool {
    let internal_prefixes = [
        "detail_",
        "detail2_",
        "internal_",
        "dtoa_impl_",
        "std_",
        "std_tuple_",
    ];
    let internal_suffixes = [
        "_impl", "_impl2", "_helper", "_fn", "_tag", "_cond", "_base", "_traits", "_adapter",
    ];
    internal_prefixes.iter().any(|p| name.starts_with(p))
        || internal_suffixes.iter().any(|s| name.ends_with(s))
}

/// 从命名空间扁平化类名中提取原始类名（不含命名空间前缀）。
///
/// 例如：`pugi_xml_node` → `xml_node`（从 `pugi::` 命名空间中）
///       `rapidjson_CrtAllocator` → `CrtAllocator`（从 `rapidjson::` 中）
///
/// 算法：用 `namespace_prefix`（如 `pugi::`）提取命名空间名（如 `pugi`），
/// 然后从扁平化类名中剥离 `namespace_` 前缀。
fn strip_namespace_from_name(flat_name: &str, namespace_prefix: &str) -> String {
    if namespace_prefix.is_empty() {
        return flat_name.to_string();
    }
    let ns_path = namespace_prefix.trim_end_matches(':').trim_end_matches(':');
    let flat_ns_path = ns_path.replace("::", "_");
    let full_prefix = format!("{}_", flat_ns_path);
    if flat_name.starts_with(&full_prefix) {
        return flat_name[full_prefix.len()..].to_string();
    }
    let inner_ns = ns_path
        .rsplit_once(':')
        .map(|(_, inner)| inner)
        .unwrap_or(ns_path);
    let inner_prefix = format!("{}_", inner_ns);
    if flat_name.starts_with(&inner_prefix) {
        return flat_name[inner_prefix.len()..].to_string();
    }
    flat_name.to_string()
}

/// 生成 `using` 别名声明，用于将命名空间类映射为 hicc 可用的扁平名称。
///
/// 例如：namespace_prefix=`pugi::`，flat_name=`pugi_xml_node` →
///       `using xml_node = pugi::xml_node;`
///
/// 返回 `(flat_name_for_rust, using_declaration)`。
/// 若类不在命名空间中，返回 `(original_name, None)`。
fn build_using_alias(ci: &ClassInfo) -> (String, Option<String>) {
    if !ci.is_in_namespace || ci.namespace_prefix.is_empty() {
        return (ci.name.clone(), None);
    }
    let flat_name = strip_namespace_from_name(&ci.name, &ci.namespace_prefix);
    let cpp_qualified = format!("{}{}", ci.namespace_prefix, flat_name);
    let using_decl = format!("using {} = {};", flat_name, cpp_qualified);
    (flat_name, Some(using_decl))
}

/// 在 direct 模式下构建所有可导出类的 `ClassSpec` 列表。
///
/// 与原版 `build_direct_class_specs` 的差异：
/// - 允许命名空间类（通过 `using` 别名映射为扁平名称）
/// - 过滤内部实现类（`detail_*`、`internal_*` 等）
/// - 所有类的 `destroy_fn` 为 `None`（hicc `make_unique` 默认调用 `delete`）
/// - 所有类的方法直接通过 `#[cpp(method = "...")]` 暴露
pub(crate) fn build_direct_class_specs(classes: &[ClassInfo]) -> (Vec<ClassSpec>, Vec<String>) {
    let eligible_classes: Vec<&ClassInfo> = classes
        .iter()
        .filter(|c| !c.name.is_empty() && !c.is_abstract && !is_internal_class(&c.name))
        .filter(|c| {
            if c.is_in_namespace && c.namespace_prefix.is_empty() {
                return false;
            }
            true
        })
        .collect();

    let exported_class_names: Vec<&str> =
        eligible_classes.iter().map(|c| c.name.as_str()).collect();

    let using_aliases: Vec<String> = eligible_classes
        .iter()
        .filter_map(|ci| build_using_alias(ci).1)
        .collect();

    let class_specs: Vec<ClassSpec> = eligible_classes
        .iter()
        .map(|ci| {
            let (flat_name, _) = build_using_alias(ci);
            let mut spec = class_spec::build_class_spec(ci, classes, &exported_class_names)
                .unwrap_or_else(|| ClassSpec {
                    name: flat_name.clone(),
                    methods: Vec::new(),
                    associated_fns: Vec::new(),
                    destroy_fn: None,
                    is_interface: false,
                });
            spec.name = flat_name;
            spec.destroy_fn = None;
            spec
        })
        .collect();

    (class_specs, using_aliases)
}

/// 在 direct 模式下构建 `LibSpec`：
/// 1. 每个已通过 `build_direct_class_specs` 过滤的类对应一个 `make_unique<T>` 工厂。
/// 2. 保留所有未归类为 MethodAccessor / Ctor / Dtor 的「Standalone」自由函数
///    （如 `sum_zero`、`manhattan_distance` 等与类无关的全局函数）。
///
/// 形如：
/// ```text
/// #[cpp(func = "std::unique_ptr<Counter> hicc::make_unique<Counter>()")]
/// pub fn counter_new() -> Counter;
/// ```
///
/// 与 shim 模式的差异：
/// - 不含 MethodAccessor/Ctor/Dtor 类型的 shim 绑定（counter_get/counter_delete 等）
/// - 每个有非静态方法的类一个工厂，返回 owned T（不是 *mut T）
/// - Standalone 自由函数保留（与 shim 模式一致，避免丢失导出符号）
pub(crate) fn build_direct_lib_spec(
    class_specs: &[ClassSpec],
    all_classes: &[ClassInfo],
    functions: &[&FunctionInfo],
    unit_name: &str,
) -> LibSpec {
    let link_name = std::path::Path::new(unit_name)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or(unit_name)
        .to_string();

    // 类名用于 classify_fn 判定（识别哪些函数是某个类的 accessor/ctor/dtor）
    let class_names: Vec<&str> = all_classes.iter().map(|c| c.name.as_str()).collect();

    // 收集 standalone 自由函数（不属于任何类的 MethodAccessor / Dtor）。
    // Direct 模式下「Ctor」由 make_unique 工厂替代，「MethodAccessor」由 #[cpp(method)] 替代，
    // 「Dtor」由 hicc 默认 delete 替代。但 Standalone 和 StaticAccessor 是真正的 C++ 函数，
    // 需要保留（StaticAccessor 在 shim 模式下是静态方法包装，但在 direct 模式下是
    // 真正的自由函数，如 point_new_polar）。
    let classified = super::classify_functions(functions, &class_names);
    let standalone_fns: Vec<FnBinding> = classified
        .iter()
        .filter(|(_, kind)| {
            matches!(
                kind,
                super::ShimKind::Standalone | super::ShimKind::StaticAccessor
            )
        })
        .filter(|(fi, _)| !fi.is_variadic)
        .filter(|(fi, _)| !fi.name.starts_with("operator"))
        .filter(|(fi, _)| !fi.params.iter().any(|p| p.type_name.contains("::*)")))
        .filter(|(fi, _)| !fi.return_type.contains("::*)"))
        .filter(|(fi, _)| {
            fi.params.iter().all(|p| {
                super::is_mappable_rust_type(
                    &super::type_mapper::cpp_to_rust(&p.type_name),
                    &class_names,
                )
            }) && super::is_mappable_rust_type(
                &super::type_mapper::cpp_to_rust(&fi.return_type),
                &class_names,
            )
        })
        .map(|(fi, _)| super::lib_spec::build_fn_binding(fi, &class_names))
        .collect();

    // make_unique 工厂：为每个有非静态方法的类生成。
    // 多构造函数：每个 public 构造函数对应一个 make_unique<T>(args) 工厂。
    // - 默认构造函数 → hicc::make_unique<T>()（hicc 特殊版本，确保正确析构）
    // - 带参数构造函数 → std::make_unique<T>(arg_types)（标准 C++ 转发）
    // - 移动构造函数（参数含 &&）→ 跳过（Rust 已有自身移动语义）
    // class_map_primary: 映射 ClassInfo.name → ClassInfo（使用命名空间前缀名）
    let class_map_primary: std::collections::HashMap<&str, &ClassInfo> = all_classes
        .iter()
        .filter(|c| !c.name.is_empty() && !c.is_abstract)
        .map(|c| (c.name.as_str(), c))
        .collect();
    // ns_alias_to_original: 映射扁平别名名 → 原始命名空间前缀名
    let ns_alias_to_original: std::collections::HashMap<String, String> = all_classes
        .iter()
        .filter(|c| c.is_in_namespace && !c.namespace_prefix.is_empty())
        .map(|c| {
            (
                strip_namespace_from_name(&c.name, &c.namespace_prefix),
                c.name.clone(),
            )
        })
        .collect();

    // lookup_class_info: 查找 ClassInfo，先按当前名查找，再按别名→原始名二级查找
    let lookup_class_info = |name: &str| -> Option<&ClassInfo> {
        if let Some(ci) = class_map_primary.get(name) {
            return Some(ci);
        }
        if let Some(original) = ns_alias_to_original.get(name) {
            class_map_primary.get(original.as_str()).copied()
        } else {
            None
        }
    };

    let empty_ctor = MethodInfo {
        name: String::new(),
        return_type: String::new(),
        params: Vec::new(),
        is_const: false,
        is_volatile: false,
        is_virtual: false,
        is_pure_virtual: false,
        is_static: false,
        is_constructor: true,
        is_destructor: false,
        is_inline: false,
        accessibility: "public".to_string(),
        body_offset: None,
        is_override: false,
        is_default: false,
        is_copy_ctor: false,
    };
    let is_move_ctor = |ctor: &MethodInfo| -> bool {
        ctor.params.iter().any(|p| {
            let t = p.type_name.trim();
            t.contains("&&")
                || (t == class_name_from_ctor_param(t, all_classes)
                    && !t.contains('*')
                    && !t.contains('&'))
        })
    };
    let is_deleted_ctor = |ctor: &MethodInfo| -> bool {
        ctor.is_copy_ctor && !ctor.is_default && ctor.body_offset.is_none()
    };
    let mut factory_fns: Vec<FnBinding> = class_specs
        .iter()
        .flat_map(|cs| {
            let ci = match lookup_class_info(&cs.name) {
                Some(ci) => ci,
                None => return vec![build_make_unique_factory(&cs.name, &empty_ctor)],
            };
            let has_only_static_methods = ci.methods.iter().all(|m| {
                m.is_static || m.is_constructor || m.is_destructor || m.accessibility != "public"
            });
            if has_only_static_methods {
                return Vec::new();
            }
            let ctors: Vec<&MethodInfo> = ci
                .methods
                .iter()
                .filter(|m| {
                    m.is_constructor
                        && m.accessibility == "public"
                        && !is_move_ctor(m)
                        && !is_deleted_ctor(m)
                })
                .collect();
            if ctors.is_empty() {
                vec![build_make_unique_factory(&cs.name, &empty_ctor)]
            } else {
                ctors
                    .iter()
                    .map(|ctor| build_make_unique_factory(&cs.name, ctor))
                    .collect()
            }
        })
        .collect();

    // 解析工厂函数命名冲突：当多个构造函数产生相同 Rust 名称时，
    // 用参数类型后缀区分（如 widget_new_with_v_i32 vs widget_new_with_v_f64）
    resolve_factory_name_conflicts(&mut factory_fns);

    // 静态方法绑定：为每个有 public 静态方法的类生成独立函数绑定。
    // 静态方法不属于 import_class!（无 self/this 参数），需要单独作为 import_lib! 中的函数。
    // 形如：#[cpp(func = "RetType ClassName::methodName(params)")]
    //       pub fn class_name_method_name(params) -> RetType;
    let static_fns: Vec<FnBinding> = class_specs
        .iter()
        .flat_map(|cs| {
            let ci = match lookup_class_info(&cs.name) {
                Some(ci) => ci,
                None => return Vec::new(),
            };
            ci.methods
                .iter()
                .filter(|m| {
                    m.is_static
                        && m.accessibility == "public"
                        && !m.is_constructor
                        && !m.is_destructor
                })
                .filter(|m| {
                    let rust_ret = cpp_to_rust(&m.return_type);
                    super::is_mappable_rust_type(&rust_ret, &class_names)
                        && m.params.iter().all(|p| {
                            super::is_mappable_rust_type(&cpp_to_rust(&p.type_name), &class_names)
                        })
                })
                .map(|m| build_static_method_binding(&cs.name, m, &class_names))
                .collect()
        })
        .collect();

    // 合并：standalone 在前，静态方法次之，工厂最后
    let mut fn_bindings: Vec<FnBinding> = standalone_fns;
    fn_bindings.extend(static_fns);
    fn_bindings.extend(factory_fns);

    // 前向声明：保留 lib_spec 函数引用的类。
    // 使用扁平名称（如 ConfigManager）而非命名空间前缀名（如 config_ConfigManager），
    // 因为 import_lib! 的 #[cpp(func)] 签名引用的是 using 别名名。
    let spec_class_names: Vec<&str> = class_specs.iter().map(|cs| cs.name.as_str()).collect();
    let used_spec_classes: std::collections::HashSet<&str> = fn_bindings
        .iter()
        .flat_map(|fb| {
            spec_class_names.iter().filter(move |cn| {
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
    let fwd_decls: Vec<String> = spec_class_names
        .iter()
        .filter(|cn| used_spec_classes.contains(**cn))
        .map(|n| format!("class {};", n))
        .collect();

    LibSpec {
        link_name,
        fwd_decls,
        fn_bindings,
    }
}

/// 检查构造函数参数类型是否为类名自身（裸类名或类引用，不含指针），
/// 用于识别移动/拷贝构造函数。
fn class_name_from_ctor_param<'a>(type_str: &str, all_classes: &'a [ClassInfo]) -> &'a str {
    let cleaned: String = type_str
        .replace("const ", "")
        .replace("volatile ", "")
        .replace("struct ", "")
        .replace("class ", "")
        .replace("&&", "")
        .replace("&", "");
    all_classes
        .iter()
        .find(|c| c.name == cleaned)
        .map(|c| c.name.as_str())
        .unwrap_or("")
}

/// 为单个类生成 `make_unique<T>` 工厂绑定。
///
/// 工厂命名规则（使用 CamelCase→snake_case 转换）：
/// - 默认构造函数（无参数）：`<class_snake>_new`（如 `counter_new`、`unique_vector_new`）
/// - 带参数构造函数：`<class_snake>_new_<n>` 其中 n 为参数数量（如 `point_new_2`）
///   当参数为 1 时，用 `<class_snake>_new_with_<param_name>`（如 `buffer_new_with_sz`）
///
/// C++ 签名：
/// - 默认构造函数：`std::unique_ptr<T> hicc::make_unique<T>()`（hicc 版本）
/// - 带参数构造函数：`std::unique_ptr<T> std::make_unique<T>(arg_types)`（标准 C++ 版本）
///
/// `ctor`：构造函数的 MethodInfo。`&[]` 表示默认构造函数（无参数）。
fn build_make_unique_factory(class_name: &str, ctor: &MethodInfo) -> FnBinding {
    let class_snake = to_snake_case(class_name);
    let param_types: Vec<String> = ctor
        .params
        .iter()
        .map(|p| super::normalize_ptr_spacing(super::strip_volatile(clean_type(&p.type_name))))
        .collect();
    let rust_params: Vec<(String, String)> = ctor
        .params
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let name = if p.name.is_empty() || p.name == "_" {
                format!("arg{}", i)
            } else {
                super::sanitize_param_name(&p.name, i)
            };
            (name, cpp_to_rust(&p.type_name))
        })
        .collect();

    let is_default = ctor.params.is_empty();
    let rust_name = if is_default {
        format!("{}_new", class_snake)
    } else if ctor.params.len() == 1 {
        let param_name = rust_params[0].0.clone();
        format!("{}_new_with_{}", class_snake, param_name)
    } else {
        format!("{}_new_{}", class_snake, ctor.params.len())
    };

    let cpp_args = if param_types.is_empty() {
        String::new()
    } else {
        param_types.join(", ")
    };
    let make_unique_ns = if is_default { "hicc" } else { "std" };
    let cpp_sig = format!(
        "std::unique_ptr<{cls}> {ns}::make_unique<{cls}>({args})",
        cls = class_name,
        ns = make_unique_ns,
        args = cpp_args
    );

    let is_unsafe = should_mark_unsafe(&rust_params, class_name);

    FnBinding {
        cpp_sig,
        rust_name,
        params: rust_params,
        ret_type: Some(class_name.to_string()),
        is_unsafe,
        has_fn_ptr_param: false,
    }
}

/// 为类的 public 静态方法生成独立函数绑定（放入 import_lib!）。
///
/// 静态方法形如：
/// ```text
/// #[cpp(func = "int Counter::getInstanceCount()")]
/// pub fn counter_get_instance_count() -> i32;
/// ```
fn build_static_method_binding(
    class_name: &str,
    method: &MethodInfo,
    _class_names: &[&str],
) -> FnBinding {
    let class_snake = to_snake_case(class_name);
    let method_snake = to_snake_case(&method.name);
    let rust_name = format!("{}_{}", class_snake, method_snake);

    let param_types: Vec<String> = method
        .params
        .iter()
        .map(|p| super::normalize_ptr_spacing(super::strip_volatile(clean_type(&p.type_name))))
        .collect();
    let cpp_args = if param_types.is_empty() {
        String::new()
    } else {
        param_types.join(", ")
    };

    let ret_type_str = if method.return_type.is_empty() || method.return_type == "void" {
        "void".to_string()
    } else {
        super::normalize_ptr_spacing(&super::strip_struct_class_keyword(clean_type(
            &method.return_type,
        )))
    };

    let cpp_sig = format!(
        "{ret} {cls}::{method}({args})",
        ret = ret_type_str,
        cls = class_name,
        method = method.name,
        args = cpp_args
    );

    let rust_params: Vec<(String, String)> = method
        .params
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let name = if p.name.is_empty() || p.name == "_" {
                format!("arg{}", i)
            } else {
                super::sanitize_param_name(&p.name, i)
            };
            (name, cpp_to_rust(&p.type_name))
        })
        .collect();

    let rust_ret = if method.return_type.is_empty() || method.return_type == "void" {
        None
    } else {
        Some(cpp_to_rust(&method.return_type))
    };

    let is_unsafe = should_mark_unsafe(&rust_params, class_name);

    FnBinding {
        cpp_sig,
        rust_name,
        params: rust_params,
        ret_type: rust_ret,
        is_unsafe,
        has_fn_ptr_param: false,
    }
}

/// 解析工厂函数命名冲突：当同一类的多个构造函数产生相同的 Rust 名称时，
/// 用第一个参数的 Rust 类型后缀区分。
///
/// 例如：Widget(int v) 和 Widget(double v) 都产生 `widget_new_with_v`，
/// 解析后变成 `widget_new_with_v_i32` 和 `widget_new_with_v_f64`。
fn resolve_factory_name_conflicts(factory_fns: &mut [FnBinding]) {
    let mut name_counts: std::collections::HashMap<String, Vec<usize>> =
        std::collections::HashMap::new();
    for (i, fb) in factory_fns.iter().enumerate() {
        name_counts.entry(fb.rust_name.clone()).or_default().push(i);
    }

    for (name, indices) in &name_counts {
        if indices.len() > 1 {
            for &idx in indices {
                let fb = &mut factory_fns[idx];
                let type_suffix = if let Some((_, t)) = fb.params.first() {
                    sanitize_type_suffix(t)
                } else {
                    format!("{}", indices.len())
                };
                fb.rust_name = format!("{}_{}", name, type_suffix);
            }
        }
    }
}

/// 将 Rust 类型名转为合法的函数名后缀片段。
fn sanitize_type_suffix(t: &str) -> String {
    t.replace("*mut ", "ptr_mut_")
        .replace("*const ", "ptr_const_")
        .replace("&mut ", "ref_mut_")
        .replace("&", "ref_")
        .replace(" ", "_")
        .replace("<", "_")
        .replace(">", "_")
}

/// 判断函数是否需要标记为 `unsafe`。
/// 规则：参数或返回类型含 raw pointer（*mut/*const i8 等）或 C 函数指针 → unsafe。
fn should_mark_unsafe(rust_params: &[(String, String)], class_name: &str) -> bool {
    rust_params.iter().any(|(_, t)| {
        if t == "*const i8" {
            return true;
        }
        if t.starts_with("unsafe extern") {
            return true;
        }
        if let Some(inner) = t.strip_prefix("*mut ") {
            return inner != class_name;
        }
        false
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast_parser::{ClassInfo, FunctionInfo, MethodInfo, ParamInfo};

    fn make_class(name: &str) -> ClassInfo {
        ClassInfo {
            name: name.to_string(),
            is_struct: false,
            is_abstract: false,
            template_args: Vec::new(),
            bases: Vec::new(),
            methods: Vec::new(),
            fields: Vec::new(),
            is_in_namespace: false,
            namespace_prefix: String::new(),
            is_from_current_file: true,
        }
    }

    fn make_class_with_method(name: &str, method_name: &str) -> ClassInfo {
        let mut class = make_class(name);
        class.methods = vec![MethodInfo {
            name: method_name.to_string(),
            return_type: "int".to_string(),
            params: Vec::new(),
            is_const: true,
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
            is_copy_ctor: false,
        }];
        class
    }

    fn make_function(name: &str, params: Vec<ParamInfo>, return_type: &str) -> FunctionInfo {
        FunctionInfo {
            name: name.to_string(),
            return_type: return_type.to_string(),
            params,
            is_inline: false,
            is_variadic: false,
            is_extern_c: true,
            friend_of: None,
            body_offset: None,
            is_from_current_file: true,
        }
    }

    fn make_param(name: &str, type_name: &str) -> ParamInfo {
        ParamInfo {
            name: name.to_string(),
            type_name: type_name.to_string(),
            has_default: false,
        }
    }

    // ── classify 基础场景 ──────────────────────────────────────────

    #[test]
    fn classify_no_classes_returns_shim() {
        // 无类项目：保守返回 Shim
        let mode = classify(&[], &[]);
        assert_eq!(mode, BindingMode::Shim);
    }

    #[test]
    fn classify_pure_direct_no_shim_functions() {
        // 纯 C++ 类，无任何配对 shim 函数 → Direct
        let classes = vec![make_class_with_method("Counter", "get")];
        let functions = vec![]; // 无 extern-C shim
        let mode = classify(&classes, &functions);
        assert_eq!(mode, BindingMode::Direct);
    }

    #[test]
    fn classify_pure_shim_with_paired_functions() {
        // 类配对 shim 函数 → Shim
        let classes = vec![make_class_with_method("Counter", "get")];
        let functions = vec![
            // counter_new 是工厂函数（返回 Counter*）
            make_function("counter_new", vec![], "Counter*"),
            // counter_get 是访问器（首参 Counter*）
            make_function("counter_get", vec![make_param("self", "Counter*")], "int"),
        ];
        let mode = classify(&classes, &functions);
        assert_eq!(mode, BindingMode::Shim);
    }

    #[test]
    fn classify_mixed_returns_shim_with_warning() {
        // 即使只有部分类被 extern-C 函数接触（widget_new 返回 Widget*），
        // 整体按 Shim 模式处理（保守）
        let classes = vec![
            make_class_with_method("Counter", "get"),
            make_class_with_method("Widget", "render"),
        ];
        let functions = vec![
            // 只为 Widget 提供 extern-C 包装
            make_function("widget_new", vec![], "Widget*"),
        ];
        let mode = classify(&classes, &functions);
        assert_eq!(mode, BindingMode::Shim);
    }

    #[test]
    fn classify_filehandle_style_shim_returns_shim() {
        // 不规范命名的 shim（file_ 而非 filehandle_）：file_open 返回 FileHandle*
        // 仍应被识别为 shim 模式（避免丢失自定义 deleter）
        let classes = vec![make_class_with_method("FileHandle", "is_open")];
        let functions = vec![
            make_function(
                "file_open",
                vec![make_param("filename", "const char*")],
                "FileHandle*",
            ),
            make_function(
                "file_close",
                vec![make_param("handle", "FileHandle*")],
                "void",
            ),
        ];
        let mode = classify(&classes, &functions);
        assert_eq!(mode, BindingMode::Shim);
    }

    #[test]
    fn classify_template_class_without_shim_returns_direct() {
        // 模板类无配对 shim → Direct
        let mut class = make_class_with_method("Stack", "push");
        class.template_args = vec!["T".to_string()];
        let mode = classify(&[class], &[]);
        assert_eq!(mode, BindingMode::Direct);
    }

    // ── has_any_class_pointer_or_ref_function 细节测试 ───────────────────

    #[test]
    fn has_any_class_pointer_or_ref_function_factory_returns_true() {
        // counter_new() -> Counter*：返回类型为类指针 → 有 shim 信号
        let fns = vec![make_function("counter_new", vec![], "Counter*")];
        assert!(has_any_class_pointer_or_ref_function(&["Counter"], &fns));
    }

    #[test]
    fn has_any_class_pointer_or_ref_function_first_param_class_ptr_returns_true() {
        // file_close(FileHandle* h)：首参为类指针 → 有 shim 信号（即使命名不规范）
        let fns = vec![make_function(
            "file_close",
            vec![make_param("h", "FileHandle*")],
            "void",
        )];
        assert!(has_any_class_pointer_or_ref_function(&["FileHandle"], &fns));
    }

    #[test]
    fn has_any_class_pointer_or_ref_function_unrelated_returns_false() {
        // sum_zero() / manhattan_distance(int, int)：与任何类无关 → 无 shim 信号
        let fns = vec![
            make_function("sum_zero", vec![], "int"),
            make_function(
                "manhattan_distance",
                vec![make_param("x", "int"), make_param("y", "int")],
                "int",
            ),
        ];
        assert!(!has_any_class_pointer_or_ref_function(&["Counter"], &fns));
    }

    #[test]
    fn has_any_class_pointer_or_ref_function_const_class_ptr_returns_true() {
        // const Counter* 也算指针/引用
        let fns = vec![make_function(
            "counter_value",
            vec![make_param("self", "const Counter*")],
            "int",
        )];
        assert!(has_any_class_pointer_or_ref_function(&["Counter"], &fns));
    }

    #[test]
    fn has_any_class_pointer_or_ref_function_empty_returns_false() {
        assert!(!has_any_class_pointer_or_ref_function(&["Counter"], &[]));
    }

    #[test]
    fn has_any_class_pointer_or_ref_function_bare_class_returns_false() {
        // point_new_polar() -> Point：返回 owned T（非指针/引用）→ 非 shim 信号
        let fns = vec![make_function("point_new_polar", vec![], "Point")];
        assert!(!has_any_class_pointer_or_ref_function(&["Point"], &fns));
    }

    // ── type_references_class_ptr_or_ref 单元测试 ────────────────────────────

    #[test]
    fn type_references_class_ptr_or_ref_plain_ptr() {
        assert!(type_references_class_ptr_or_ref("Counter*", "Counter"));
        assert!(type_references_class_ptr_or_ref("Counter&", "Counter"));
    }

    #[test]
    fn type_references_class_ptr_or_ref_with_const() {
        assert!(type_references_class_ptr_or_ref(
            "const Counter*",
            "Counter"
        ));
        assert!(type_references_class_ptr_or_ref(
            "const Counter&",
            "Counter"
        ));
    }

    #[test]
    fn type_references_class_ptr_or_ref_with_struct_keyword() {
        assert!(type_references_class_ptr_or_ref(
            "struct Counter*",
            "Counter"
        ));
    }

    #[test]
    fn type_references_class_ptr_or_ref_unrelated() {
        assert!(!type_references_class_ptr_or_ref("Widget*", "Counter"));
        assert!(!type_references_class_ptr_or_ref("int", "Counter"));
    }

    #[test]
    fn type_references_class_ptr_or_ref_prefixed_name_not_match() {
        // CounterEx 不应匹配 Counter（按 token 完全匹配）
        assert!(!type_references_class_ptr_or_ref("CounterEx*", "Counter"));
    }

    #[test]
    fn type_references_class_ptr_or_ref_bare_class_name_returns_false() {
        // 裸类名 Point（非指针/引用）不应匹配
        assert!(!type_references_class_ptr_or_ref("Point", "Point"));
        assert!(!type_references_class_ptr_or_ref("Buffer", "Buffer"));
    }

    // ── is_internal_class 单元测试 ──────────────────────────────────────────

    #[test]
    fn is_internal_class_detail_prefix() {
        assert!(is_internal_class("detail_nonesuch"));
        assert!(is_internal_class("detail_position_t"));
        assert!(is_internal_class("detail2_begin_tag"));
        assert!(is_internal_class("detail2_end_tag"));
    }

    #[test]
    fn is_internal_class_internal_prefix() {
        assert!(is_internal_class("internal_SelectIfImpl"));
        assert!(is_internal_class("internal_Double"));
        assert!(is_internal_class("internal_BigInteger"));
    }

    #[test]
    fn is_internal_class_dtoa_impl_prefix() {
        assert!(is_internal_class("dtoa_impl_diyfp"));
        assert!(is_internal_class("dtoa_impl_boundaries"));
        assert!(is_internal_class("dtoa_impl_cached_power"));
    }

    #[test]
    fn is_internal_class_std_prefix() {
        assert!(is_internal_class("std_less"));
        assert!(is_internal_class("std_tuple_size"));
        assert!(is_internal_class("std_tuple_element"));
    }

    #[test]
    fn is_internal_class_suffix_impl() {
        assert!(is_internal_class("value_in_range_of_impl2"));
        assert!(is_internal_class("json_ref_impl"));
    }

    #[test]
    fn is_internal_class_suffix_helper_fn_tag_cond_base_traits_adapter() {
        assert!(is_internal_class("foo_helper"));
        assert!(is_internal_class("from_json_fn"));
        assert!(is_internal_class("begin_tag"));
        assert!(is_internal_class("conjunction_cond"));
        assert!(is_internal_class("json_default_base"));
        assert!(is_internal_class("char_traits"));
        assert!(is_internal_class("input_adapter"));
    }

    #[test]
    fn is_internal_class_public_class_returns_false() {
        assert!(!is_internal_class("xml_node"));
        assert!(!is_internal_class("xml_attribute"));
        assert!(!is_internal_class("CrtAllocator"));
        assert!(!is_internal_class("ParseResult"));
        assert!(!is_internal_class("Counter"));
        assert!(!is_internal_class("MemoryStream"));
    }

    // ── strip_namespace_from_name 单元测试 ──────────────────────────────────

    #[test]
    fn strip_namespace_from_name_pugi() {
        assert_eq!(
            strip_namespace_from_name("pugi_xml_node", "pugi::"),
            "xml_node"
        );
        assert_eq!(
            strip_namespace_from_name("pugi_xml_attribute", "pugi::"),
            "xml_attribute"
        );
    }

    #[test]
    fn strip_namespace_from_name_rapidjson() {
        assert_eq!(
            strip_namespace_from_name("rapidjson_CrtAllocator", "rapidjson::"),
            "CrtAllocator"
        );
        assert_eq!(
            strip_namespace_from_name("rapidjson_ParseResult", "rapidjson::"),
            "ParseResult"
        );
    }

    #[test]
    fn strip_namespace_from_name_nested_namespace() {
        assert_eq!(
            strip_namespace_from_name("foo_bar_config_ConfigManager", "foo::bar::config::"),
            "ConfigManager"
        );
    }

    #[test]
    fn strip_namespace_from_name_inner_ns_fallback() {
        // flat_name 不含完整路径前缀时，回退到最内层命名空间前缀
        assert_eq!(
            strip_namespace_from_name("config_ConfigManager", "foo::bar::config::"),
            "ConfigManager"
        );
    }

    #[test]
    fn strip_namespace_from_name_empty_prefix_returns_original() {
        assert_eq!(strip_namespace_from_name("Counter", ""), "Counter");
    }

    #[test]
    fn strip_namespace_from_name_no_match_returns_original() {
        assert_eq!(
            strip_namespace_from_name("SomeClass", "rapidjson::"),
            "SomeClass"
        );
    }

    // ── build_using_alias 单元测试 ──────────────────────────────────────────

    #[test]
    fn build_using_alias_namespace_class() {
        let ci = ClassInfo {
            name: "pugi_xml_node".to_string(),
            is_struct: false,
            is_abstract: false,
            template_args: Vec::new(),
            bases: Vec::new(),
            methods: Vec::new(),
            fields: Vec::new(),
            is_in_namespace: true,
            namespace_prefix: "pugi::".to_string(),
            is_from_current_file: true,
        };
        let (flat_name, using_decl) = build_using_alias(&ci);
        assert_eq!(flat_name, "xml_node");
        assert_eq!(
            using_decl,
            Some("using xml_node = pugi::xml_node;".to_string())
        );
    }

    #[test]
    fn build_using_alias_no_namespace_returns_none() {
        let ci = ClassInfo {
            name: "Counter".to_string(),
            is_struct: false,
            is_abstract: false,
            template_args: Vec::new(),
            bases: Vec::new(),
            methods: Vec::new(),
            fields: Vec::new(),
            is_in_namespace: false,
            namespace_prefix: String::new(),
            is_from_current_file: true,
        };
        let (flat_name, using_decl) = build_using_alias(&ci);
        assert_eq!(flat_name, "Counter");
        assert_eq!(using_decl, None);
    }

    #[test]
    fn build_using_alias_namespace_but_empty_prefix_returns_none() {
        let ci = ClassInfo {
            name: "SomeClass".to_string(),
            is_struct: false,
            is_abstract: false,
            template_args: Vec::new(),
            bases: Vec::new(),
            methods: Vec::new(),
            fields: Vec::new(),
            is_in_namespace: true,
            namespace_prefix: String::new(),
            is_from_current_file: true,
        };
        let (flat_name, using_decl) = build_using_alias(&ci);
        assert_eq!(flat_name, "SomeClass");
        assert_eq!(using_decl, None);
    }

    #[test]
    fn build_using_alias_rapidjson_class() {
        let ci = ClassInfo {
            name: "rapidjson_CrtAllocator".to_string(),
            is_struct: false,
            is_abstract: false,
            template_args: Vec::new(),
            bases: Vec::new(),
            methods: Vec::new(),
            fields: Vec::new(),
            is_in_namespace: true,
            namespace_prefix: "rapidjson::".to_string(),
            is_from_current_file: true,
        };
        let (flat_name, using_decl) = build_using_alias(&ci);
        assert_eq!(flat_name, "CrtAllocator");
        assert_eq!(
            using_decl,
            Some("using CrtAllocator = rapidjson::CrtAllocator;".to_string())
        );
    }
}
