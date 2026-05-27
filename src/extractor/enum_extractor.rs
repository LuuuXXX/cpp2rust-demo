use clang::{Entity, EntityKind};

use crate::types::{CppEnum, CppEnumVariant};

pub fn extract_enum(entity: &Entity<'_>, namespace: &[String]) -> Option<CppEnum> {
    let name = entity.get_name().or_else(|| entity.get_display_name())?;
    let mut cpp_enum = CppEnum {
        name,
        namespace: namespace.to_vec(),
        is_scoped: entity
            .get_display_name()
            .unwrap_or_default()
            .contains("enum class"),
        variants: Vec::new(),
    };

    for child in entity.get_children() {
        if child.get_kind() == EntityKind::EnumConstantDecl {
            cpp_enum.variants.push(CppEnumVariant {
                name: child.get_name().unwrap_or_default(),
                value: None,
            });
        }
    }

    Some(cpp_enum)
}
