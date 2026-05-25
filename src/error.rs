use std::fmt::{Display, Formatter};
use std::path::PathBuf;

pub type Result<T> = std::result::Result<T, DemoError>;

#[derive(Debug)]
pub enum DemoError {
    Io(std::io::Error),
    Json(serde_json::Error),
    CommandFailed { program: String, code: Option<i32> },
    MissingArgument(&'static str),
    InvalidPath(PathBuf),
    Parse(String),
}

impl Display for DemoError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(err) => write!(f, "io error: {err}"),
            Self::Json(err) => write!(f, "json error: {err}"),
            Self::CommandFailed { program, code } => match code {
                Some(code) => write!(f, "command `{program}` failed with exit code {code}"),
                None => write!(f, "command `{program}` terminated by signal"),
            },
            Self::MissingArgument(arg) => write!(f, "missing required argument: {arg}"),
            Self::InvalidPath(path) => write!(f, "invalid path: {}", path.display()),
            Self::Parse(msg) => write!(f, "parse error: {msg}"),
        }
    }
}

impl std::error::Error for DemoError {}

impl From<std::io::Error> for DemoError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<serde_json::Error> for DemoError {
    fn from(value: serde_json::Error) -> Self {
        Self::Json(value)
    }
}

impl From<dialoguer::Error> for DemoError {
    fn from(value: dialoguer::Error) -> Self {
        Self::Parse(value.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_command_failure_with_code() {
        let err = DemoError::CommandFailed {
            program: "make".into(),
            code: Some(2),
        };
        assert_eq!(err.to_string(), "command `make` failed with exit code 2");
    }

    #[test]
    fn formats_command_failure_without_code() {
        let err = DemoError::CommandFailed {
            program: "clang++".into(),
            code: None,
        };
        assert!(err.to_string().contains("signal"));
    }

    #[test]
    fn converts_io_error() {
        let err = std::io::Error::from(std::io::ErrorKind::NotFound);
        let demo: DemoError = err.into();
        assert!(matches!(demo, DemoError::Io(_)));
    }
}
