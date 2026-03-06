//! Patch engine: check and apply orchestration.
//!
//! The engine operates in two phases:
//! 1. Validation phase (shared by check and apply):
//!    - Parse the patch
//!    - Validate paths
//!    - Verify applicability (read files, compute replacements)
//! 2. Commit phase (apply only):
//!    - Write changes to disk only after full validation succeeds
//!
//! No writes happen if validation fails at any point.

use std::path::{Path, PathBuf};

use crate::errors::{io_error, AiPatchError};
use crate::parser::{Hunk, ParsedPatch, UpdateFileChunk};
use crate::paths::validate_path;
use crate::seek_sequence::seek_sequence;
use crate::write_ops::{commit_add, commit_delete, commit_update};

/// The result of applying a patch: list of human-readable change summaries.
pub struct ApplyResult {
    pub summary: String,
}

/// Parsed + validated plan: a list of concrete file operations.
enum Operation {
    Add {
        dest: PathBuf,
        contents: String,
    },
    Delete {
        path: PathBuf,
    },
    Update {
        src: PathBuf,
        dest: PathBuf, // same as src if no move
        new_content: String,
        is_move: bool,
    },
}

/// Validate that a patch is applicable to the filesystem rooted at `root_dir`.
/// Does NOT write anything to disk.
pub fn check(patch: &str, root_dir: &Path) -> Result<(), AiPatchError> {
    let parsed = crate::parser::parse_patch(patch)?;
    build_plan(&parsed, root_dir)?;
    Ok(())
}

/// Apply a patch to the filesystem rooted at `root_dir`.
/// Validates fully before any writes.
pub fn apply(patch: &str, root_dir: &Path) -> Result<ApplyResult, AiPatchError> {
    let parsed = crate::parser::parse_patch(patch)?;
    let plan = build_plan(&parsed, root_dir)?;

    // Commit phase: all validation passed, now write.
    let mut added: Vec<String> = Vec::new();
    let mut modified: Vec<String> = Vec::new();
    let mut deleted: Vec<String> = Vec::new();

    for op in plan {
        match op {
            Operation::Add { dest, contents } => {
                commit_add(&dest, &contents)?;
                added.push(dest.display().to_string());
            }
            Operation::Delete { path } => {
                commit_delete(&path)?;
                deleted.push(path.display().to_string());
            }
            Operation::Update {
                src,
                dest,
                new_content,
                is_move,
            } => {
                let move_dest = if is_move { Some(dest.as_path()) } else { None };
                commit_update(&src, &new_content, move_dest)?;
                modified.push(dest.display().to_string());
            }
        }
    }

    let summary = build_summary(&added, &modified, &deleted);
    Ok(ApplyResult { summary })
}

/// Validation phase: parse hunks, validate paths, check applicability.
/// Returns the operation plan if everything is valid.
fn build_plan(parsed: &ParsedPatch, root_dir: &Path) -> Result<Vec<Operation>, AiPatchError> {
    let mut ops: Vec<Operation> = Vec::new();

    for hunk in &parsed.hunks {
        match hunk {
            Hunk::AddFile { path, contents } => {
                let dest = validate_path(path, root_dir)?;
                ops.push(Operation::Add {
                    dest,
                    contents: contents.clone(),
                });
            }
            Hunk::DeleteFile { path } => {
                let full_path = validate_path(path, root_dir)?;
                // Verify the file actually exists.
                if !full_path.exists() {
                    return Err(io_error(
                        format!("file to delete not found: {}", full_path.display()),
                        std::io::Error::new(
                            std::io::ErrorKind::NotFound,
                            "file not found",
                        ),
                    ));
                }
                if full_path.is_dir() {
                    return Err(AiPatchError::Unsupported(format!(
                        "{} is a directory, not a file",
                        full_path.display()
                    )));
                }
                ops.push(Operation::Delete { path: full_path });
            }
            Hunk::UpdateFile {
                path,
                move_path,
                chunks,
            } => {
                let src = validate_path(path, root_dir)?;
                let dest = if let Some(mp) = move_path {
                    validate_path(mp, root_dir)?
                } else {
                    src.clone()
                };

                // Read the source file.
                if !src.exists() {
                    return Err(io_error(
                        format!("file to update not found: {}", src.display()),
                        std::io::Error::new(std::io::ErrorKind::NotFound, "not found"),
                    ));
                }
                if src.is_dir() {
                    return Err(AiPatchError::Unsupported(format!(
                        "{} is a directory, not a file",
                        src.display()
                    )));
                }

                // Read and verify UTF-8.
                let original_contents = std::fs::read_to_string(&src).map_err(|e| {
                    io_error(format!("read file {}", src.display()), e)
                })?;

                let new_content = compute_new_content(&original_contents, &src, chunks)?;

                ops.push(Operation::Update {
                    src: src.clone(),
                    dest,
                    new_content,
                    is_move: move_path.is_some(),
                });
            }
        }
    }

    Ok(ops)
}

