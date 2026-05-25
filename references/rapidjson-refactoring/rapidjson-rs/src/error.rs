//! Error model for rapidjson-rs core infrastructure.

use core::fmt;

/// Unified error type for rapidjson-rs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// Errors that arise during parsing of JSON input.
    ///
    /// The optional `offset` represents the byte position in the
    /// input where the error was detected when available.
    Parse {
        message: &'static str,
        offset: Option<usize>,
    },
    /// Errors that arise during encoding or serialization.
    ///
    /// The optional `offset` represents the byte position in the
    /// output where the error was detected when available.
    Encode {
        message: &'static str,
        offset: Option<usize>,
    },
    /// I/O related failures.
    Io,
    /// Memory allocation or exhaustion failures.
    Memory,
    /// Internal invariant violations.
    Internal,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Parse { message, offset } => {
                if let Some(pos) = offset {
                    write!(f, "parse error at {}: {}", pos, message)
                } else {
                    write!(f, "parse error: {}", message)
                }
            }
            Error::Encode { message, offset } => {
                if let Some(pos) = offset {
                    write!(f, "encode error at {}: {}", pos, message)
                } else {
                    write!(f, "encode error: {}", message)
                }
            }
            Error::Io => write!(f, "io error"),
            Error::Memory => write!(f, "memory error"),
            Error::Internal => write!(f, "internal error"),
        }
    }
}

impl std::error::Error for Error {}

#[cfg(test)]
mod tests {
    use super::Error;

    #[test]
    fn should_format_message_when_parse_error() {
        let err = Error::Parse {
            message: "invalid token",
            offset: None,
        };
        let formatted = err.to_string();
        assert!(formatted.contains("parse error"));
        assert!(formatted.contains("invalid token"));
    }

    #[test]
    fn should_format_message_when_encode_error() {
        let err = Error::Encode {
            message: "invalid state",
            offset: None,
        };
        let formatted = err.to_string();
        assert!(formatted.contains("encode error"));
        assert!(formatted.contains("invalid state"));
    }

    #[test]
    fn should_include_offset_when_parse_error_with_position() {
        let err = Error::Parse {
            message: "unexpected token",
            offset: Some(10),
        };
        let formatted = err.to_string();
        assert!(formatted.contains("parse error at 10"));
        assert!(formatted.contains("unexpected token"));
    }

    #[test]
    fn should_format_labels_when_simple_error_variants() {
        assert_eq!(Error::Io.to_string(), "io error");
        assert_eq!(Error::Memory.to_string(), "memory error");
        assert_eq!(Error::Internal.to_string(), "internal error");
    }

    #[test]
    fn should_be_send_and_sync_when_error() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<Error>();
    }
}
