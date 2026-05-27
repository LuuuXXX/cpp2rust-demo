pub mod friend_handler;
pub mod lambda_handler;
pub mod operator_handler;

use crate::types::CppAst;

pub fn postprocess_ast(ast: &mut CppAst) {
    operator_handler::normalize_operator_methods(ast);
    friend_handler::mark_friend_methods(ast);
    lambda_handler::annotate_lambda_artifacts(ast);
}
