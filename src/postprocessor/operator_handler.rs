use crate::types::CppAst;

pub fn normalize_operator_methods(ast: &mut CppAst) {
    for class in &mut ast.classes {
        for method in &mut class.methods {
            if let Some(kind) = &method.operator_kind {
                method.name = kind.to_rust_name(&class.name);
            }
        }
    }
}
