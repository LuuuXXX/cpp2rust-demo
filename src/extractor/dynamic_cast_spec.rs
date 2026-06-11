//! `@dynamic_cast` 下行转换绑定骨架构建（v6 Phase C（续）：高级映射）
//!
//! 从「继承多态基类（含虚函数）的派生类」派生 hicc `@dynamic_cast` 下行转换骨架，
//! 用于 RTTI 场景把多态基类指针安全地向下转换为派生类指针
//! （见 `references/hicc/examples/dynamic_cast`），替代 v5 的整数枚举绕过方案
//! （对应 v6 方案 §3.2 示例 023 typeid_rtti）。
//!
//! 与 v6 其余高级映射能力一致：提取器**始终**构建规格（开销极小），是否输出由生成器侧的
//! `CPP2RUST_GEN_DYNAMIC_CAST` 开关裁决，默认关闭时不影响产物。

use super::type_mapper::{clean_type, to_snake_case};
use crate::ast_parser::ClassInfo;
use crate::ffi_model::DynamicCastSpec;

/// 由 `CppAst` 构建 `@dynamic_cast` 下行转换骨架列表。
///
/// 对当前编译单元（`is_from_current_file`）中**继承自某个多态基类**的派生类，针对每个
/// `(多态基类, 派生类)` 关系派生一个下行转换骨架。基类需为「多态类」（自身或其祖先含
/// 虚函数，包括虚析构），否则 C++ 的 `dynamic_cast` 无法编译。
///
/// v6 Phase C（收尾）：除**直接基类**外，还遍历**递归祖先链**中的所有多态祖先，为
/// 「跨层（间接）继承」关系派生下行转换骨架（如 `Foo <- Bar <- Baz` 额外派生 `Foo → Baz`）。
/// 这与 C++ 允许 `dynamic_cast` 跨任意层级向下转换的语义一致。结果按 `(src, dst)` 去重。
pub fn build_dynamic_casts(ast: &crate::ast_parser::CppAst) -> Vec<DynamicCastSpec> {
    let mut out: Vec<DynamicCastSpec> = Vec::new();
    let mut seen: std::collections::HashSet<(String, String)> = std::collections::HashSet::new();

    for ci in &ast.classes {
        if !ci.is_from_current_file || ci.name.is_empty() {
            continue;
        }
        // 收集（递归）所有祖先，针对每个「多态祖先」生成 祖先 → 派生类 的下行转换骨架。
        // 直接基类与间接（跨层）祖先一并覆盖。
        let mut ancestors: Vec<String> = Vec::new();
        let mut visited: std::collections::HashSet<String> = std::collections::HashSet::new();
        collect_ancestors(ci, &ast.classes, &mut ancestors, &mut visited);

        for ancestor_name in ancestors {
            if ancestor_name == ci.name {
                continue;
            }
            let ancestor_ci = match ast.classes.iter().find(|c| c.name == ancestor_name) {
                Some(c) => c,
                None => continue,
            };
            if !is_polymorphic(ancestor_ci, &ast.classes) {
                continue;
            }
            if seen.insert((ancestor_name.clone(), ci.name.clone())) {
                out.push(build_one(&ancestor_name, &ci.name));
            }
        }
    }

    out
}

/// 递归收集 `ci` 的所有祖先类名（直接基类 + 间接祖先），用于跨层下行转换。
/// `visited` 防止循环 / 重复继承导致的无限递归与重复收集。
fn collect_ancestors(
    ci: &ClassInfo,
    all_classes: &[ClassInfo],
    out: &mut Vec<String>,
    visited: &mut std::collections::HashSet<String>,
) {
    for base in &ci.bases {
        let base_name = clean_type(&base.name).to_string();
        if base_name.is_empty() || base_name == ci.name {
            continue;
        }
        if !visited.insert(base_name.clone()) {
            continue;
        }
        out.push(base_name.clone());
        if let Some(base_ci) = all_classes.iter().find(|c| c.name == base_name) {
            collect_ancestors(base_ci, all_classes, out, visited);
        }
    }
}

/// 判断一个类是否为「多态类」：自身或任一（递归）基类含虚函数（包括虚析构 / 纯虚）。
fn is_polymorphic(ci: &ClassInfo, all_classes: &[ClassInfo]) -> bool {
    if ci.methods.iter().any(|m| m.is_virtual || m.is_pure_virtual) {
        return true;
    }
    for base in &ci.bases {
        let base_name = clean_type(&base.name).to_string();
        if base_name == ci.name {
            continue;
        }
        if let Some(base_ci) = all_classes.iter().find(|c| c.name == base_name) {
            if is_polymorphic(base_ci, all_classes) {
                return true;
            }
        }
    }
    false
}

