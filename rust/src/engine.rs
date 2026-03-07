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

    match get_entry_state(dest, state)? {
        PlannedEntry::Missing => {}
        PlannedEntry::Directory => {
            return Err(AiPatchError::Unsupported(format!(
                "{} is a directory, not a file",
                dest.display()
            )));
        }
        PlannedEntry::File(_) => {
            return Err(AiPatchError::PatchConflict(format!(
                "cannot add file because it already exists: {}",
                dest.display()
            )));
        }
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

    let bytes = std::fs::read(path).map_err(|err| io_error(format!("read file {}", path.display()), err))?;

    if bytes.contains(&0) {
        return Err(AiPatchError::Unsupported(format!(
            "binary file is not supported: {}",
            path.display()
        )));
    }

    let contents = String::from_utf8(bytes).map_err(|_| {
        AiPatchError::Unsupported(format!(
            "non-UTF-8 text file is not supported: {}",
            path.display()
        ))
    })?;

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

fn format_conflict_message(
    tag: &str,
    hint: &str,
    file: &Path,
    hunk_index: usize,
    expected_label: &str,
    expected_value: &str,
    nearest_actual: Option<String>,
    detail: String,
) -> String {
    let mut message = format!(
        "tag: {tag}\nhint: {hint}\nfile: {}\nhunk: {hunk_index}\n{expected_label}: {expected_value}",
        file.display()
    );

    if let Some(actual) = nearest_actual {
        message.push_str(&format!("\nnearest_actual: {actual}"));
    }

    message.push_str(&format!("\ndetail: {detail}"));
    message
}

fn normalise_hint_text(s: &str) -> String {
    s.trim()
        .chars()
        .map(|c| match c {
            '\u{2010}' | '\u{2011}' | '\u{2012}' | '\u{2013}' | '\u{2014}' | '\u{2015}'
            | '\u{2212}' => '-',
            '\u{2018}' | '\u{2019}' | '\u{201A}' | '\u{201B}' => '\'',
            '\u{201C}' | '\u{201D}' | '\u{201E}' | '\u{201F}' => '"',
            '\u{00A0}' | '\u{2002}' | '\u{2003}' | '\u{2004}' | '\u{2005}' | '\u{2006}'
            | '\u{2007}' | '\u{2008}' | '\u{2009}' | '\u{200A}' | '\u{202F}' | '\u{205F}'
            | '\u{3000}' => ' ',
            other => other,
        })
        .collect::<String>()
}

fn strip_all_whitespace(s: &str) -> String {
    normalise_hint_text(s)
        .chars()
        .filter(|ch| !ch.is_whitespace())
        .collect()
}

fn find_similar_line_index(lines: &[String], target: &str, start: usize) -> Option<usize> {
    let target_normalized = normalise_hint_text(target).to_lowercase();
    let target_compact = strip_all_whitespace(target).to_lowercase();

    let mut best: Option<(usize, usize)> = None;

    for (idx, line) in lines.iter().enumerate().skip(start) {
        let line_normalized = normalise_hint_text(line).to_lowercase();
        let line_compact = strip_all_whitespace(line).to_lowercase();

        let score = if line_normalized == target_normalized {
            4
        } else if !target_compact.is_empty() && line_compact == target_compact {
            3
        } else if !target_normalized.is_empty()
            && (line_normalized.contains(&target_normalized)
                || target_normalized.contains(&line_normalized))
        {
            2
        } else if !target_compact.is_empty()
            && (line_compact.contains(&target_compact) || target_compact.contains(&line_compact))
        {
            1
        } else {
            0
        };

        if score == 0 {
            continue;
        }

        match best {
            Some((best_score, _)) if best_score >= score => {}
            _ => best = Some((score, idx)),
        }
    }

    best.map(|(_, idx)| idx)
}

fn snippet_from(lines: &[String], start: usize, max_lines: usize) -> String {
    lines.iter()
        .skip(start)
        .take(max_lines)
        .map(String::as_str)
        .collect::<Vec<_>>()
        .join("\\n")
}

fn detect_probable_causes(lines: &[String], target: &str, start: usize) -> &'static str {
    if find_similar_line_index(lines, target, start).is_some() {
        "the file likely changed, the patch needs more context, or whitespace / line-ending differences are significant"
    } else {
        "the file likely changed since the patch was generated or the patch targets the wrong location"
    }
}

fn build_context_conflict(
    path: &Path,
    hunk_index: usize,
    expected_context: &str,
    lines: &[String],
    start: usize,
) -> AiPatchError {
    let nearest_actual = find_similar_line_index(lines, expected_context, start)
        .map(|idx| snippet_from(lines, idx, 3));
    let hint = detect_probable_causes(lines, expected_context, start);
    AiPatchError::PatchConflict(format_conflict_message(
        "conflict.update.context_not_found",
        hint,
        path,
        hunk_index,
        "expected_context",
        expected_context,
        nearest_actual,
        format!("failed to find context '{expected_context}' in {}", path.display()),
    ))
}

