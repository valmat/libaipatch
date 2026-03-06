//! Patch engine: check and apply orchestration.
//!
//! The engine operates in two phases:
//! 1. Validation phase (shared by check and apply):
//!    - Parse the patch
//!    - Validate paths
//!    - Verify applicability against the real filesystem and an in-memory
//!      sequential patch state
//! 2. Commit phase (apply only):
//!    - Write changes to disk only after full validation succeeds
//!
//! No writes happen if validation fails at any point.

use std::collections::HashMap;
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
        dest: PathBuf,
        new_content: String,
    },
}

#[derive(Clone)]
enum PlannedEntry {
    Missing,
    File(String),
    Directory,
}

pub fn check(patch: &str, root_dir: &Path) -> Result<(), AiPatchError> {
    let parsed = crate::parser::parse_patch(patch)?;
    build_plan(&parsed, root_dir)?;
    Ok(())
}

pub fn apply(patch: &str, root_dir: &Path) -> Result<ApplyResult, AiPatchError> {
    let parsed = crate::parser::parse_patch(patch)?;
    let plan = build_plan(&parsed, root_dir)?;

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
            } => {
                let move_dest = (src != dest).then_some(dest.as_path());
                commit_update(&src, &new_content, move_dest)?;
                modified.push(dest.display().to_string());
            }
        }
    }

    Ok(ApplyResult {
        summary: build_summary(&added, &modified, &deleted),
    })
}

fn build_plan(parsed: &ParsedPatch, root_dir: &Path) -> Result<Vec<Operation>, AiPatchError> {
    validate_root_dir(root_dir)?;

    if parsed.hunks.is_empty() {
        return Err(AiPatchError::ParseError(
            crate::parser::ParseError::InvalidPatchError("patch does not contain any hunks".into()),
        ));
    }

    let mut ops: Vec<Operation> = Vec::new();
    let mut state: HashMap<PathBuf, PlannedEntry> = HashMap::new();

    for hunk in &parsed.hunks {
        match hunk {
            Hunk::AddFile { path, contents } => {
                let dest = validate_path(path, root_dir)?;
                validate_add_destination(&dest, root_dir, &mut state)?;
                state.insert(dest.clone(), PlannedEntry::File(contents.clone()));
                ops.push(Operation::Add {
                    dest,
                    contents: contents.clone(),
                });
            }
            Hunk::DeleteFile { path } => {
                let full_path = validate_path(path, root_dir)?;
                match get_entry_state(&full_path, &mut state)? {
                    PlannedEntry::Missing => {
                        return Err(AiPatchError::PatchConflict(format!(
                            "file to delete not found: {}",
                            full_path.display()
                        )));
                    }
                    PlannedEntry::Directory => {
                        return Err(AiPatchError::Unsupported(format!(
                            "{} is a directory, not a file",
                            full_path.display()
                        )));
                    }
                    PlannedEntry::File(_) => {
                        state.insert(full_path.clone(), PlannedEntry::Missing);
                        ops.push(Operation::Delete { path: full_path });
                    }
                }
            }
            Hunk::UpdateFile {
                path,
                move_path,
                chunks,
            } => {
                let src = validate_path(path, root_dir)?;
                let dest = match move_path {
                    Some(path) => validate_path(path, root_dir)?,
                    None => src.clone(),
                };

                let source_contents = match get_entry_state(&src, &mut state)? {
                    PlannedEntry::Missing => {
                        return Err(AiPatchError::PatchConflict(format!(
                            "file to update not found: {}",
                            src.display()
                        )));
                    }
                    PlannedEntry::Directory => {
                        return Err(AiPatchError::Unsupported(format!(
                            "{} is a directory, not a file",
                            src.display()
                        )));
                    }
                    PlannedEntry::File(contents) => contents,
                };

                validate_update_destination(&src, &dest, root_dir, &mut state)?;
                let new_content = compute_new_content(&source_contents, &src, chunks)?;

                if src != dest {
                    state.insert(src.clone(), PlannedEntry::Missing);
                }
                state.insert(dest.clone(), PlannedEntry::File(new_content.clone()));
                ops.push(Operation::Update {
                    src,
                    dest,
                    new_content,
                });
            }
        }
    }

    Ok(ops)
}

fn validate_root_dir(root_dir: &Path) -> Result<(), AiPatchError> {
    if root_dir.as_os_str().is_empty() {
        return Err(AiPatchError::InvalidArgument(
            "root_dir must not be empty".into(),
        ));
    }

    let metadata = std::fs::metadata(root_dir).map_err(|err| {
        if err.kind() == std::io::ErrorKind::NotFound {
            AiPatchError::InvalidArgument(format!(
                "root_dir does not exist: {}",
                root_dir.display()
            ))
        } else {
            io_error(format!("stat root_dir {}", root_dir.display()), err)
        }
    })?;

    if !metadata.is_dir() {
        return Err(AiPatchError::InvalidArgument(format!(
            "root_dir is not a directory: {}",
            root_dir.display()
        )));
    }

    Ok(())
}

