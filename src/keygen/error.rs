//! Error types for key generation workflows.

use thiserror::Error;

/// Errors that can occur while handling key generation requests.
#[derive(Debug, Error, Eq, PartialEq)]
pub enum KeygenError {
    /// Feature is not implemented yet.
    #[error("key generation is not implemented yet")]
    NotImplemented,
    /// Input validation failed.
    #[error("invalid key generation request: {0}")]
    InvalidRequest(String),
    /// Underlying I/O failure.
    #[error("key generation I/O error: {0}")]
    Io(String),
    /// Downstream command failure (e.g., age-keygen error).
    #[error("key generation subprocess failed: {0}")]
    Subprocess(String),
    /// File already exists and force flag not provided.
    #[error("file already exists: {0} (use --force to overwrite)")]
    FileExists(String),
    /// Required binary not found.
    #[error("required binary not found: {0}. Please install age-keygen (https://github.com/FiloSottile/age)")]
    BinaryNotFound(String),
    /// Invalid recipient group.
    #[error("invalid recipient group: {0}")]
    InvalidGroup(String),
}
