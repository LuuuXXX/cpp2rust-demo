//! `@make_proxy` 代理工厂骨架构建（v6 Phase C：高级映射）
//!
//! 从「继承 C++ 抽象接口（纯虚类）的具体类」的公有构造函数派生 hicc `@make_proxy`
//! 代理工厂骨架，使 Rust 侧可通过组合模式实现 C++ 抽象类
//! （见 `references/hicc/examples/interface`）。
//!
//! 与 v6 模板能力一致：提取器**始终**构建规格（开销极小），是否输出由生成器侧的
//! `CPP2RUST_GEN_PROXY` 开关裁决，默认关闭时不影响产物。

use super::class_spec::is_interface_class;
use super::type_mapper::{clean_type, cpp_to_rust, to_snake_case};
use super::{normalize_ptr_spacing, sanitize_param_name, strip_volatile};
use crate::ast_parser::{ClassInfo, MethodInfo};
use crate::ffi_model::ProxyFactorySpec;

/// 由 `CppAst` 构建 `@make_proxy` 代理工厂骨架列表。
///
/// 仅纳入来自当前编译单元（`is_from_current_file`）、非抽象（可实例化）且**继承自某个
/// 接口基类**的具体类；对其每个公有构造函数（排除拷贝 / 移动构造）派生一个工厂规格。
pub fn build_proxy_factories(ast: &crate::ast_parser::CppAst) -> Vec<ProxyFactorySpec> {
    let mut out: Vec<ProxyFactorySpec> = Vec::new();

    for ci in &ast.classes {
        if !ci.is_from_current_file || ci.name.is_empty() {
            continue;
        }
        // 抽象类本身不可实例化，无法作为 @make_proxy 的目标具体类
        if ci.is_abstract {
            continue;
        }
        // 该具体类本身若是纯虚接口，跳过（接口走 #[interface]，不生成代理工厂）
        if is_interface_class(ci, &ast.classes) {
            continue;
        }

        // 寻找直接接口基类（按声明顺序取第一个为接口的基类）
        let interface_name = match find_interface_base(ci, &ast.classes) {
            Some(name) => name,
            None => continue,
        };

        // 收集公有构造函数（排除拷贝 / 移动构造）
        let ctors: Vec<&MethodInfo> = ci
            .methods
            .iter()
            .filter(|m| m.is_constructor && m.accessibility == "public")
            .filter(|m| !is_copy_or_move_ctor(m, &ci.name))
            .collect();
        if ctors.is_empty() {
            continue;
        }

        let multiple = ctors.len() > 1;
        for (idx, ctor) in ctors.iter().enumerate() {
            out.push(build_one(&ci.name, &interface_name, ctor, idx, multiple));
        }
    }

    out
}

/// 在 `ci` 的直接基类中寻找第一个「纯虚接口」基类名（清理后）。
fn find_interface_base(ci: &ClassInfo, all_classes: &[ClassInfo]) -> Option<String> {
    for base in &ci.bases {
        let base_name = clean_type(&base.name).to_string();
        if let Some(base_ci) = all_classes.iter().find(|c| c.name == base_name) {
            if is_interface_class(base_ci, all_classes) {
                return Some(base_name);
            }
        }
    }
    None
}

/// 判断构造函数是否为拷贝 / 移动构造（唯一参数为本类的引用，如 `const Foo&` / `Foo&&`）。
fn is_copy_or_move_ctor(ctor: &MethodInfo, class_name: &str) -> bool {
    if ctor.params.len() != 1 {
        return false;
    }
    let ty = clean_type(&ctor.params[0].type_name);
    // 去除引用 / cv 限定后比较核心类型名
    let core = ty
        .trim_end_matches('&')
        .trim()
        .trim_start_matches("const ")
        .trim_start_matches("volatile ")
        .trim();
    (ty.contains('&')) && core == class_name
}

