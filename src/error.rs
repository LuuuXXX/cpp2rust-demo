use anyhow::anyhow;
use std::fmt;

pub type Result<T> = anyhow::Result<T>;

/// 为错误附加上下文信息的便捷 trait
pub trait ToError<T> {
    fn ctx(self, info: &str) -> Result<T>;
}

impl<T, E: fmt::Display> ToError<T> for core::result::Result<T, E> {
    fn ctx(self, info: &str) -> Result<T> {
        self.map_err(|e| anyhow!("{}: {}", info, e))
    }
}
