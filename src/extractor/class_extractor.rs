use clang::{Entity, EntityKind};

use crate::instantiation_tracker::parse_template_instantiation;
use crate::types::{
    CppClass, CppConstructor, CppMethod, CppParam, CppStaticMember, CppType, OperatorKind,
};

pub fn extract_class(entity: &Entity<'_>, namespace: &[String]) -> Option<CppClass> {
    let name = entity.get_name().or_else(|| entity.get_display_name())?;
    if name.is_empty() {
        return None;
    }

    let mut class = CppClass {
        name: strip_template_suffix(&name),
        namespace: namespace.to_vec(),
        ..CppClass::default()
    };

    if let Some(instantiation) = parse_template_instantiation(&name) {
        class.is_template_specialization = true;
        class.template_args = instantiation.type_args;
    }

    for child in entity.get_children() {
        match child.get_kind() {
            EntityKind::Constructor => {
                class.constructors.push(extract_constructor(&child));
            }
            EntityKind::Destructor => {
                class.has_destructor = true;
            }
            EntityKind::Method | EntityKind::ConversionFunction => {
                let method = extract_method(&child);
                if method.is_pure_virtual {
                    class.is_abstract = true;
                }
                class.methods.push(method);
            }
            EntityKind::BaseSpecifier => {
                if let Some(name) = child.get_display_name().or_else(|| child.get_name()) {
                    class.bases.push(name);
                }
            }
            EntityKind::VarDecl | EntityKind::FieldDecl => {
                if let Some(name) = child.get_name() {
                    let type_ = child
                        .get_type()
                        .map(type_from_clang)
                        .unwrap_or_else(|| CppType::new("void"));
                    class.static_members.push(CppStaticMember { name, type_ });
                }
            }
            _ => {}
        }
    }

    Some(class)
}

fn extract_constructor(entity: &Entity<'_>) -> CppConstructor {
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
    CppConstructor {
        is_copy: params.len() == 1 && params[0].type_.is_reference,
        is_move: params.len() == 1 && params[0].type_.is_rvalue_ref,
        is_explicit: display_name.contains("explicit "),
        params,
    }
}

fn extract_method(entity: &Entity<'_>) -> CppMethod {
    let name = entity
        .get_name()
        .or_else(|| entity.get_display_name())
        .unwrap_or_default();
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

    CppMethod {
        name: name.clone(),
        return_type: entity
            .get_result_type()
            .map(type_from_clang)
            .unwrap_or_else(|| CppType::new("void")),
        params,
        is_const: entity.is_const_method(),
        is_virtual: entity.is_virtual_method(),
        is_pure_virtual: entity.is_pure_virtual_method(),
        is_override: entity
            .get_display_name()
            .unwrap_or_default()
            .contains("override"),
        is_static: entity.is_static_method(),
        is_volatile: entity
            .get_display_name()
            .unwrap_or_default()
            .contains("volatile"),
        is_friend: entity
            .get_display_name()
            .unwrap_or_default()
            .contains("friend "),
        operator_kind: OperatorKind::from_cpp_name(&name),
    }
}

pub(crate) fn type_from_clang(ty: clang::Type<'_>) -> CppType {
    let display = ty.get_display_name();
    let mut cpp_type = CppType::new(display.as_str());
    cpp_type.is_const = ty.is_const_qualified();
    cpp_type.is_pointer = display.contains('*');
    cpp_type.is_reference = display.contains('&') && !display.contains("&&");
    cpp_type.is_rvalue_ref = display.contains("&&");
    cpp_type.name = display
        .replace("const", "")
        .replace(['&', '*'], "")
        .trim()
        .to_string();
    cpp_type
}

fn strip_template_suffix(name: &str) -> String {
    if let Some((prefix, _)) = name.split_once('<') {
        prefix.trim().to_string()
    } else {
        name.trim().to_string()
    }
}
