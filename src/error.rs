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
    IoError(String),
}

impl std::fmt::Display for Cpp2RustError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Cpp2RustError::LibclangInit(msg) => write!(f, "libclang 初始化失败: {}", msg),
            Cpp2RustError::ParseFailed(msg) => write!(f, "AST 解析失败: {}", msg),
            Cpp2RustError::PreprocessFailed(msg) => write!(f, "C++ 预处理失败: {}", msg),
            Cpp2RustError::IoError(msg) => write!(f, "I/O 错误: {}", msg),
        }
    }
}

impl std::error::Error for Cpp2RustError {}
