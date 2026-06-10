//! 冒烟测试生成器（Phase init）
//!
//! 从所有编译单元的 `FfiSpec` 生成 `tests/smoke_test.rs` 的内容。
//! 测试分四类：
//! - A：类生命周期（构造 → 方法调用 → 析构）
//! - B：独立自由函数（全基本类型参数）
//! - C：含指针/函数指针参数（生成注释桩，提示人工补充）
//! - D：接口类（纯虚）工厂函数测试

use crate::ffi_model::{ClassSpec, FfiSpec, FnBinding, MethodBinding, SelfKind};

// ─────────────────────────────────────────────
//  公开 API
// ─────────────────────────────────────────────

/// 从多个编译单元生成冒烟测试文件内容。
///
/// `units`：`(unit_path, &FfiSpec)` 对的列表，顺序与 init 生成顺序一致。
/// `lib_name`：Cargo `[lib] name`（`-` 已替换为 `_`）。
pub fn generate(units: &[(&str, &FfiSpec)], lib_name: &str) -> String {
    let mut out = String::new();

    // ── 文件头 ──────────────────────────────────────────────────────────
    out.push_str("// 自动生成的 FFI 冒烟测试 — 由 cpp2rust-demo init 生成\n");
    out.push_str("// 用途：验证 FFI 层编译链接正常，基本接口可调用\n");
    out.push_str("// 运行：cargo test -- --nocapture（需要已编译的 C++ 库）\n");
    out.push_str("#![allow(unused_imports, dead_code, unused_variables, unused_unsafe)]\n");
    out.push('\n');

    // ── use 声明（每个 unit 一条）────────────────────────────────────────
    for (unit_path, _) in units {
        let mod_path = mod_path_from_unit(unit_path);
        out.push_str(&format!("use {}::{}::*;\n", lib_name, mod_path));
    }
    out.push('\n');

    // ── 按单元生成测试 ────────────────────────────────────────────────────
    for (unit_path, spec) in units {
        out.push_str(&format!("// ═══ 单元：{} ═══\n", unit_path));
        out.push('\n');

        // 类别 A：类生命周期测试
        for cs in &spec.class_specs {
            out.push_str(&emit_class_lifecycle(unit_path, spec, cs));
        }

        // 类别 B/C：独立自由函数
        for fb in &spec.lib_spec.fn_bindings {
            out.push_str(&emit_free_fn(unit_path, fb));
        }
    }

    out
}

// ─────────────────────────────────────────────
//  类别 A：类生命周期
// ─────────────────────────────────────────────

fn emit_class_lifecycle(unit_path: &str, spec: &FfiSpec, cs: &ClassSpec) -> String {
    let mut out = String::new();

    // 接口类（纯虚，is_interface=true）走类别 D
    if cs.is_interface {
        return emit_interface_via_factory(unit_path, spec, cs);
    }

    // 必须有 destroy_fn，才能做安全生命周期测试
    let Some(_dtor) = &cs.destroy_fn else {
        return out;
    };

    // 找构造函数：associated_fns 中返回类名本身（owned，已经去掉 *mut 前缀）的函数
    let ctor = cs
        .associated_fns
        .iter()
        .find(|fb| fb.ret_type.as_deref() == Some(cs.name.as_str()));

    let Some(ctor) = ctor else {
        // 没有可用构造函数，生成注释提示
        out.push_str(&format!(
            "// smoke_{unit}: {class} 无可直接使用的构造函数，需人工补充测试\n\n",
            unit = test_name_segment(unit_path),
            class = cs.name.to_lowercase()
        ));
        return out;
    };

    // 检查构造函数参数是否全部可以生成零值
    let ctor_args = match build_args(&ctor.params) {
        ArgsResult::Ok(args) => args,
        ArgsResult::NeedsUnsafe(args) => args,
        ArgsResult::HasPointer => {
            // ctor 有指针参数，生成注释版本（类别 C）
            out.push_str(&emit_class_lifecycle_stub(unit_path, cs, ctor));
            return out;
        }
    };
    let ctor_unsafe = needs_unsafe_args(&ctor.params) || ctor.is_unsafe;

    let class_lower = cs.name.to_lowercase();
    let fn_name = test_name(&format!("{}_{}_{}", unit_path, class_lower, "lifecycle"));

    out.push_str(&format!(
        "/// 冒烟测试 A：{class} 类完整生命周期（构造 → 方法调用 → 析构）\n",
        class = cs.name
    ));
    out.push_str("#[test]\n");
    out.push_str("#[ignore = \"Requires runtime environment\"]\n");
    out.push_str(&format!("fn {}() {{\n", fn_name));

    // 构造
    let ctor_call = if ctor_unsafe {
        format!("    let mut obj = unsafe {{ {}({}) }};\n", ctor.rust_name, ctor_args)
    } else {
        format!("    let mut obj = {}({});\n", ctor.rust_name, ctor_args)
    };
    out.push_str(&ctor_call);

    // 方法调用（只处理全基本类型参数的方法）
    for mb in &cs.methods {
        out.push_str(&emit_method_call("    ", &mb));
    }

    // drop
    out.push_str("    drop(obj);\n");
    out.push_str("}\n\n");

    out
}

