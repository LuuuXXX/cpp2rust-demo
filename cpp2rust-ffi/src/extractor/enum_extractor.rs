use crate::types::*;

/// 枚举提取器
pub struct EnumExtractor;

impl EnumExtractor {
    /// 生成 Rust 常量定义（用于 enum class）
    pub fn generate_consts(enum_: &CppEnum) -> Vec<String> {
        let mut lines = Vec::new();
        let (rust_type, _) = map_cpp_type_to_rust(&enum_.underlying_type);

        for (name, value) in &enum_.values {
            lines.push(format!(
                "pub const {}: {} = {};",
                name.to_uppercase(),
                rust_type,
                value
            ));
        }
        lines
    }
}