/// Compute the new file contents after applying chunks to the original.
/// Adapted from codex/codex-rs/apply-patch/src/lib.rs.
fn compute_new_content(
    original_contents: &str,
    path: &Path,
    chunks: &[UpdateFileChunk],
) -> Result<String, AiPatchError> {
    let mut original_lines: Vec<String> =
        original_contents.split('\n').map(String::from).collect();

    // Drop the trailing empty element that results from the final newline.
    if original_lines.last().is_some_and(String::is_empty) {
        original_lines.pop();
    }

    let replacements = compute_replacements(&original_lines, path, chunks)?;
    let mut new_lines = apply_replacements(original_lines, &replacements);

    // Always ensure trailing newline (compatible with codex behaviour).
    if !new_lines.last().is_some_and(String::is_empty) {
        new_lines.push(String::new());
    }

    Ok(new_lines.join("\n"))
}

/// Compute a list of `(start_index, old_len, new_lines)` replacements.
/// Adapted from codex/codex-rs/apply-patch/src/lib.rs.
fn compute_replacements(
    original_lines: &[String],
    path: &Path,
    chunks: &[UpdateFileChunk],
) -> Result<Vec<(usize, usize, Vec<String>)>, AiPatchError> {
    let mut replacements: Vec<(usize, usize, Vec<String>)> = Vec::new();
    let mut line_index: usize = 0;

    for chunk in chunks {
        // If a chunk has a change_context, seek it first.
        if let Some(ctx_line) = &chunk.change_context {
            if let Some(idx) = seek_sequence(
                original_lines,
                std::slice::from_ref(ctx_line),
                line_index,
                false,
            ) {
                line_index = idx + 1;
            } else {
                return Err(AiPatchError::PatchConflict(format!(
                    "failed to find context '{}' in {}",
                    ctx_line,
                    path.display()
                )));
            }
        }

        if chunk.old_lines.is_empty() {
            // Pure addition: insert at end (or before final empty newline).
            let insertion_idx = if original_lines
                .last()
                .is_some_and(String::is_empty)
            {
                original_lines.len() - 1
            } else {
                original_lines.len()
            };
            replacements.push((insertion_idx, 0, chunk.new_lines.clone()));
            continue;
        }

        // Try to match old_lines in the file.
        let mut pattern: &[String] = &chunk.old_lines;
        let mut found = seek_sequence(original_lines, pattern, line_index, chunk.is_end_of_file);

        let mut new_slice: &[String] = &chunk.new_lines;

        // Retry without trailing empty line (represents final newline sentinel).
        if found.is_none() && pattern.last().is_some_and(String::is_empty) {
            pattern = &pattern[..pattern.len() - 1];
            if new_slice.last().is_some_and(String::is_empty) {
                new_slice = &new_slice[..new_slice.len() - 1];
            }
            found = seek_sequence(original_lines, pattern, line_index, chunk.is_end_of_file);
        }

        if let Some(start_idx) = found {
            replacements.push((start_idx, pattern.len(), new_slice.to_vec()));
            line_index = start_idx + pattern.len();
        } else {
            return Err(AiPatchError::PatchConflict(format!(
                "failed to find expected lines in {}:\n{}",
                path.display(),
                chunk.old_lines.join("\n"),
            )));
        }
    }

    replacements.sort_by(|(a, _, _), (b, _, _)| a.cmp(b));
    Ok(replacements)
}

/// Apply replacements to lines in reverse order (to preserve indices).
fn apply_replacements(
    mut lines: Vec<String>,
    replacements: &[(usize, usize, Vec<String>)],
) -> Vec<String> {
    for (start_idx, old_len, new_segment) in replacements.iter().rev() {
        let start_idx = *start_idx;
        let old_len = *old_len;

        for _ in 0..old_len {
            if start_idx < lines.len() {
                lines.remove(start_idx);
            }
        }
        for (offset, new_line) in new_segment.iter().enumerate() {
            lines.insert(start_idx + offset, new_line.clone());
        }
    }
    lines
}