/// 由 `(基类, 派生类)` 派生一个下行转换规格。
fn build_one(base: &str, derived: &str) -> DynamicCastSpec {
    let rust_name = format!(
        "dynamic_cast_{}_to_{}",
        to_snake_case(base),
        to_snake_case(derived)
    );
    let cpp_sig = format!(
        "const {derived}* @dynamic_cast<const {derived}*>(const {base}*)",
        derived = derived,
        base = base
    );
    DynamicCastSpec {
        rust_name,
        src_class: base.to_string(),
        dst_class: derived.to_string(),
        cpp_sig,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast_parser::{BaseInfo, ClassInfo, CppAst, MethodInfo};
    use std::path::PathBuf;

    fn virtual_method(name: &str, pure: bool) -> MethodInfo {
        MethodInfo {
            name: name.to_string(),
            return_type: "void".to_string(),
            params: vec![],
            is_const: true,
            is_volatile: false,
            is_virtual: true,
            is_pure_virtual: pure,
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

    fn class(name: &str, bases: Vec<&str>, methods: Vec<MethodInfo>) -> ClassInfo {
        ClassInfo {
            name: name.to_string(),
            is_struct: false,
            is_abstract: false,
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

    /// 继承多态基类的派生类应派生下行转换骨架
    #[test]
    fn derives_downcast_for_polymorphic_base() {
        let foo = class("Foo", vec![], vec![virtual_method("foo", true)]);
        let bar = class("Bar", vec!["Foo"], vec![virtual_method("foo", false)]);
        let ast = ast_with(vec![foo, bar]);
        let casts = build_dynamic_casts(&ast);
        assert_eq!(casts.len(), 1, "应派生 1 个下行转换，实际：{:?}", casts);
        let dc = &casts[0];
        assert_eq!(dc.rust_name, "dynamic_cast_foo_to_bar");
        assert_eq!(dc.src_class, "Foo");
        assert_eq!(dc.dst_class, "Bar");
        assert_eq!(
            dc.cpp_sig,
            "const Bar* @dynamic_cast<const Bar*>(const Foo*)"
        );
    }

    /// 非多态基类（无虚函数）不应派生下行转换
    #[test]
    fn skips_non_polymorphic_base() {
        let base = class("Base", vec![], vec![]);
        let derived = class("Derived", vec!["Base"], vec![]);
        let ast = ast_with(vec![base, derived]);
        assert!(build_dynamic_casts(&ast).is_empty());
    }

    /// 多层继承时，多态性应沿基类链传递；并应额外派生跨层（间接）下行转换
    #[test]
    fn polymorphism_propagates_through_base_chain() {
        let foo = class("Foo", vec![], vec![virtual_method("foo", true)]);
        let bar = class("Bar", vec!["Foo"], vec![virtual_method("foo", false)]);
        // Baz 直接基类 Bar 本身无新虚函数但继承自多态 Foo，仍应被视为多态
        let baz = class("Baz", vec!["Bar"], vec![]);
        let ast = ast_with(vec![foo, bar, baz]);
        let casts = build_dynamic_casts(&ast);
        let names: Vec<&str> = casts.iter().map(|c| c.rust_name.as_str()).collect();
        assert!(names.contains(&"dynamic_cast_foo_to_bar"));
        assert!(names.contains(&"dynamic_cast_bar_to_baz"));
        // 跨层：Foo 是 Baz 的间接（多态）祖先，应额外派生 Foo → Baz 下行转换
        assert!(
            names.contains(&"dynamic_cast_foo_to_baz"),
            "应派生跨层下行转换 Foo → Baz，实际：{:?}",
            names
        );
    }

    /// 跨层下行转换：直接基类非多态、但间接祖先多态时，仍应对多态祖先派生下行转换，
    /// 且仅对多态祖先派生（非多态的中间基类不产出）
    #[test]
    fn derives_cross_layer_downcast_skips_non_polymorphic_ancestor() {
        // Poly（多态） <- Mid（非多态中间类） <- Leaf
        let poly = class("Poly", vec![], vec![virtual_method("v", true)]);
        let mid = class("Mid", vec!["Poly"], vec![]);
        let leaf = class("Leaf", vec!["Mid"], vec![]);
        let ast = ast_with(vec![poly, mid, leaf]);
        let casts = build_dynamic_casts(&ast);
        let names: Vec<&str> = casts.iter().map(|c| c.rust_name.as_str()).collect();
        // Mid 继承多态 Poly，故 Mid 也是多态 → Poly→Mid、Poly→Leaf、Mid→Leaf 均应派生
        assert!(names.contains(&"dynamic_cast_poly_to_mid"));
        assert!(names.contains(&"dynamic_cast_poly_to_leaf"));
        assert!(names.contains(&"dynamic_cast_mid_to_leaf"));
    }

    /// 跨层下行转换应按 (src, dst) 去重（菱形 / 重复继承不产出重复骨架）
    #[test]
    fn dedups_cross_layer_downcasts() {
        // Base（多态） <- L、R；Diamond 同时继承 L、R（均经 Base）
        let base = class("Base", vec![], vec![virtual_method("v", true)]);
        let l = class("L", vec!["Base"], vec![]);
        let r = class("R", vec!["Base"], vec![]);
        let diamond = class("Diamond", vec!["L", "R"], vec![]);
        let ast = ast_with(vec![base, l, r, diamond]);
        let casts = build_dynamic_casts(&ast);
        let base_to_diamond: Vec<_> = casts
            .iter()
            .filter(|c| c.rust_name == "dynamic_cast_base_to_diamond")
            .collect();
        assert_eq!(
            base_to_diamond.len(),
            1,
            "Base → Diamond 下行转换应去重为 1 个，实际：{:?}",
            casts
        );
    }

    /// 基类不在当前编译单元（未知类型）时跳过
    #[test]
    fn skips_unknown_base() {
        let derived = class("Derived", vec!["Unknown"], vec![virtual_method("f", false)]);
        let ast = ast_with(vec![derived]);
        assert!(build_dynamic_casts(&ast).is_empty());
    }
}
