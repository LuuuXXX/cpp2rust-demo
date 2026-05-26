use crate::types::*;

/// 运算符重载后处理器
/// 将 operator+/- 等转换为命名函数 shim
pub struct OperatorHandler;

impl OperatorHandler {
    /// 将 C++ 运算符名转换为 Rust 友好的函数名
    pub fn operator_to_name(class_prefix: &str, op: &str) -> Option<String> {
        let op_name = match op.trim_start_matches("operator").trim() {
            "+" => "add",
            "-" => "sub",
            "*" => "mul",
            "/" => "div",
            "%" => "rem",
            "==" => "eq",
            "!=" => "ne",
            "<" => "lt",
            "<=" => "le",
            ">" => "gt",
            ">=" => "ge",
            "&&" => "and",
            "||" => "or",
            "!" => "not",
            "~" => "bitwise_not",
            "&" => "bitand",
            "|" => "bitor",
            "^" => "bitxor",
            "<<" => "shl",
            ">>" => "shr",
            "+=" => "add_assign",
            "-=" => "sub_assign",
            "*=" => "mul_assign",
            "/=" => "div_assign",
            "[]" => "index",
            "()" => "call",
            "=" => "assign",
            _ => return None,
        };
        Some(format!("{}_{}", class_prefix, op_name))
    }

    /// 检查函数名是否是运算符
    pub fn is_operator(method_name: &str) -> bool {
        method_name.starts_with("operator")
    }
}

/// Lambda 后处理器
/// 处理 lambda 表达式和 std::function
pub struct LambdaHandler;

impl LambdaHandler {
    /// 检查类是否是 lambda wrapper（通常匿名，或包含 operator()）
    pub fn is_lambda_wrapper(class: &CppClass) -> bool {
        class.methods.iter().any(|m| m.name == "operator()")
    }
}

/// 友元函数后处理器
pub struct FriendHandler;

impl FriendHandler {
    /// 友元函数在 FFI 中等同于普通全局函数，无需特殊处理
    /// 只需标注注释说明友元身份
    pub fn annotate_friend(func: &mut crate::types::CppFunction, class_name: &str) {
        // 添加说明注释（通过在 cpp_signature 前加注释）
        func.cpp_signature = format!(
            "/* friend of {} */ {}",
            class_name, func.cpp_signature
        );
    }
}