/// 生成类别 A 的注释桩（ctor 有指针参数，无法自动生成）
fn emit_class_lifecycle_stub(unit_path: &str, cs: &ClassSpec, ctor: &FnBinding) -> String {
    let class_lower = cs.name.to_lowercase();
    let fn_name = test_name(&format!("{}_{}_{}", unit_path, class_lower, "lifecycle"));
    format!(
        "// smoke_{fn}: {class} 构造函数含指针参数，需人工补充测试\n\
         // 构造函数签名：{sig}\n\
         // cpp2rust-todo[SMOKE]: 补充安全的入参后取消注释\n\
         // #[test]\n\
         // #[ignore = \"Requires runtime environment\"]\n\
         // fn {fn}() {{ /* TODO */ }}\n\n",
        fn = fn_name,
        class = cs.name,
        sig = ctor.cpp_sig
    )
}

// ─────────────────────────────────────────────
//  类别 B/C：自由函数
// ─────────────────────────────────────────────

fn emit_free_fn(unit_path: &str, fb: &FnBinding) -> String {
    let mut out = String::new();

    if fb.has_fn_ptr_param {
        // 类别 C：函数指针参数
        return emit_free_fn_stub(unit_path, fb, "含函数指针参数");
    }

    // 检查是否有指针参数（非 *const i8 的通用指针）
    match build_args(&fb.params) {
        ArgsResult::HasPointer => {
            return emit_free_fn_stub(unit_path, fb, "含指针参数");
        }
        ArgsResult::Ok(args) | ArgsResult::NeedsUnsafe(args) => {
            let use_unsafe = needs_unsafe_args(&fb.params) || fb.is_unsafe;
            let fn_name = test_name(&format!("{}_fn_{}", unit_path, fb.rust_name));
            let has_ret = fb.ret_type.is_some();
            let call = format!("{}({})", fb.rust_name, args);

            out.push_str(&format!(
                "/// 冒烟测试 B：自由函数 {}\n",
                fb.rust_name
            ));
            out.push_str("#[test]\n");
            out.push_str("#[ignore = \"Requires runtime environment\"]\n");
            out.push_str(&format!("fn {}() {{\n", fn_name));

            if use_unsafe {
                if has_ret {
                    out.push_str(&format!("    let _ = unsafe {{ {} }};\n", call));
                } else {
                    out.push_str(&format!("    unsafe {{ {} }};\n", call));
                }
            } else if has_ret {
                out.push_str(&format!("    let _ = {};\n", call));
            } else {
                out.push_str(&format!("    {};\n", call));
            }

            out.push_str("}\n\n");
        }
    }

    out
}

fn emit_free_fn_stub(unit_path: &str, fb: &FnBinding, reason: &str) -> String {
    let fn_name = test_name(&format!("{}_fn_{}", unit_path, fb.rust_name));
    format!(
        "// smoke_{fn}: {reason}，需人工补充测试\n\
         // 函数签名：{sig}\n\
         // cpp2rust-todo[SMOKE]: 补充安全的入参后取消注释\n\
         // #[test]\n\
         // #[ignore = \"Requires runtime environment\"]\n\
         // fn {fn}() {{ /* TODO */ }}\n\n",
        fn = fn_name,
        reason = reason,
        sig = fb.cpp_sig
    )
}

// ─────────────────────────────────────────────
//  类别 D：接口类工厂函数
// ─────────────────────────────────────────────

