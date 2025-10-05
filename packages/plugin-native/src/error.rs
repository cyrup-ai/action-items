use std::io;

use thiserror::Error;

#[derive(Error, Debug)]
#[allow(clippy::enum_variant_names)] // Variants follow established error type patterns
pub enum Error {
    #[error("Plugin error: {0}")]
    PluginError(String),
    #[error("Execution error: {0}")]
    ExecutionError(String),
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Extism error: {0}")]
    Extism(String),
    #[error("Storage error: {0}")]
    StorageError(String),
    #[error("Plugin load error: {0}")]
    PluginLoadError(String),
    #[error("System error: {0}")]
    SystemError(String),
}

impl From<extism::Error> for Error {
    fn from(err: extism::Error) -> Self {
        Error::Extism(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, Error>;
