use std::path::Path;

use anyhow::{Context, Result};
use clang::{Clang, Entity, EntityKind, Index};

use crate::extractor::{extract_class, extract_enum, extract_function};
use crate::instantiation_tracker::extract_template_instantiation;
use crate::postprocessor::postprocess_ast;
use crate::types::CppAst;

pub fn parse_preprocessed(file: &Path) -> Result<CppAst> {
    let clang = Clang::new().map_err(anyhow::Error::msg)?;
    let index = Index::new(&clang, false, false);
    let arguments = ["-xc++", "-std=c++20"];
    let translation_unit = index
        .parser(file)
        .arguments(&arguments)
        .parse()
        .with_context(|| format!("failed to parse {}", file.display()))?;

    let mut ast = CppAst {
        source_file: file.display().to_string(),
        ..CppAst::default()
    };
    let mut namespace_stack = Vec::new();
    visit_entity(
        &translation_unit.get_entity(),
        &mut namespace_stack,
        &mut ast,
    );
    postprocess_ast(&mut ast);
    ast.namespaces.sort();
    ast.namespaces.dedup();
    Ok(ast)
}

fn visit_entity(entity: &Entity<'_>, namespace_stack: &mut Vec<String>, ast: &mut CppAst) {
    for child in entity.get_children() {
        match child.get_kind() {
            EntityKind::Namespace => {
                let Some(name) = child.get_name() else {
                    continue;
                };
                namespace_stack.push(name);
                ast.namespaces.push(namespace_stack.join("::"));
                visit_entity(&child, namespace_stack, ast);
                namespace_stack.pop();
            }
            kind if is_class_kind(kind) => {
                if let Some(instantiation) = extract_template_instantiation(&child) {
                    ast.template_instantiations.push(instantiation);
                }
                if let Some(class) = extract_class(&child, namespace_stack) {
                    ast.classes.push(class);
                }
                visit_entity(&child, namespace_stack, ast);
            }
            EntityKind::FunctionDecl => {
                if let Some(function) = extract_function(&child, namespace_stack) {
                    ast.functions.push(function);
                }
            }
            EntityKind::EnumDecl => {
                if let Some(cpp_enum) = extract_enum(&child, namespace_stack) {
                    ast.enums.push(cpp_enum);
                }
            }
            _ => visit_entity(&child, namespace_stack, ast),
        }
    }
}

fn is_class_kind(kind: EntityKind) -> bool {
    matches!(
        kind,
        EntityKind::ClassDecl
            | EntityKind::StructDecl
            | EntityKind::ClassTemplate
            | EntityKind::ClassTemplatePartialSpecialization
    )
}