fn emit_interface_via_factory(unit_path: &str, spec: &FfiSpec, cs: &ClassSpec) -> String {
    let mut out = String::new();

    // 工厂函数：lib_spec.fn_bindings 中返回 *mut ClassName 的函数
    let factories: Vec<&FnBinding> = spec
        .lib_spec
        .fn_bindings
        .iter()
        .chain(cs.associated_fns.iter())
        .filter(|fb| {
            fb.ret_type.as_deref() == Some(&format!("*mut {}", cs.name))
        })
        .collect();

    if factories.is_empty() {
        out.push_str(&format!(
            "// smoke_{unit}: 接口类 {class} 无已知工厂函数，需人工补充测试\n\n",
            unit = test_name_segment(unit_path),
            class = cs.name
        ));
        return out;
    }

    for factory in factories {
        let args_result = build_args(&factory.params);
        match args_result {
            ArgsResult::HasPointer => {
                // 工厂函数有指针参数，走注释桩
                let fn_name = test_name(&format!(
                    "{}_{}_via_factory",
                    unit_path,
                    cs.name.to_lowercase()
                ));
                out.push_str(&format!(
                    "// smoke_{fn}: 接口类 {class} 工厂函数含指针参数，需人工补充测试\n\
                     // 工厂函数签名：{sig}\n\
                     // cpp2rust-todo[SMOKE]: 补充安全的入参后取消注释\n\
                     // #[test]\n\
                     // #[ignore = \"Requires runtime environment\"]\n\
                     // fn {fn}() {{ /* TODO */ }}\n\n",
                    fn = fn_name,
                    class = cs.name,
                    sig = factory.cpp_sig,
                ));
            }
            ArgsResult::Ok(args) | ArgsResult::NeedsUnsafe(args) => {
                let fn_name = test_name(&format!(
                    "{}_{}_{}_via_factory",
                    unit_path,
                    cs.name.to_lowercase(),
                    factory.rust_name
                ));
                out.push_str(&format!(
                    "/// 冒烟测试 D：接口类 {class} 工厂函数 {fac}\n",
                    class = cs.name,
                    fac = factory.rust_name
                ));
                out.push_str("#[test]\n");
                out.push_str("#[ignore = \"Requires runtime environment\"]\n");
                out.push_str(&format!("fn {}() {{\n", fn_name));
                out.push_str(&format!(
                    "    let ptr = unsafe {{ {}({}) }};\n",
                    factory.rust_name, args
                ));
                out.push_str(&format!(
                    "    assert!(!ptr.is_null(), \"工厂函数 {} 不应返回 null\");\n",
                    factory.rust_name
                ));
                // 接口类方法（&self 的，参数全为基本类型）
                if !cs.methods.is_empty() {
                    out.push_str("    // 注：接口类通过原始指针调用方法需要 hicc AbiClass 支持\n");
                    out.push_str("    // 可在此补充方法调用验证\n");
                }
                // Safety: ptr 需要在使用完毕后手动释放；此测试不释放属预期行为
                // （目的仅为验证工厂函数能成功分配对象）。
                // 若需避免内存泄漏，请补充对应的 destroy 函数调用（参见 class_specs）。
                out.push_str("    // Safety: ptr 需要手动释放，此测试不释放属预期（验证分配成功）\n");
                out.push_str("    // 补充 destroy 调用示例：unsafe { <ClassName>_delete(ptr) };\n");
                out.push_str("}\n\n");
            }
        }
    }

    out
}

// ─────────────────────────────────────────────
//  方法调用辅助
// ─────────────────────────────────────────────

fn emit_method_call(indent: &str, mb: &MethodBinding) -> String {
    if mb.has_fn_ptr_param {
        return String::new(); // 跳过函数指针参数方法
    }
    let args_result = build_args(&mb.params);
    match args_result {
        ArgsResult::HasPointer => String::new(), // 跳过含指针参数的方法
        ArgsResult::Ok(args) | ArgsResult::NeedsUnsafe(args) => {
            let has_ret = mb.ret_type.is_some();
            let call = format!("obj.{}({})", mb.rust_name, args);
            match (mb.self_kind == SelfKind::Ref, has_ret) {
                (_, true) => format!("{}let _ = {};\n", indent, call),
                (_, false) => format!("{}{};\n", indent, call),
            }
        }
    }
}

// ─────────────────────────────────────────────
//  参数构建
// ─────────────────────────────────────────────