/// 由单个构造函数派生一个代理工厂规格。
fn build_one(
    class_name: &str,
    interface_name: &str,
    ctor: &MethodInfo,
    idx: usize,
    multiple: bool,
) -> ProxyFactorySpec {
    let base = format!("new_rust_{}", to_snake_case(class_name));
    let rust_name = if multiple {
        format!("{}_{}", base, idx)
    } else {
        base
    };

    // C++ 侧 @make_proxy 的参数类型列表（仅类型，不含名字），与构造函数一致
    let cpp_param_types: Vec<String> = ctor
        .params
        .iter()
        .map(|p| normalize_ptr_spacing(strip_volatile(clean_type(&p.type_name))))
        .collect();
    let cpp_sig = format!(
        "{} @make_proxy<{}>({})",
        class_name,
        class_name,
        cpp_param_types.join(", ")
    );

    // Rust 侧参数（位于 intf: hicc::Interface<...> 之后）
    let params: Vec<(String, String)> = ctor
        .params
        .iter()
        .enumerate()
        .map(|(i, p)| (sanitize_param_name(&p.name, i), cpp_to_rust(&p.type_name)))
        .collect();

    ProxyFactorySpec {
        rust_name,
        concrete_class: class_name.to_string(),
        interface_name: interface_name.to_string(),
        cpp_sig,
        params,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast_parser::{BaseInfo, ClassInfo, CppAst, MethodInfo, ParamInfo};
    use std::path::PathBuf;

    fn pure_virtual_method(name: &str) -> MethodInfo {
        MethodInfo {
            name: name.to_string(),
            return_type: "void".to_string(),
            params: vec![],
            is_const: true,
            is_volatile: false,
            is_virtual: true,
            is_pure_virtual: true,
            is_static: false,
            is_constructor: false,
            is_destructor: false,
            is_inline: false,
            accessibility: "public".to_string(),
            body_offset: None,
            is_override: false,
            is_default: false,
        }
    }

    fn ctor(params: Vec<ParamInfo>) -> MethodInfo {
        MethodInfo {
            name: "Baz".to_string(),
            return_type: String::new(),
            params,
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
        }
    }

    fn param(ty: &str, name: &str) -> ParamInfo {
        ParamInfo {
            name: name.to_string(),
            type_name: ty.to_string(),
            has_default: false,
        }
    }

    fn class(
        name: &str,
        is_abstract: bool,
        bases: Vec<&str>,
        methods: Vec<MethodInfo>,
    ) -> ClassInfo {
        ClassInfo {
            name: name.to_string(),
            is_struct: false,
            is_abstract,
            template_args: vec![],
            bases: bases
                .into_iter()
                .map(|b| BaseInfo {
                    name: b.to_string(),
                    is_virtual: false,
                })
                .collect(),
            methods,
            fields: vec![],
            is_in_namespace: false,
            is_from_current_file: true,
        }
    }

    fn ast_with(classes: Vec<ClassInfo>) -> CppAst {
        CppAst {
            file: PathBuf::from("test.cpp2rust"),
            classes,
            functions: vec![],
            enums: vec![],
            typedefs: vec![],
            template_class_ranges: vec![],
            template_classes: vec![],
            template_functions: vec![],
            local_var_types: vec![],
        }
    }

    /// 继承接口的具体类应派生默认构造的代理工厂骨架
    #[test]
    fn derives_proxy_factory_for_concrete_class_with_interface_base() {
        let foo = class("Foo", true, vec![], vec![pure_virtual_method("foo")]);
        let baz = class("Baz", false, vec!["Foo"], vec![ctor(vec![])]);
        let ast = ast_with(vec![foo, baz]);
        let fps = build_proxy_factories(&ast);
        assert_eq!(fps.len(), 1, "应派生 1 个代理工厂，实际：{:?}", fps);
        let pf = &fps[0];
        assert_eq!(pf.rust_name, "new_rust_baz");
        assert_eq!(pf.concrete_class, "Baz");
        assert_eq!(pf.interface_name, "Foo");
        assert_eq!(pf.cpp_sig, "Baz @make_proxy<Baz>()");
        assert!(pf.params.is_empty());
    }

    /// 构造函数参数应映射到 Rust 类型并保留在 @make_proxy 签名的类型列表中
    #[test]
    fn maps_ctor_params() {
        let foo = class("Foo", true, vec![], vec![pure_virtual_method("foo")]);
        let baz = class(
            "Baz",
            false,
            vec!["Foo"],
            vec![ctor(vec![param("int", "value")])],
        );
        let ast = ast_with(vec![foo, baz]);
        let fps = build_proxy_factories(&ast);
        assert_eq!(fps.len(), 1);
        assert_eq!(fps[0].cpp_sig, "Baz @make_proxy<Baz>(int)");
        assert_eq!(
            fps[0].params,
            vec![("value".to_string(), "i32".to_string())]
        );
    }

    /// 不继承任何接口的具体类不应派生代理工厂
    #[test]
    fn skips_class_without_interface_base() {
        let plain = class("Plain", false, vec![], vec![ctor(vec![])]);
        let ast = ast_with(vec![plain]);
        assert!(build_proxy_factories(&ast).is_empty());
    }

    /// 接口类自身（纯虚）不应派生代理工厂
    #[test]
    fn skips_interface_itself() {
        let foo = class("Foo", true, vec![], vec![pure_virtual_method("foo")]);
        let ast = ast_with(vec![foo]);
        assert!(build_proxy_factories(&ast).is_empty());
    }

    /// 拷贝构造函数应被排除，多个构造函数应追加序号后缀
    #[test]
    fn excludes_copy_ctor_and_indexes_multiple() {
        let foo = class("Foo", true, vec![], vec![pure_virtual_method("foo")]);
        let baz = class(
            "Baz",
            false,
            vec!["Foo"],
            vec![
                ctor(vec![]),
                ctor(vec![param("int", "n")]),
                ctor(vec![param("const Baz &", "other")]), // 拷贝构造，应排除
            ],
        );
        let ast = ast_with(vec![foo, baz]);
        let fps = build_proxy_factories(&ast);
        assert_eq!(fps.len(), 2, "拷贝构造应被排除，实际：{:?}", fps);
        assert_eq!(fps[0].rust_name, "new_rust_baz_0");
        assert_eq!(fps[1].rust_name, "new_rust_baz_1");
    }
}
