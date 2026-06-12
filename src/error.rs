pub type Result<T> = anyhow::Result<T>;

/// cpp2rust 工具的具体错误类型。
///
/// 调用方可通过 `anyhow::Error::downcast_ref::<Cpp2RustError>()` 区分具体错误原因，
/// 例如区分"libclang 未找到"与"文件格式错误"。对外接口仍使用 `Result<T>` 保持兼容性。
#[derive(Debug)]
pub enum Cpp2RustError {
    /// libclang 初始化失败（通常是 LIBCLANG_PATH 未设置或 libclang.so 不存在）
    LibclangInit(String),
    /// AST 解析失败（libclang 无法解析指定的 C++ 源文件）
    ParseFailed(String),
    /// C++ 预处理失败（g++/clang++ 调用失败或源文件语法错误）
    PreprocessFailed(String),
    /// I/O 错误（文件读写、目录创建等）
    IoError(std::io::Error),
}

impl std::fmt::Display for Cpp2RustError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Cpp2RustError::LibclangInit(msg) => write!(f, "libclang 初始化失败: {}", msg),
            Cpp2RustError::ParseFailed(msg) => write!(f, "AST 解析失败: {}", msg),
            Cpp2RustError::PreprocessFailed(msg) => write!(f, "C++ 预处理失败: {}", msg),
            Cpp2RustError::IoError(e) => write!(f, "I/O 错误: {}", e),
        }
    }
}

impl std::error::Error for Cpp2RustError {}

impl From<std::io::Error> for Cpp2RustError {
    fn from(e: std::io::Error) -> Self {
        Cpp2RustError::IoError(e)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_libclang_init() {
        let e = Cpp2RustError::LibclangInit("libclang.so not found".to_string());
        assert_eq!(e.to_string(), "libclang 初始化失败: libclang.so not found");
    }

    #[test]
    fn display_parse_failed() {
        let e = Cpp2RustError::ParseFailed("unexpected token".to_string());
        assert_eq!(e.to_string(), "AST 解析失败: unexpected token");
    }

    #[test]
    fn display_preprocess_failed() {
        let e = Cpp2RustError::PreprocessFailed("g++ not found".to_string());
        assert_eq!(e.to_string(), "C++ 预处理失败: g++ not found");
    }

    #[test]
    fn display_io_error() {
        let e = Cpp2RustError::IoError(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "file not found",
        ));
        assert!(e.to_string().contains("I/O 错误"));
        assert!(e.to_string().contains("file not found"));
    }

    #[test]
    fn implements_std_error() {
        let e: Box<dyn std::error::Error> =
            Box::new(Cpp2RustError::IoError(std::io::Error::other("test")));
        assert!(!e.to_string().is_empty());
    }

    #[test]
    fn from_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let cpp_err = Cpp2RustError::from(io_err);
        assert!(
            matches!(cpp_err, Cpp2RustError::IoError(_)),
            "From<io::Error> 应转换为 IoError 变体"
        );
        assert!(cpp_err.to_string().contains("I/O 错误"));
    }
}