fn validate_add_destination(
    dest: &Path,
    root_dir: &Path,
    state: &mut HashMap<PathBuf, PlannedEntry>,
) -> Result<(), AiPatchError> {
    ensure_parent_chain_is_directory(dest, root_dir, state)?;

    if matches!(get_entry_state(dest, state)?, PlannedEntry::Directory) {
        return Err(AiPatchError::Unsupported(format!(
            "{} is a directory, not a file",
            dest.display()
        )));
    }

    Ok(())
}

fn validate_update_destination(
    src: &Path,
    dest: &Path,
    root_dir: &Path,
    state: &mut HashMap<PathBuf, PlannedEntry>,
) -> Result<(), AiPatchError> {
    ensure_parent_chain_is_directory(dest, root_dir, state)?;

    if src == dest {
        return Ok(());
    }

    if matches!(get_entry_state(dest, state)?, PlannedEntry::Directory) {
        return Err(AiPatchError::Unsupported(format!(
            "{} is a directory, not a file",
            dest.display()
        )));
    }

    Ok(())
}

fn ensure_parent_chain_is_directory(
    path: &Path,
    root_dir: &Path,
    state: &mut HashMap<PathBuf, PlannedEntry>,
) -> Result<(), AiPatchError> {
    let mut current = path.parent();
    while let Some(parent) = current {
        if parent == root_dir {
            break;
        }
        match get_entry_state(parent, state)? {
            PlannedEntry::File(_) => {
                return Err(AiPatchError::Unsupported(format!(
                    "{} is a file, so {} cannot be created inside it",
                    parent.display(),
                    path.display()
                )));
            }
            PlannedEntry::Missing | PlannedEntry::Directory => {}
        }
        current = parent.parent();
    }
    Ok(())
}

fn get_entry_state(
    path: &Path,
    state: &mut HashMap<PathBuf, PlannedEntry>,
) -> Result<PlannedEntry, AiPatchError> {
    if let Some(entry) = state.get(path) {
        return Ok(entry.clone());
    }

    let loaded = load_entry_state(path)?;
    state.insert(path.to_path_buf(), loaded.clone());
    Ok(loaded)
}

fn load_entry_state(path: &Path) -> Result<PlannedEntry, AiPatchError> {
    let metadata = match std::fs::metadata(path) {
        Ok(metadata) => metadata,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(PlannedEntry::Missing),
        Err(err) => return Err(io_error(format!("stat {}", path.display()), err)),
    };

    if metadata.is_dir() {
        return Ok(PlannedEntry::Directory);
    }

    let contents = std::fs::read_to_string(path)
        .map_err(|err| io_error(format!("read file {}", path.display()), err))?;
    Ok(PlannedEntry::File(contents))
}

fn compute_new_content(
    original_contents: &str,
    path: &Path,
    chunks: &[UpdateFileChunk],
) -> Result<String, AiPatchError> {
    let mut original_lines: Vec<String> = original_contents.split('\n').map(String::from).collect();

    if original_lines.last().is_some_and(String::is_empty) {
        original_lines.pop();
    }

    let replacements = compute_replacements(&original_lines, path, chunks)?;
    let mut new_lines = apply_replacements(original_lines, &replacements);

    if !new_lines.last().is_some_and(String::is_empty) {
        new_lines.push(String::new());
    }

    Ok(new_lines.join("\n"))
}