enum ArgsResult {
    /// 全部基本类型，无需 unsafe
    Ok(String),
    /// 全部基本类型，但含需要 unsafe 的 `*const i8` 参数
    NeedsUnsafe(String),
    /// 含无法自动处理的指针参数（跳过，走注释桩）
    HasPointer,
}

/// 为参数列表构建零值参数字符串。
fn build_args(params: &[(String, String)]) -> ArgsResult {
    let mut parts = Vec::new();
    let mut needs_unsafe = false;

    for (_, ty) in params {
        match zero_value_for_type(ty) {
            ZeroValue::Basic(v) => parts.push(v),
            ZeroValue::NeedsUnsafe(v) => {
                needs_unsafe = true;
                parts.push(v);
            }
            ZeroValue::HasPointer => return ArgsResult::HasPointer,
        }
    }

    let joined = parts.join(", ");
    if needs_unsafe {
        ArgsResult::NeedsUnsafe(joined)
    } else {
        ArgsResult::Ok(joined)
    }
}

/// 判断参数列表是否需要 unsafe（含 `*const i8` 类型参数）。
fn needs_unsafe_args(params: &[(String, String)]) -> bool {
    params.iter().any(|(_, ty)| {
        matches!(zero_value_for_type(ty), ZeroValue::NeedsUnsafe(_))
    })
}

// ─────────────────────────────────────────────
//  零值生成
// ─────────────────────────────────────────────

enum ZeroValue {
    /// 纯基本类型零值
    Basic(String),
    /// 需要 unsafe 的零值（如 `*const i8` C 字符串指针）
    NeedsUnsafe(String),
    /// 无法安全生成零值（其他指针、函数指针等）
    HasPointer,
}