fn build_expected_lines_conflict(
    path: &Path,
    hunk_index: usize,
    expected_lines: &[String],
    lines: &[String],
    start: usize,
) -> AiPatchError {
    let anchor = expected_lines
        .iter()
        .find(|line| !line.trim().is_empty())
        .map(String::as_str)
        .unwrap_or("");
    let nearest_actual = find_similar_line_index(lines, anchor, start)
        .map(|idx| snippet_from(lines, idx, expected_lines.len().max(3)));
    let hint = detect_probable_causes(lines, anchor, start);
    AiPatchError::PatchConflict(format_conflict_message(
        "conflict.update.expected_lines_not_found",
        hint,
        path,
        hunk_index,
        "expected_lines",
        &expected_lines.join("\\n"),
        nearest_actual,
        format!("failed to find expected lines in {}", path.display()),
    ))
}

fn compute_replacements(
    original_lines: &[String],
    path: &Path,
    chunks: &[UpdateFileChunk],
) -> Result<Vec<(usize, usize, Vec<String>)>, AiPatchError> {
    let mut replacements: Vec<(usize, usize, Vec<String>)> = Vec::new();
    let mut line_index: usize = 0;

    for (index, chunk) in chunks.iter().enumerate() {
        let hunk_index = index + 1;
        if let Some(ctx_line) = &chunk.change_context {
            if let Some(idx) = seek_sequence(
                original_lines,
                std::slice::from_ref(ctx_line),
                line_index,
                false,
            ) {
                line_index = idx + 1;
            } else {
                return Err(build_context_conflict(
                    path,
                    hunk_index,
                    ctx_line,
                    original_lines,
                    line_index,
                ));
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
            return Err(build_expected_lines_conflict(
                path,
                hunk_index,
                &chunk.old_lines,
                original_lines,
                line_index,
            ));
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
    fn test_conflict_message_includes_expected_lines_and_hunk() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("f.txt"), "aaa\n").unwrap();
        let patch = wrap_patch("*** Update File: f.txt\n@@\n-bbb\n+ccc");
        let err = check(&patch, dir.path()).unwrap_err();
        match err {
            AiPatchError::PatchConflict(message) => {
                assert!(message.contains("tag: conflict.update.expected_lines_not_found"));
                assert!(message.contains("file: "));
                assert!(message.contains("hunk: 1"));
                assert!(message.contains("expected_lines: bbb"));
                assert!(message.contains("detail: failed to find expected lines in"));
            }
            _ => panic!("expected PatchConflict"),
        }
    }

    #[test]
    fn test_conflict_message_includes_nearest_actual_for_context() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("f.txt"), "fn main ()\nbody\n").unwrap();
        let patch = wrap_patch("*** Update File: f.txt\n@@ fn main()\n-body\n+body2");
        let err = check(&patch, dir.path()).unwrap_err();
        match err {
            AiPatchError::PatchConflict(message) => {
                assert!(message.contains("tag: conflict.update.context_not_found"));
                assert!(message.contains("expected_context: fn main()"));
                assert!(message.contains("nearest_actual: fn main ()"));
                assert!(message.contains("whitespace"));
            }
            _ => panic!("expected PatchConflict"),
        }
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

    #[test]
    fn test_add_rejects_existing_file_destination() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("dup"), "hello\n").unwrap();
        let patch = wrap_patch("*** Add File: dup\n+updated");
        let err = check(&patch, dir.path()).unwrap_err();
        match err {
            AiPatchError::PatchConflict(message) => {
                assert!(message.contains("already exists"));
            }
            _ => panic!("expected PatchConflict"),
        }
    }

    #[test]
    fn test_update_rejects_non_utf8_file_as_unsupported() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("f.txt"), [0xff, 0xfe, 0xfd]).unwrap();
        let patch = wrap_patch("*** Update File: f.txt\n@@\n-old\n+new");
        let err = check(&patch, dir.path()).unwrap_err();
        match err {
            AiPatchError::Unsupported(message) => {
                assert!(message.contains("non-UTF-8"));
                assert!(message.contains("f.txt"));
            }
            _ => panic!("expected Unsupported"),
        }
    }

    #[test]
    fn test_update_rejects_binary_file_as_unsupported() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("f.bin"), b"a\0b\n").unwrap();
        let patch = wrap_patch("*** Update File: f.bin\n@@\n-a\n+b");
        let err = check(&patch, dir.path()).unwrap_err();
        match err {
            AiPatchError::Unsupported(message) => {
                assert!(message.contains("binary file"));
                assert!(message.contains("f.bin"));
            }
            _ => panic!("expected Unsupported"),
        }
    }
}
