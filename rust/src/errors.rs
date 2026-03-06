//! Error types for libaipatch and mapping to C ABI error codes.

use thiserror::Error;

use crate::parser::ParseError;

/// ABI error code constants (used in `aipatch_result.code`).
pub const AIPATCH_OK: i32 = 0;
pub const AIPATCH_INVALID_ARGUMENT: i32 = 1;
pub const AIPATCH_PARSE_ERROR: i32 = 2;
pub const AIPATCH_IO_ERROR: i32 = 3;
pub const AIPATCH_PATCH_CONFLICT: i32 = 4;
pub const AIPATCH_PATH_VIOLATION: i32 = 5;
pub const AIPATCH_UNSUPPORTED: i32 = 6;
pub const AIPATCH_INTERNAL_ERROR: i32 = 7;

/// Internal error type for the libaipatch engine.
#[derive(Debug, Error)]
pub enum AiPatchError {
    #[error("invalid argument: {0}")]
    InvalidArgument(String),

    #[error("parse error: {0}")]
    ParseError(#[from] ParseError),

    #[error("I/O error: {context}: {source}")]
    IoError {
        context: String,
        #[source]
        source: std::io::Error,
    },

    #[error("patch conflict: {0}")]
    PatchConflict(String),

    #[error("path violation: {0}")]
    PathViolation(String),

    #[error("unsupported: {0}")]
    Unsupported(String),

    #[error("internal error: {0}")]
    Internal(String),
}

impl AiPatchError {
    /// Map this error to its ABI error code.
    pub fn abi_code(&self) -> i32 {
        match self {
            AiPatchError::InvalidArgument(_) => AIPATCH_INVALID_ARGUMENT,
            AiPatchError::ParseError(_) => AIPATCH_PARSE_ERROR,
            AiPatchError::IoError { .. } => AIPATCH_IO_ERROR,
            AiPatchError::PatchConflict(_) => AIPATCH_PATCH_CONFLICT,
            AiPatchError::PathViolation(_) => AIPATCH_PATH_VIOLATION,
            AiPatchError::Unsupported(_) => AIPATCH_UNSUPPORTED,
            AiPatchError::Internal(_) => AIPATCH_INTERNAL_ERROR,
        }
    }
}

/// Convenience constructor for IoError.
pub(crate) fn io_error(context: impl Into<String>, source: std::io::Error) -> AiPatchError {
    AiPatchError::IoError {
        context: context.into(),
        source,
    }
}