/// 为 Rust 类型字符串生成零值。
fn zero_value_for_type(ty: &str) -> ZeroValue {
    let ty = ty.trim();
    match ty {
        "i8" | "i16" | "i32" | "i64" | "isize" => ZeroValue::Basic("0".to_string()),
        "u8" | "u16" | "u32" | "u64" | "usize" => ZeroValue::Basic("0".to_string()),
        "f32" | "f64" => ZeroValue::Basic("0.0".to_string()),
        "bool" => ZeroValue::Basic("false".to_string()),
        // *const i8 → C 字符串，用 null 终止字节字符串指针（需 unsafe 上下文）
        "*const i8" => ZeroValue::NeedsUnsafe(r#"b"\0".as_ptr() as *const i8"#.to_string()),
        _ if ty.starts_with('*') => ZeroValue::HasPointer,
        // 函数指针类型
        _ if ty.contains("fn(") || ty.starts_with("unsafe fn") => ZeroValue::HasPointer,
        // 其他未知类型（class 类型、复杂类型等）
        _ => ZeroValue::HasPointer,
    }
}

// ─────────────────────────────────────────────
//  名称辅助
// ─────────────────────────────────────────────

/// 将 `utils/foo` 形式的 unit_path 转换为 `utils::foo` 模块路径。
fn mod_path_from_unit(unit_path: &str) -> String {
    unit_path.replace('/', "::")
}

/// 将任意字符串转换为合法的 Rust 标识符片段（用于测试函数名）。
/// 非字母数字字符替换为 `_`，收缩连续下划线为单个，去掉首尾下划线。
fn test_name_segment(s: &str) -> String {
    // 单趟字符折叠：非字母数字字符替换为 `_`，连续 `_` 只保留一个
    let mut result = String::with_capacity(s.len());
    for c in s.chars() {
        let ch = if c.is_ascii_alphanumeric() || c == '_' {
            c
        } else {
            '_'
        };
        // 跳过与前一字符重复的下划线
        if ch == '_' && result.ends_with('_') {
            continue;
        }
        result.push(ch);
    }
    result.trim_matches('_').to_string()
}

/// 从完整路径（如 `utils/foo_bar_lifecycle`）生成合法 Rust 测试函数名。
/// 格式为 `smoke_<sanitized_path>`。
fn test_name(path: &str) -> String {
    format!("smoke_{}", test_name_segment(path))
}

// ─────────────────────────────────────────────
//  单元测试
// ─────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ffi_model::{ClassSpec, FfiSpec, FnBinding, LibSpec, MethodBinding, SelfKind};

    fn make_basic_fn(rust_name: &str, params: Vec<(String, String)>, ret: Option<&str>) -> FnBinding {
        FnBinding {
            cpp_sig: format!("void {}()", rust_name),
            rust_name: rust_name.to_string(),
            params,
            ret_type: ret.map(|s| s.to_string()),
            is_unsafe: false,
            has_fn_ptr_param: false,
        }
    }

    fn make_ptr_fn(rust_name: &str) -> FnBinding {
        FnBinding {
            cpp_sig: format!("void* {}(void*)", rust_name),
            rust_name: rust_name.to_string(),
            params: vec![("p".to_string(), "*mut i32".to_string())],
            ret_type: None,
            is_unsafe: true,
            has_fn_ptr_param: false,
        }
    }

    fn make_class_spec(name: &str, with_ctor: bool, with_method: bool) -> ClassSpec {
        let associated_fns = if with_ctor {
            vec![FnBinding {
                cpp_sig: format!("{}* {}_new()", name, name.to_lowercase()),
                rust_name: format!("{}_new", name.to_lowercase()),
                params: vec![],
                ret_type: Some(name.to_string()),
                is_unsafe: false,
                has_fn_ptr_param: false,
            }]
        } else {
            vec![]
        };
        let methods = if with_method {
            vec![MethodBinding {
                cpp_sig: format!("int get() const"),
                rust_name: "get".to_string(),
                self_kind: SelfKind::Ref,
                params: vec![],
                ret_type: Some("i32".to_string()),
                has_fn_ptr_param: false,
            }]
        } else {
            vec![]
        };
        ClassSpec {
            name: name.to_string(),
            methods,
            associated_fns,
            destroy_fn: Some(format!("{}_delete", name.to_lowercase())),
            is_interface: false,
        }
    }

    fn make_spec(unit: &str, cs: ClassSpec, fns: Vec<FnBinding>) -> FfiSpec {
        FfiSpec {
            unit_name: unit.to_string(),
            cpp_block_lines: vec![],
            class_specs: vec![cs],
            lib_spec: LibSpec {
                link_name: unit.to_string(),
                fwd_decls: vec![],
                fn_bindings: fns,
            },
        }
    }

    /// generate 应输出文件头注释
    #[test]
    fn generate_has_file_header() {
        let spec = FfiSpec::default();
        let out = generate(&[("hello", &spec)], "hello_world");
        assert!(out.contains("自动生成的 FFI 冒烟测试"), "缺少文件头注释");
        assert!(out.contains("#![allow(unused_imports"), "缺少 allow 属性");
    }

    /// 每个 unit 应生成一条 use 声明
    #[test]
    fn generate_use_declarations() {
        let spec = FfiSpec::default();
        let out = generate(&[("utils/foo", &spec)], "mylib");
        assert!(out.contains("use mylib::utils::foo::*;"), "缺少 use 声明，实际：\n{}", out);
    }

    /// 类别 B：全基本类型参数的自由函数应生成可调用测试
    #[test]
    fn generate_free_fn_basic_types() {
        let fb = make_basic_fn("add", vec![("a".to_string(), "i32".to_string()), ("b".to_string(), "i32".to_string())], Some("i32"));
        let spec = FfiSpec {
            unit_name: "myunit".to_string(),
            cpp_block_lines: vec![],
            class_specs: vec![],
            lib_spec: LibSpec {
                link_name: "myunit".to_string(),
                fwd_decls: vec![],
                fn_bindings: vec![fb],
            },
        };
        let out = generate(&[("myunit", &spec)], "mylib");
        assert!(out.contains("fn smoke_myunit_fn_add()"), "缺少类别 B 测试，实际：\n{}", out);
        assert!(out.contains("add(0, 0)"), "缺少零值参数调用");
        assert!(out.contains("#[ignore = \"Requires runtime environment\"]"), "缺少 ignore");
    }

    /// 类别 C：含指针参数的自由函数应生成注释桩
    #[test]
    fn generate_free_fn_ptr_becomes_stub() {
        let fb = make_ptr_fn("do_something");
        let spec = FfiSpec {
            unit_name: "myunit".to_string(),
            cpp_block_lines: vec![],
            class_specs: vec![],
            lib_spec: LibSpec {
                link_name: "myunit".to_string(),
                fwd_decls: vec![],
                fn_bindings: vec![fb],
            },
        };
        let out = generate(&[("myunit", &spec)], "mylib");
        assert!(out.contains("cpp2rust-todo[SMOKE]"), "缺少类别 C 注释桩，实际：\n{}", out);
        assert!(!out.contains("#[test]\nfn smoke"), "不应生成可运行测试函数");
    }

    /// 类别 A：有析构函数和构造函数的类应生成生命周期测试
    #[test]
    fn generate_class_lifecycle() {
        let cs = make_class_spec("Counter", true, true);
        let spec = make_spec("counter", cs, vec![]);
        let out = generate(&[("counter", &spec)], "mylib");
        assert!(out.contains("fn smoke_counter_counter_lifecycle()"), "缺少生命周期测试，实际：\n{}", out);
        assert!(out.contains("counter_new()"), "缺少构造函数调用");
        assert!(out.contains("drop(obj)"), "缺少 drop");
    }

    /// mod_path_from_unit 应将 / 替换为 ::
    #[test]
    fn mod_path_conversion() {
        assert_eq!(mod_path_from_unit("utils/foo"), "utils::foo");
        assert_eq!(mod_path_from_unit("foo"), "foo");
        assert_eq!(mod_path_from_unit("a/b/c"), "a::b::c");
    }

    /// test_name_segment 应生成合法 Rust 标识符
    #[test]
    fn test_name_segment_valid() {
        assert_eq!(test_name_segment("hello-world"), "hello_world");
        assert_eq!(test_name_segment("foo/bar"), "foo_bar");
        assert_eq!(test_name_segment("001_class"), "001_class");
    }

    /// zero_value_for_type 应对基本类型返回正确零值
    #[test]
    fn zero_value_basic_types() {
        assert!(matches!(zero_value_for_type("i32"), ZeroValue::Basic(v) if v == "0"));
        assert!(matches!(zero_value_for_type("f64"), ZeroValue::Basic(v) if v == "0.0"));
        assert!(matches!(zero_value_for_type("bool"), ZeroValue::Basic(v) if v == "false"));
        assert!(matches!(zero_value_for_type("*mut i32"), ZeroValue::HasPointer));
    }

    /// *const i8 应生成 NeedsUnsafe 零值
    #[test]
    fn zero_value_const_i8_ptr_needs_unsafe() {
        assert!(matches!(zero_value_for_type("*const i8"), ZeroValue::NeedsUnsafe(_)));
    }

    // ── 新增：含 ctor/dtor 的类生命周期（有方法的完整路径）──────────────────────

    /// 类别 A：含 ctor/dtor 且方法参数全为基本类型 → 生成完整生命周期测试（含方法调用）
    #[test]
    fn generate_class_with_method_call() {
        let cs = make_class_spec("Widget", true, true);
        let spec = make_spec("widget", cs, vec![]);
        let out = generate(&[("widget", &spec)], "mylib");
        // 应生成生命周期测试
        assert!(out.contains("fn smoke_widget_widget_lifecycle()"), "缺少生命周期测试函数名");
        // 应含方法调用
        assert!(out.contains("obj.get()"), "应含方法调用 obj.get()");
    }

    /// 类别 A：无 destroy_fn 的类不生成生命周期测试
    #[test]
    fn generate_class_without_dtor_no_lifecycle() {
        let cs = ClassSpec {
            name: "NoDtor".to_string(),
            methods: vec![],
            associated_fns: vec![],
            destroy_fn: None,
            is_interface: false,
        };
        let spec = make_spec("nodtor", cs, vec![]);
        let out = generate(&[("nodtor", &spec)], "mylib");
        assert!(!out.contains("fn smoke_nodtor_nodtor_lifecycle()"), "无析构函数时不应生成生命周期测试");
    }

    /// 类别 A 注释桩：ctor 含指针参数 → 生成注释桩而非可运行测试
    #[test]
    fn generate_class_ctor_with_ptr_param_becomes_stub() {
        let cs = ClassSpec {
            name: "Complex".to_string(),
            methods: vec![],
            associated_fns: vec![FnBinding {
                cpp_sig: "Complex* complex_new(int* data)".to_string(),
                rust_name: "complex_new".to_string(),
                params: vec![("data".to_string(), "*mut i32".to_string())],
                ret_type: Some("Complex".to_string()),
                is_unsafe: false,
                has_fn_ptr_param: false,
            }],
            destroy_fn: Some("complex_delete".to_string()),
            is_interface: false,
        };
        let spec = make_spec("complex", cs, vec![]);
        let out = generate(&[("complex", &spec)], "mylib");
        // ctor 有指针参数，应生成注释桩
        assert!(out.contains("cpp2rust-todo[SMOKE]"), "ctor 有指针参数时应生成注释桩，实际：\n{}", out);
        // stub 中 #[test] 与 fn 之间有 // 注释行隔开，故 "#[test]\nfn" 不会相邻出现
        let normalized = out.replace("\r\n", "\n");
        assert!(!normalized.contains("#[test]\nfn smoke_complex_complex_lifecycle"), "ctor 有指针参数时不应生成可运行测试");
    }

    // ── 新增：接口类（类别 D）工厂函数测试 ──────────────────────────────────────

    /// 类别 D：接口类有工厂函数 → 生成工厂调用测试
    #[test]
    fn generate_interface_class_with_factory() {
        let factory = FnBinding {
            cpp_sig: "IShape* create_shape()".to_string(),
            rust_name: "create_shape".to_string(),
            params: vec![],
            ret_type: Some("*mut IShape".to_string()),
            is_unsafe: false,
            has_fn_ptr_param: false,
        };
        let cs = ClassSpec {
            name: "IShape".to_string(),
            methods: vec![],
            associated_fns: vec![],
            destroy_fn: None,
            is_interface: true,
        };
        let spec = FfiSpec {
            unit_name: "shapes".to_string(),
            cpp_block_lines: vec![],
            class_specs: vec![cs],
            lib_spec: LibSpec {
                link_name: "shapes".to_string(),
                fwd_decls: vec![],
                fn_bindings: vec![factory],
            },
        };
        let out = generate(&[("shapes", &spec)], "mylib");
        // 接口类测试（类别 D）
        assert!(out.contains("create_shape()"), "应含工厂函数调用，实际：\n{}", out);
        assert!(out.contains("is_null"), "应含 is_null 断言，实际：\n{}", out);
    }

    /// 类别 D：接口类无工厂函数 → 生成注释提示
    #[test]
    fn generate_interface_class_without_factory_gives_comment() {
        let cs = ClassSpec {
            name: "IBase".to_string(),
            methods: vec![],
            associated_fns: vec![],
            destroy_fn: None,
            is_interface: true,
        };
        let spec = FfiSpec {
            unit_name: "base".to_string(),
            cpp_block_lines: vec![],
            class_specs: vec![cs],
            lib_spec: LibSpec {
                link_name: "base".to_string(),
                fwd_decls: vec![],
                fn_bindings: vec![],
            },
        };
        let out = generate(&[("base", &spec)], "mylib");
        assert!(out.contains("无已知工厂函数"), "无工厂函数时应生成注释提示，实际：\n{}", out);
    }

    // ── 新增：含函数指针参数的函数（类别 C）─────────────────────────────────────

    /// 类别 C：has_fn_ptr_param=true 的函数 → 生成注释桩（含函数指针参数提示）
    #[test]
    fn generate_fn_ptr_param_generates_stub() {
        let fb = FnBinding {
            cpp_sig: "void register_callback(void (*cb)(int))".to_string(),
            rust_name: "register_callback".to_string(),
            params: vec![("cb".to_string(), "unsafe extern \"C\" fn(i32)".to_string())],
            ret_type: None,
            is_unsafe: false,
            has_fn_ptr_param: true,
        };
        let spec = FfiSpec {
            unit_name: "callbacks".to_string(),
            cpp_block_lines: vec![],
            class_specs: vec![],
            lib_spec: LibSpec {
                link_name: "callbacks".to_string(),
                fwd_decls: vec![],
                fn_bindings: vec![fb],
            },
        };
        let out = generate(&[("callbacks", &spec)], "mylib");
        assert!(out.contains("含函数指针参数"), "has_fn_ptr_param=true 应生成函数指针注释桩，实际：\n{}", out);
        assert!(out.contains("cpp2rust-todo[SMOKE]"), "应含 SMOKE 占位符，实际：\n{}", out);
        // 不应生成可运行测试（stub 中 #[test] 与 fn 之间有注释行隔开）
        let normalized = out.replace("\r\n", "\n");
        assert!(!normalized.contains("#[test]\nfn smoke_callbacks_fn_register"), "不应生成可运行测试，实际：\n{}", out);
    }

    // ── 新增边界情形测试 ─────────────────────────────────────────────────────

    /// 空 units 切片：生成内容仍含文件头、无 use 声明、无测试函数
    #[test]
    fn generate_empty_units() {
        let out = generate(&[], "mylib");
        assert!(out.contains("自动生成的 FFI 冒烟测试"), "空 units 时仍应有文件头注释");
        assert!(!out.contains("use mylib::"), "空 units 时不应有 use 声明，实际：\n{}", out);
        assert!(!out.contains("#[test]"), "空 units 时不应有测试函数，实际：\n{}", out);
    }

    /// 全指针参数：所有参数均为 *mut T，应只生成注释桩而无 #[test]
    #[test]
    fn generate_all_pointer_params() {
        let fb = FnBinding {
            cpp_sig: "void process(void* buf, int* len)".to_string(),
            rust_name: "process".to_string(),
            params: vec![
                ("buf".to_string(), "*mut u8".to_string()),
                ("len".to_string(), "*mut i32".to_string()),
            ],
            ret_type: None,
            is_unsafe: true,
            has_fn_ptr_param: false,
        };
        let spec = FfiSpec {
            unit_name: "buf_unit".to_string(),
            cpp_block_lines: vec![],
            class_specs: vec![],
            lib_spec: LibSpec {
                link_name: "buf_unit".to_string(),
                fwd_decls: vec![],
                fn_bindings: vec![fb],
            },
        };
        let out = generate(&[("buf_unit", &spec)], "mylib");
        assert!(
            !out.lines().any(|line| line.trim() == "#[test]"),
            "全指针参数时不应生成可运行测试，实际：\n{}",
            out
        );
        assert!(out.contains("cpp2rust-todo[SMOKE]"), "全指针参数时应生成注释桩，实际：\n{}", out);
    }

    /// 零参函数（类别 B）：无参函数应生成 #[test] 和 #[ignore]
    #[test]
    fn generate_no_param_fn_produces_type_b() {
        let fb = make_basic_fn("ping", vec![], None);
        let spec = FfiSpec {
            unit_name: "pinger".to_string(),
            cpp_block_lines: vec![],
            class_specs: vec![],
            lib_spec: LibSpec {
                link_name: "pinger".to_string(),
                fwd_decls: vec![],
                fn_bindings: vec![fb],
            },
        };
        let out = generate(&[("pinger", &spec)], "mylib");
        assert!(out.contains("#[test]"), "无参函数应生成 #[test]，实际：\n{}", out);
        assert!(out.contains("#[ignore"), "无参函数应生成 #[ignore]，实际：\n{}", out);
        assert!(out.contains("ping()"), "应含函数调用，实际：\n{}", out);
    }

    /// 含 *const i8 参数的函数（类别 B NeedsUnsafe）：测试体应用 unsafe {}
    #[test]
    fn generate_string_param_fn_needs_unsafe() {
        let fb = FnBinding {
            cpp_sig: "void log_msg(const char* msg)".to_string(),
            rust_name: "log_msg".to_string(),
            params: vec![("msg".to_string(), "*const i8".to_string())],
            ret_type: None,
            is_unsafe: false,
            has_fn_ptr_param: false,
        };
        let spec = FfiSpec {
            unit_name: "logger".to_string(),
            cpp_block_lines: vec![],
            class_specs: vec![],
            lib_spec: LibSpec {
                link_name: "logger".to_string(),
                fwd_decls: vec![],
                fn_bindings: vec![fb],
            },
        };
        let out = generate(&[("logger", &spec)], "mylib");
        assert!(out.contains("#[test]"), "含 *const i8 参数的函数应生成 #[test]，实际：\n{}", out);
        assert!(out.contains("unsafe {"), "含 *const i8 参数时测试体应用 unsafe {{}}，实际：\n{}", out);
    }

    /// write_smoke_test：在临时目录调用后 tests/smoke_test.rs 存在且内容完整
    #[test]
    fn write_smoke_test_creates_dir_and_file() {
        use crate::generator::project_generator;
        use tempfile::TempDir;

        let tmp = TempDir::new().expect("无法创建临时目录");
        let content = "// 测试内容\nfn dummy() {}\n";
        project_generator::write_smoke_test(tmp.path(), content)
            .expect("write_smoke_test 失败");

        let path = tmp.path().join("tests").join("smoke_test.rs");
        assert!(path.exists(), "tests/smoke_test.rs 应被创建，路径：{}", path.display());

        let written = std::fs::read_to_string(&path).expect("读取文件失败");
        assert_eq!(written, content, "写入内容应与传入内容一致");
    }
}
