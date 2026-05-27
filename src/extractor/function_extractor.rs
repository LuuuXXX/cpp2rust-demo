use clang::Entity;

use super::class_extractor::type_from_clang;
use crate::types::{CppFunction, CppParam, CppType};

pub fn extract_function(entity: &Entity<'_>, namespace: &[String]) -> Option<CppFunction> {
    let name = entity.get_name().or_else(|| entity.get_display_name())?;
    if name.is_empty() {
        return None;
    }

    let params = entity
        .get_arguments()
        .unwrap_or_default()
        .into_iter()
        .enumerate()
        .map(|(idx, arg)| CppParam {
            name: arg.get_name().unwrap_or_else(|| format!("arg{idx}")),
            type_: arg
                .get_type()
                .map(type_from_clang)
                .unwrap_or_else(|| CppType::new("void")),
            default_value: None,
        })
        .collect::<Vec<_>>();

    let display_name = entity.get_display_name().unwrap_or_default();
    Some(CppFunction {
        name,
        namespace: namespace.to_vec(),
        return_type: entity
            .get_result_type()
            .map(type_from_clang)
            .unwrap_or_else(|| CppType::new("void")),
        params,
        is_inline: display_name.contains(" inline ") || display_name.starts_with("inline "),
        is_variadic: display_name.contains("..."),
        is_extern_c: display_name.contains("extern \"C\""),
        body: None,
    })
}