fn compute_replacements(
    original_lines: &[String],
    path: &Path,
    chunks: &[UpdateFileChunk],
) -> Result<Vec<(usize, usize, Vec<String>)>, AiPatchError> {
    let mut replacements: Vec<(usize, usize, Vec<String>)> = Vec::new();
    let mut line_index: usize = 0;

    for chunk in chunks {
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
            let insertion_idx = if original_lines.last().is_some_and(String::is_empty) {
                original_lines.len() - 1
            } else {
                original_lines.len()
            };
            replacements.push((insertion_idx, 0, chunk.new_lines.clone()));
            continue;
        }

        let mut pattern: &[String] = &chunk.old_lines;
        let mut found = seek_sequence(original_lines, pattern, line_index, chunk.is_end_of_file);
        let mut new_slice: &[String] = &chunk.new_lines;

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
    let mut summary = String::from("Success. Updated the following files:\n");
    for path in added {
        summary.push_str(&format!("A {path}\n"));
    }
    for path in modified {
        summary.push_str(&format!("M {path}\n"));
    }
    for path in deleted {
        summary.push_str(&format!("D {path}\n"));
    }
    summary
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::TempDir;
    use std::fs;

    fn wrap_patch(body: &str) -> String {
        format!("*** Begin Patch\n{body}\n*** End Patch")
    }

    #[test]
    fn test_check_valid_add_patch() {
        let dir = TempDir::new().unwrap();
        let patch = wrap_patch("*** Add File: new.txt\n+hello");
        check(&patch, dir.path()).unwrap();
        assert!(!dir.path().join("new.txt").exists());
    }

    #[test]
    fn test_empty_patch_is_rejected() {
        let dir = TempDir::new().unwrap();
        let patch = wrap_patch("");
        let err = check(&patch, dir.path()).unwrap_err();
        assert!(matches!(err, AiPatchError::ParseError(_)));
    }

    #[test]
    fn test_apply_add_file() {
        let dir = TempDir::new().unwrap();
        let patch = wrap_patch("*** Add File: hello.txt\n+line1\n+line2");
        apply(&patch, dir.path()).unwrap();
        let contents = fs::read_to_string(dir.path().join("hello.txt")).unwrap();
        assert_eq!(contents, "line1\nline2\n");
    }

    #[test]
    fn test_apply_delete_file() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("del.txt"), "x").unwrap();
        let patch = wrap_patch("*** Delete File: del.txt");
        apply(&patch, dir.path()).unwrap();
        assert!(!dir.path().join("del.txt").exists());
    }

    #[test]
    fn test_apply_update_file() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("f.txt"), "foo\nbar\n").unwrap();
        let patch = wrap_patch("*** Update File: f.txt\n@@\n foo\n-bar\n+baz");
        apply(&patch, dir.path()).unwrap();
        let contents = fs::read_to_string(dir.path().join("f.txt")).unwrap();
        assert_eq!(contents, "foo\nbaz\n");
    }

    #[test]
    fn test_apply_update_with_move() {
        let dir = TempDir::new().unwrap();
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
    fn test_repeated_updates_use_planned_state() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("f.txt"), "one\n").unwrap();
        let patch = wrap_patch(
            "*** Update File: f.txt\n@@\n-one\n+two\n*** Update File: f.txt\n@@\n-two\n+three",
        );
        apply(&patch, dir.path()).unwrap();
        let contents = fs::read_to_string(dir.path().join("f.txt")).unwrap();
        assert_eq!(contents, "three\n");
    }

    #[test]
    fn test_check_does_not_write() {
        let dir = TempDir::new().unwrap();
        let patch = wrap_patch("*** Add File: shouldnotexist.txt\n+data");
        check(&patch, dir.path()).unwrap();
        assert!(!dir.path().join("shouldnotexist.txt").exists());
    }

    #[test]
    fn test_apply_invalid_patch_does_not_write() {
        let dir = TempDir::new().unwrap();
        let result = apply("*** Begin Patch\n*** Add File: bad.txt\n+x", dir.path());
        assert!(result.is_err());
        assert!(!dir.path().join("bad.txt").exists());
    }

    #[test]
    fn test_apply_conflict_does_not_write() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("f.txt"), "aaa\n").unwrap();
        let patch = wrap_patch("*** Update File: f.txt\n@@\n-bbb\n+ccc");
        let result = apply(&patch, dir.path());
        assert!(result.is_err());
        let contents = fs::read_to_string(dir.path().join("f.txt")).unwrap();
        assert_eq!(contents, "aaa\n");
    }

    #[test]
    fn test_path_traversal_rejected() {
        let dir = TempDir::new().unwrap();
        let patch = wrap_patch("*** Add File: ../../evil.txt\n+x");
        let result = apply(&patch, dir.path());
        assert!(matches!(result, Err(AiPatchError::PathViolation(_))));
    }

    #[test]
    fn test_apply_creates_parent_dirs() {
        let dir = TempDir::new().unwrap();
        let patch = wrap_patch("*** Add File: a/b/c.txt\n+hello");
        apply(&patch, dir.path()).unwrap();
        assert!(dir.path().join("a/b/c.txt").exists());
    }

    #[test]
    fn test_add_rejects_directory_destination() {
        let dir = TempDir::new().unwrap();
        fs::create_dir_all(dir.path().join("dup")).unwrap();
        let patch = wrap_patch("*** Add File: dup\n+hello");
        let err = check(&patch, dir.path()).unwrap_err();
        assert!(matches!(err, AiPatchError::Unsupported(_)));
    }
}
