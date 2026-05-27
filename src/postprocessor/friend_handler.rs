use crate::types::CppAst;

pub fn mark_friend_methods(ast: &mut CppAst) {
    for class in &mut ast.classes {
        for method in &mut class.methods {
            if method.name.contains("friend") {
                method.is_friend = true;
            }
        }
    }
}
