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

    #[error("{message}")]
    IoError {
        message: String,
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

fn io_error_hint(kind: std::io::ErrorKind) -> &'static str {
    match kind {
        std::io::ErrorKind::NotFound => {
            "check that the referenced file or directory exists and was not changed concurrently"
        }
        std::io::ErrorKind::PermissionDenied => {
            "check filesystem permissions and whether the destination is writable"
        }
        std::io::ErrorKind::AlreadyExists => {
            "check whether a file or directory already exists at the destination path"
        }
        _ => "check filesystem state, permissions, free space, and concurrent modifications",
    }
}

/// Convenience constructor for IoError.
pub(crate) fn io_error(context: impl Into<String>, source: std::io::Error) -> AiPatchError {
    let context = context.into();
    let kind = source.kind();
    AiPatchError::IoError {
        message: format!(
            "tag: io.error\nhint: {}\ncontext: {}\nkind: {:?}\ndetail: {}",
            io_error_hint(kind),
            context,
            kind,
            source
        ),
        source,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_io_error_is_agent_friendly() {
        let err = io_error(
            "write file /tmp/demo.txt",
            std::io::Error::new(std::io::ErrorKind::PermissionDenied, "permission denied"),
        );

        match err {
            AiPatchError::IoError { message, .. } => {
                assert!(message.contains("tag: io.error"));
                assert!(message.contains("context: write file /tmp/demo.txt"));
                assert!(message.contains("kind: PermissionDenied"));
                assert!(message.contains("permission denied"));
            }
            _ => panic!("expected IoError"),
        }
    }
}