fn build_summary(added: &[String], modified: &[String], deleted: &[String]) -> String {
    let mut s = String::from("Success. Updated the following files:\n");
    for p in added {
        s.push_str(&format!("A {p}\n"));
    }
    for p in modified {
        s.push_str(&format!("M {p}\n"));
    }
    for p in deleted {
        s.push_str(&format!("D {p}\n"));
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    fn wrap_patch(body: &str) -> String {
        format!("*** Begin Patch\n{body}\n*** End Patch")
    }

    #[test]
    fn test_check_valid_add_patch() {
        let dir = tempdir().unwrap();
        let patch = wrap_patch("*** Add File: new.txt\n+hello");
        // check should succeed without writing anything.
        check(&patch, dir.path()).unwrap();
        assert!(!dir.path().join("new.txt").exists());
    }

    #[test]
    fn test_apply_add_file() {
        let dir = tempdir().unwrap();
        let patch = wrap_patch("*** Add File: hello.txt\n+line1\n+line2");
        apply(&patch, dir.path()).unwrap();
        let contents = fs::read_to_string(dir.path().join("hello.txt")).unwrap();
        assert_eq!(contents, "line1\nline2\n");
    }

    #[test]
    fn test_apply_delete_file() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("del.txt"), "x").unwrap();
        let patch = wrap_patch("*** Delete File: del.txt");
        apply(&patch, dir.path()).unwrap();
        assert!(!dir.path().join("del.txt").exists());
    }

    #[test]
    fn test_apply_update_file() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("f.txt"), "foo\nbar\n").unwrap();
        let patch = wrap_patch(
            "*** Update File: f.txt\n@@\n foo\n-bar\n+baz",
        );
        apply(&patch, dir.path()).unwrap();
        let contents = fs::read_to_string(dir.path().join("f.txt")).unwrap();
        assert_eq!(contents, "foo\nbaz\n");
    }

    #[test]
    fn test_apply_update_with_move() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("src.txt"), "line\n").unwrap();
        let patch = wrap_patch(
            "*** Update File: src.txt\n*** Move to: dst.txt\n@@\n-line\n+line2",
        );
        apply(&patch, dir.path()).unwrap();
        assert!(!dir.path().join("src.txt").exists());
        let contents = fs::read_to_string(dir.path().join("dst.txt")).unwrap();
        assert_eq!(contents, "line2\n");
    }

    #[test]
    fn test_check_does_not_write() {
        let dir = tempdir().unwrap();
        let patch = wrap_patch("*** Add File: shouldnotexist.txt\n+data");
        check(&patch, dir.path()).unwrap();
        assert!(!dir.path().join("shouldnotexist.txt").exists());
    }

    #[test]
    fn test_apply_invalid_patch_does_not_write() {
        let dir = tempdir().unwrap();
        // Invalid patch (missing End Patch).
        let result = apply("*** Begin Patch\n*** Add File: bad.txt\n+x", dir.path());
        assert!(result.is_err());
        assert!(!dir.path().join("bad.txt").exists());
    }

    #[test]
    fn test_apply_conflict_does_not_write() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("f.txt"), "aaa\n").unwrap();
        // Patch expects "bbb" which is not in the file.
        let patch = wrap_patch("*** Update File: f.txt\n@@\n-bbb\n+ccc");
        let result = apply(&patch, dir.path());
        assert!(result.is_err());
        // File should be unchanged.
        let contents = fs::read_to_string(dir.path().join("f.txt")).unwrap();
        assert_eq!(contents, "aaa\n");
    }

    #[test]
    fn test_path_traversal_rejected() {
        let dir = tempdir().unwrap();
        let patch = wrap_patch("*** Add File: ../../evil.txt\n+x");
        let result = apply(&patch, dir.path());
        assert!(matches!(result, Err(AiPatchError::PathViolation(_))));
    }

    #[test]
    fn test_apply_creates_parent_dirs() {
        let dir = tempdir().unwrap();
        let patch = wrap_patch("*** Add File: a/b/c.txt\n+hello");
        apply(&patch, dir.path()).unwrap();
        assert!(dir.path().join("a/b/c.txt").exists());
    }
}
