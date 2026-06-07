//! 菱形继承处理器（Phase 2）
//!
//! 检测菱形虚继承结构，将菱形基类方法从 `import_class!` 中移除，
//! 并生成独立的 snake_case shim 函数插入 `cpp!` 块和 `import_lib!`。

use crate::ast_parser::{ClassInfo, CppAst, FunctionInfo};
use crate::extractor::type_mapper::{clean_type, cpp_to_rust, to_snake_case};
use crate::ffi_model::{FfiSpec, FnBinding};
use std::collections::HashSet;

/// 对每个类检测菱形虚基类，生成 shim 并修正 FfiSpec。
pub fn apply(spec: &mut FfiSpec, ast: &CppAst, functions: &[&FunctionInfo]) {
    let class_names: Vec<&str> = ast.classes.iter().map(|c| c.name.as_str()).collect();

    for ci in &ast.classes {
        apply_class(spec, ast, ci, functions, &class_names);
    }
}

fn apply_class(
    spec: &mut FfiSpec,
    ast: &CppAst,
    ci: &ClassInfo,
    functions: &[&FunctionInfo],
    _class_names: &[&str],
) {
    let diamond_bases = find_diamond_bases(ci, &ast.classes);
    if diamond_bases.is_empty() {
        return;
    }

    let cn_lower = to_snake_case(&ci.name);

    // 收集所有菱形基类的 public 非 ctor/dtor 非 static 方法名
    let mut diamond_method_names: Vec<String> = Vec::new();
    for base_name in &diamond_bases {
        if let Some(base_ci) = ast.classes.iter().find(|c| c.name == *base_name) {
            for m in &base_ci.methods {
                if !m.is_constructor
                    && !m.is_destructor
                    && m.accessibility == "public"
                    && !m.is_static
                    && !m.name.starts_with("operator")
                    && !diamond_method_names.contains(&m.name)
                {
                    diamond_method_names.push(m.name.clone());
                }
            }
        }
    }

    if diamond_method_names.is_empty() {
        return;
    }

    let mut cpp_shim_lines: Vec<String> = Vec::new();
    let mut new_bindings: Vec<FnBinding> = Vec::new();
    let mut diamond_snake_names: HashSet<String> = HashSet::new();

    for method_name in &diamond_method_names {
        // 对应的 MethodAccessor 名称（camelCase，如 d_getAValue）
        let accessor_name = format!("{}_{}", cn_lower, method_name);

        // 在已去重的函数列表中查找该 MethodAccessor
        let accessor = functions.iter().find(|fi| fi.name == accessor_name);
        let ret_cpp = if let Some(fi) = accessor {
            clean_type(&fi.return_type).to_string()
        } else {
            // 从基类方法获取返回类型
            let m_ret = find_method_return_type(method_name, &diamond_bases, &ast.classes);
            m_ret.unwrap_or_else(|| "int".to_string())
        };

        // 生成 snake_case shim 名（如 d_get_a_value）
        let shim_name = to_snake_case(&accessor_name);
        let rust_ret = cpp_to_rust(&ret_cpp);

        // 生成 cpp! 块中的 shim 函数
        cpp_shim_lines.push(format!("{} {}({}* self) {{", ret_cpp, shim_name, ci.name));
        cpp_shim_lines.push(format!("    return self->{}();", method_name));
        cpp_shim_lines.push("}".to_string());

        // 生成 FnBinding
        let ret_type = if rust_ret.is_empty() || rust_ret == "void" {
            None
        } else {
            Some(rust_ret)
        };
        let cpp_sig = format!("{} {}({}*)", ret_cpp, shim_name, ci.name);
        new_bindings.push(FnBinding {
            cpp_sig,
            rust_name: shim_name,
            params: vec![("self_".to_string(), format!("*mut {}", ci.name))],
            ret_type,
            is_unsafe: false, // 菱形 shim 遵循 golden 规则：不标记 unsafe
            has_fn_ptr_param: false,
        });

        diamond_snake_names.insert(to_snake_case(method_name));
    }

    if cpp_shim_lines.is_empty() {
        return;
    }

    // 在 cpp_block_lines 中找到 ctor（{ClassName}* {cn_lower}_new(）的位置，将 shim 插入其前
    let ctor_pos = find_ctor_line_pos(&spec.cpp_block_lines, &ci.name, &cn_lower);
    let mut insert_idx = ctor_pos;
    // 若 ctor_pos 前有空行，在空行前插入（保持空行分隔）
    for line in &cpp_shim_lines {
        spec.cpp_block_lines.insert(insert_idx, line.clone());
        insert_idx += 1;
    }
    // 在 shim 和 ctor 之间插入空行
    spec.cpp_block_lines.insert(insert_idx, String::new());

    // 在 lib_spec.fn_bindings 中找到 ctor binding 位置，将新 binding 插入其前
    let ctor_binding_pos = find_ctor_binding_pos(&spec.lib_spec.fn_bindings, &cn_lower);
    for (i, fb) in new_bindings.into_iter().enumerate() {
        spec.lib_spec.fn_bindings.insert(ctor_binding_pos + i, fb);
    }

    // 从 ClassSpec 中删除菱形基类方法（按 rust_name 即 snake_case 匹配）
    if let Some(cs) = spec.class_specs.iter_mut().find(|cs| cs.name == ci.name) {
        cs.methods
            .retain(|mb| !diamond_snake_names.contains(&mb.rust_name));
    }
}

