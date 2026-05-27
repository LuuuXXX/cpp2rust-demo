use crate::types::{CppAst, CppClass};

pub fn annotate_lambda_artifacts(ast: &mut CppAst) {
    for function in &ast.functions {
        if function.name.contains("lambda") {
            ast.classes.push(CppClass {
                name: format!("{}Wrapper", function.name),
                ..CppClass::default()
            });
        }
    }
}
