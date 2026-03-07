//! Path policy for libaipatch.
//!
//! Enforces that all paths in a patch are:
//! - Relative (not absolute)
//! - Within the given root_dir after normalization
//! - Free of path traversal attempts
//!
//! This policy is mandatory and cannot be bypassed by the caller.

use std::path::{Component, Path, PathBuf};

use crate::errors::AiPatchError;

fn format_path_violation(
    tag: &str,
    hint: &str,
    raw: &Path,
    root_dir: &Path,
    detail: String,
) -> AiPatchError {
    AiPatchError::PathViolation(format!(
        "tag: {tag}\nhint: {hint}\npath: {}\nroot_dir: {}\ndetail: {detail}",
        raw.display(),
        root_dir.display()
    ))
}

/// Validate and resolve a raw path from the patch against root_dir.
///
/// Returns the absolute resolved path if valid, or an error if the path
/// is absolute, uses traversal, or escapes root_dir.
pub fn validate_path(raw: &Path, root_dir: &Path) -> Result<PathBuf, AiPatchError> {
    // Reject absolute paths in the patch.
    if raw.is_absolute() {
        return Err(format_path_violation(
            "path.absolute_not_allowed",
            "use a relative path inside root_dir",
            raw,
            root_dir,
            format!("absolute paths are not allowed in patches: {}", raw.display()),
        ));
    }

    // Normalize the path by resolving '..' and '.' components without
    // actually touching the filesystem (Path::canonicalize would require
    // the path to exist). We do a manual component walk instead.
    let mut normalized = root_dir.to_path_buf();
    for component in raw.components() {
        match component {
            Component::CurDir => {
                // '.' — stay in same directory
            }
            Component::ParentDir => {
                // '..' — try to pop, but if we can't go further back than
                // root_dir, reject as a traversal attempt.
                if !normalized.pop() || normalized.as_os_str().is_empty() {
                    return Err(format_path_violation(
                        "path.traversal_detected",
                        "remove '..' segments that escape root_dir and keep the patch path relative",
                        raw,
                        root_dir,
                        format!("path traversal detected: {} escapes root_dir", raw.display()),
                    ));
                }
                // Check that we haven't gone above root_dir.
                if !normalized.starts_with(root_dir) {
                    return Err(format_path_violation(
                        "path.traversal_detected",
                        "remove '..' segments that escape root_dir and keep the patch path relative",
                        raw,
                        root_dir,
                        format!("path traversal detected: {} escapes root_dir", raw.display()),
                    ));
                }
            }
            Component::Normal(part) => {
                normalized.push(part);
            }
            Component::RootDir | Component::Prefix(_) => {
                // These should not appear in a relative path, but guard anyway.
                return Err(format_path_violation(
                    "path.invalid_component",
                    "use a normal relative filesystem path without drive prefixes or root markers",
                    raw,
                    root_dir,
                    format!("invalid path component in patch path: {}", raw.display()),
                ));
            }
        }
    }

    // Final check: the normalized path must start with root_dir.
    if !normalized.starts_with(root_dir) {
        return Err(format_path_violation(
            "path.outside_root",
            "keep the resolved patch path inside root_dir",
            raw,
            root_dir,
            format!("path {} escapes root_dir {}", raw.display(), root_dir.display()),
        ));
    }

    Ok(normalized)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn root() -> PathBuf {
        PathBuf::from("/sandbox")
    }

    #[test]
    fn test_valid_relative_path() {
        let result = validate_path(Path::new("src/main.rs"), &root());
        assert_eq!(result.unwrap(), PathBuf::from("/sandbox/src/main.rs"));
    }

    #[test]
    fn test_reject_absolute_path() {
        let result = validate_path(Path::new("/etc/passwd"), &root());
        match result.unwrap_err() {
            AiPatchError::PathViolation(message) => {
                assert!(message.contains("tag: path.absolute_not_allowed"));
                assert!(message.contains("root_dir: /sandbox"));
            }
            _ => panic!("expected PathViolation"),
        }
    }

    #[test]
    fn test_reject_simple_traversal() {
        let result = validate_path(Path::new("../../etc/passwd"), &root());
        assert!(matches!(result, Err(AiPatchError::PathViolation(_))));
    }

    #[test]
    fn test_reject_hidden_traversal() {
        let result = validate_path(Path::new("foo/../../../etc"), &root());
        assert!(matches!(result, Err(AiPatchError::PathViolation(_))));
    }

    #[test]
    fn test_curdir_is_fine() {
        let result = validate_path(Path::new("./src/lib.rs"), &root());
        assert_eq!(result.unwrap(), PathBuf::from("/sandbox/src/lib.rs"));
    }

    #[test]
    fn test_parent_within_root_is_fine() {
        // foo/../bar is within root — should resolve to /sandbox/bar
        let result = validate_path(Path::new("foo/../bar"), &root());
        assert_eq!(result.unwrap(), PathBuf::from("/sandbox/bar"));
    }
}