/// 收集类祖先（包含自身）的集合
fn collect_ancestors(class_name: &str, all_classes: &[ClassInfo], visited: &mut HashSet<String>) {
    if visited.contains(class_name) {
        return;
    }
    visited.insert(class_name.to_string());
    if let Some(ci) = all_classes.iter().find(|c| c.name == class_name) {
        for base in &ci.bases {
            let base_name = clean_type(&base.name).to_string();
            collect_ancestors(&base_name, all_classes, visited);
        }
    }
}

/// 查找菱形基类：在 2 条及以上路径中均能到达的基类
fn find_diamond_bases(ci: &ClassInfo, all_classes: &[ClassInfo]) -> HashSet<String> {
    if ci.bases.len() < 2 {
        return HashSet::new();
    }

    // 每个直接基类的祖先集（不含直接基类自身，只含其祖先）
    let mut base_ancestor_sets: Vec<HashSet<String>> = Vec::new();
    for base in &ci.bases {
        let base_name = clean_type(&base.name).to_string();
        let mut ancestors = HashSet::new();
        collect_ancestors(&base_name, all_classes, &mut ancestors);
        base_ancestor_sets.push(ancestors);
    }

    // 出现在 2+ 个集合中的类即为菱形基类
    let mut diamond: HashSet<String> = HashSet::new();
    for i in 0..base_ancestor_sets.len() {
        for j in (i + 1)..base_ancestor_sets.len() {
            for name in base_ancestor_sets[i].intersection(&base_ancestor_sets[j]) {
                diamond.insert(name.clone());
            }
        }
    }
    diamond
}

/// 从菱形基类的方法列表中查找指定方法的返回类型
fn find_method_return_type(
    method_name: &str,
    diamond_bases: &HashSet<String>,
    all_classes: &[ClassInfo],
) -> Option<String> {
    for base_name in diamond_bases {
        if let Some(base_ci) = all_classes.iter().find(|c| c.name == *base_name) {
            for m in &base_ci.methods {
                if m.name == method_name {
                    return Some(clean_type(&m.return_type).to_string());
                }
            }
        }
    }
    None
}

/// 在 cpp_block_lines 中查找 ctor 函数签名行的位置
fn find_ctor_line_pos(cpp_block_lines: &[String], class_name: &str, cn_lower: &str) -> usize {
    let ctor_prefix1 = format!("{}*", class_name);
    let ctor_prefix2 = format!("{} *", class_name);
    let ctor_name = format!("{}_new(", cn_lower);

    for (i, line) in cpp_block_lines.iter().enumerate() {
        let t = line.trim();
        if (t.starts_with(&ctor_prefix1) || t.starts_with(&ctor_prefix2)) && t.contains(&ctor_name)
        {
            return i;
        }
    }
    cpp_block_lines.len()
}

/// 在 fn_bindings 中查找 ctor binding 的位置
fn find_ctor_binding_pos(fn_bindings: &[FnBinding], cn_lower: &str) -> usize {
    let ctor_name = format!("{}_new", cn_lower);
    for (i, fb) in fn_bindings.iter().enumerate() {
        if fb.rust_name == ctor_name {
            return i;
        }
    }
    fn_bindings.len()
}
