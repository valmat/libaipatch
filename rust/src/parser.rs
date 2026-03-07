//! Parses & validates a patch into a list of "hunks".
//! Ported from codex/codex-rs/apply-patch/src/parser.rs with CLI-specific
//! parts removed and adapted for libaipatch.
//!
//! The official Lark grammar for the apply-patch format is:
//!
//! start: begin_patch hunk+ end_patch
//! begin_patch: "*** Begin Patch" LF
//! end_patch: "*** End Patch" LF?
//!
//! hunk: add_hunk | delete_hunk | update_hunk
//! add_hunk: "*** Add File: " filename LF add_line+
//! delete_hunk: "*** Delete File: " filename LF
//! update_hunk: "*** Update File: " filename LF change_move? change?
//! filename: /(.+)/
//! add_line: "+" /(.+)/ LF -> line
//!
//! change_move: "*** Move to: " filename LF
//! change: (change_context | change_line)+ eof_line?
//! change_context: ("@@" | "@@ " /(.+)/) LF
//! change_line: ("+" | "-" | " ") /(.+)/ LF
//! eof_line: "*** End of File" LF
//!
//! The parser is a little more lenient than the explicit spec and allows for
//! leading/trailing whitespace around patch markers.

use std::path::{Path, PathBuf};

use thiserror::Error;

const BEGIN_PATCH_MARKER: &str = "*** Begin Patch";
const END_PATCH_MARKER: &str = "*** End Patch";
const ADD_FILE_MARKER: &str = "*** Add File: ";
const DELETE_FILE_MARKER: &str = "*** Delete File: ";
const UPDATE_FILE_MARKER: &str = "*** Update File: ";
const MOVE_TO_MARKER: &str = "*** Move to: ";
const EOF_MARKER: &str = "*** End of File";
const CHANGE_CONTEXT_MARKER: &str = "@@ ";
const EMPTY_CHANGE_CONTEXT_MARKER: &str = "@@";

/// We always use lenient mode (compatible with gpt-4.1 heredoc style patches).
const PARSE_IN_STRICT_MODE: bool = false;

#[derive(Debug, PartialEq, Error, Clone)]
pub enum ParseError {
    #[error("invalid patch: {0}")]
    InvalidPatchError(String),
    #[error("invalid hunk at line {line_number}, {message}")]
    InvalidHunkError { message: String, line_number: usize },
}
use ParseError::*;

fn format_agent_error(tag: &str, hint: &str, detail: impl Into<String>) -> String {
    format!("tag: {tag}\nhint: {hint}\ndetail: {}", detail.into())
}

fn invalid_patch_message(tag: &str, hint: &str, detail: impl Into<String>) -> ParseError {
    InvalidPatchError(format_agent_error(tag, hint, detail))
}

#[derive(Debug, PartialEq, Clone)]
#[allow(clippy::enum_variant_names)]
pub enum Hunk {
    AddFile {
        path: PathBuf,
        contents: String,
    },
    DeleteFile {
        path: PathBuf,
    },
    UpdateFile {
        path: PathBuf,
        move_path: Option<PathBuf>,
        /// Chunks should be in order, i.e. the `change_context` of one chunk
        /// should occur later in the file than the previous chunk.
        chunks: Vec<UpdateFileChunk>,
    },
}

impl Hunk {
    pub fn path(&self) -> &Path {
        match self {
            Hunk::AddFile { path, .. } => path.as_path(),
            Hunk::DeleteFile { path } => path.as_path(),
            Hunk::UpdateFile { path, .. } => path.as_path(),
        }
    }

    pub fn move_path(&self) -> Option<&Path> {
        match self {
            Hunk::UpdateFile {
                move_path: Some(mp),
                ..
            } => Some(mp.as_path()),
            _ => None,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct UpdateFileChunk {
    /// A single line of context used to narrow down the position of the chunk.
    pub change_context: Option<String>,
    /// A contiguous block of lines that should be replaced with `new_lines`.
    pub old_lines: Vec<String>,
    pub new_lines: Vec<String>,
    /// If true, `old_lines` must occur at the end of the source file.
    pub is_end_of_file: bool,
}

/// The result of parsing a patch.
#[derive(Debug, PartialEq)]
pub struct ParsedPatch {
    pub hunks: Vec<Hunk>,
}

pub fn parse_patch(patch: &str) -> Result<ParsedPatch, ParseError> {
    let mode = if PARSE_IN_STRICT_MODE {
        ParseMode::Strict
    } else {
        ParseMode::Lenient
    };
    parse_patch_text(patch, mode)
}

enum ParseMode {
    Strict,
    /// Lenient mode: strips heredoc wrappers like `<<'EOF'...EOF` if present.
    Lenient,
}

fn parse_patch_text(patch: &str, mode: ParseMode) -> Result<ParsedPatch, ParseError> {
    let lines: Vec<&str> = patch.trim().lines().collect();
    let lines: &[&str] = match check_patch_boundaries_strict(&lines) {
        Ok(()) => &lines,
        Err(e) => match mode {
            ParseMode::Strict => {
                return Err(e);
            }
            ParseMode::Lenient => check_patch_boundaries_lenient(&lines, e)?,
        },
    };

    let mut hunks: Vec<Hunk> = Vec::new();
    let last_line_index = lines.len().saturating_sub(1);
    let mut remaining_lines = &lines[1..last_line_index];
    let mut line_number = 2;
    while !remaining_lines.is_empty() {
        let (hunk, hunk_lines) = parse_one_hunk(remaining_lines, line_number)?;
        hunks.push(hunk);
        line_number += hunk_lines;
        remaining_lines = &remaining_lines[hunk_lines..]
    }
    Ok(ParsedPatch { hunks })
}

fn check_patch_boundaries_strict(lines: &[&str]) -> Result<(), ParseError> {
    let (first_line, last_line) = match lines {
        [] => (None, None),
        [first] => (Some(first), Some(first)),
        [first, .., last] => (Some(first), Some(last)),
    };
    check_start_and_end_lines_strict(first_line, last_line)
}

fn check_patch_boundaries_lenient<'a>(
    original_lines: &'a [&'a str],
    original_parse_error: ParseError,
) -> Result<&'a [&'a str], ParseError> {
    match original_lines {
        [first, .., last] => {
            if (first == &"<<EOF" || first == &"<<'EOF'" || first == &"<<\"EOF\"")
                && last.ends_with("EOF")
                && original_lines.len() >= 4
            {
                let inner_lines = &original_lines[1..original_lines.len() - 1];
                match check_patch_boundaries_strict(inner_lines) {
                    Ok(()) => Ok(inner_lines),
                    Err(e) => Err(e),
                }
            } else {
                Err(original_parse_error)
            }
        }
        _ => Err(original_parse_error),
    }
}

fn check_start_and_end_lines_strict(
    first_line: Option<&&str>,
    last_line: Option<&&str>,
) -> Result<(), ParseError> {
    let first_line = first_line.map(|line| line.trim());
    let last_line = last_line.map(|line| line.trim());

    match (first_line, last_line) {
        (Some(first), Some(last)) if first == BEGIN_PATCH_MARKER && last == END_PATCH_MARKER => {
            Ok(())
        }
        (Some(first), _) if first != BEGIN_PATCH_MARKER => Err(invalid_patch_message(
            "parse.patch.missing_begin",
            "start the patch with the exact line '*** Begin Patch'",
            "The first line of the patch must be '*** Begin Patch'",
        )),
        _ => Err(invalid_patch_message(
            "parse.patch.missing_end",
            "end the patch with the exact line '*** End Patch'",
            "The last line of the patch must be '*** End Patch'",
        )),
    }
}

/// Attempts to parse a single hunk from the start of lines.
/// Returns the parsed hunk and the number of lines parsed (or a ParseError).
fn parse_one_hunk(lines: &[&str], line_number: usize) -> Result<(Hunk, usize), ParseError> {
    let first_line = lines[0].trim();
    if let Some(path) = first_line.strip_prefix(ADD_FILE_MARKER) {
        let mut contents = String::new();
        let mut parsed_lines = 1;
        for add_line in &lines[1..] {
            if let Some(line_to_add) = add_line.strip_prefix('+') {
                contents.push_str(line_to_add);
                contents.push(0x0A as char);
                parsed_lines += 1;
            } else {
                break;
            }
        }
        if parsed_lines == 1 {
            return Err(InvalidHunkError {
                message: format_agent_error(
                    "parse.add_file.empty",
                    "content lines must start immediately after the header, and every file line, including blank lines, must begin with '+'",
                    format!("Add file hunk for path '{path}' is empty"),
                ),
                line_number,
            });
        }
        return Ok((
            Hunk::AddFile {
                path: PathBuf::from(path),
                contents,
            },
            parsed_lines,
        ));
    } else if let Some(path) = first_line.strip_prefix(DELETE_FILE_MARKER) {
        return Ok((
            Hunk::DeleteFile {
                path: PathBuf::from(path),
            },
            1,
        ));
    } else if let Some(path) = first_line.strip_prefix(UPDATE_FILE_MARKER) {
        let mut remaining_lines = &lines[1..];
        let mut parsed_lines = 1;

        let move_path = remaining_lines
            .first()
            .and_then(|x| x.strip_prefix(MOVE_TO_MARKER));

        if move_path.is_some() {
            remaining_lines = &remaining_lines[1..];
            parsed_lines += 1;
        }

        let mut chunks = Vec::new();
        while !remaining_lines.is_empty() {
            if remaining_lines[0].trim().is_empty() {
                parsed_lines += 1;
                remaining_lines = &remaining_lines[1..];
                continue;
            }
            if remaining_lines[0].starts_with("***") {
                break;
            }
            let (chunk, chunk_lines) = parse_update_file_chunk(
                remaining_lines,
                line_number + parsed_lines,
                chunks.is_empty(),
            )?;
            chunks.push(chunk);
            parsed_lines += chunk_lines;
            remaining_lines = &remaining_lines[chunk_lines..]
        }

        if chunks.is_empty() {
            let (tag, hint) = if move_path.is_some() {
                (
                    "parse.update_file.empty_after_move",
                    "after '*** Move to:' the patch still needs a non-empty '@@' hunk; for a pure rename, the current format requires at least one context line",
                )
            } else {
                (
                    "parse.update_file.empty",
                    "an update hunk must contain a non-empty '@@' section with context, additions, or removals",
                )
            };
            return Err(InvalidHunkError {
                message: format_agent_error(
                    tag,
                    hint,
                    format!("Update file hunk for path '{path}' is empty"),
                ),
                line_number,
            });
        }

        return Ok((
            Hunk::UpdateFile {
                path: PathBuf::from(path),
                move_path: move_path.map(PathBuf::from),
                chunks,
            },
            parsed_lines,
        ));
    }

    Err(InvalidHunkError {
        message: format!(
            "'{first_line}' is not a valid hunk header. Valid hunk headers: '*** Add File: {{path}}', '*** Delete File: {{path}}', '*** Update File: {{path}}'"
        ),
        line_number,
    })
}

fn parse_update_file_chunk(
    lines: &[&str],
    line_number: usize,
    allow_missing_context: bool,
) -> Result<(UpdateFileChunk, usize), ParseError> {
    if lines.is_empty() {
        return Err(InvalidHunkError {
            message: format_agent_error(
                "parse.update_chunk.empty",
                "start the chunk with '@@' or '@@ <context>' and include at least one context, added, or removed line after it",
                "Update hunk does not contain any lines",
            ),
            line_number,
        });
    }
    let (change_context, start_index) = if lines[0] == EMPTY_CHANGE_CONTEXT_MARKER {
        (None, 1)
    } else if let Some(context) = lines[0].strip_prefix(CHANGE_CONTEXT_MARKER) {
        (Some(context.to_string()), 1)
    } else {
        if !allow_missing_context {
            return Err(InvalidHunkError {
                message: format_agent_error(
                    "parse.update_chunk.missing_context_marker",
                    "start each non-initial update chunk with '@@' or '@@ <context>' before any ' ', '+', or '-' lines",
                    format!(
                        "Expected update hunk to start with a @@ context marker, got: '{}'",
                        lines[0]
                    ),
                ),
                line_number,
            });
        }
        (None, 0)
    };
    if start_index >= lines.len() {
        return Err(InvalidHunkError {
            message: format_agent_error(
                "parse.update_chunk.empty",
                "after '@@' include at least one context, added, or removed line",
                "Update hunk does not contain any lines",
            ),
            line_number: line_number + 1,
        });
    }
    let mut chunk = UpdateFileChunk {
        change_context,
        old_lines: Vec::new(),
        new_lines: Vec::new(),
        is_end_of_file: false,
    };
    let mut parsed_lines = 0;
    for line in &lines[start_index..] {
        match *line {
            EOF_MARKER => {
                if parsed_lines == 0 {
                    return Err(InvalidHunkError {
                        message: format_agent_error(
                            "parse.update_chunk.empty",
                            "'*** End of File' can appear only after at least one context, added, or removed line in the chunk",
                            "Update hunk does not contain any lines",
                        ),
                        line_number: line_number + 1,
                    });
                }
                chunk.is_end_of_file = true;
                parsed_lines += 1;
                break;
            }
            line_contents => {
                match line_contents.chars().next() {
                    None => {
                        chunk.old_lines.push(String::new());
                        chunk.new_lines.push(String::new());
                    }
                    Some(' ') => {
                        chunk.old_lines.push(line_contents[1..].to_string());
                        chunk.new_lines.push(line_contents[1..].to_string());
                    }
                    Some('+') => {
                        chunk.new_lines.push(line_contents[1..].to_string());
                    }
                    Some('-') => {
                        chunk.old_lines.push(line_contents[1..].to_string());
                    }
                    _ => {
                        if parsed_lines == 0 {
                            return Err(InvalidHunkError {
                                message: format_agent_error(
                                    "parse.update_chunk.invalid_line",
                                    "after '@@', every payload line must start with ' ' for context, '+' for additions, or '-' for removals",
                                    format!(
                                        "Unexpected line found in update hunk: '{line_contents}'. Every line should start with ' ' (context line), '+' (added line), or '-' (removed line)"
                                    ),
                                ),
                                line_number: line_number + 1,
                            });
                        }
                        break;
                    }
                }
                parsed_lines += 1;
            }
        }
    }

    Ok((chunk, parsed_lines + start_index))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic_add() {
        let patch = "*** Begin Patch\n*** Add File: foo.txt\n+hello\n+world\n*** End Patch";
        let result = parse_patch(patch).unwrap();
        assert_eq!(result.hunks.len(), 1);
        match &result.hunks[0] {
            Hunk::AddFile { path, contents } => {
                assert_eq!(path, &PathBuf::from("foo.txt"));
                assert_eq!(contents, "hello\nworld\n");
            }
            _ => panic!("expected AddFile"),
        }
    }

    #[test]
    fn test_parse_basic_delete() {
        let patch = "*** Begin Patch\n*** Delete File: old.txt\n*** End Patch";
        let result = parse_patch(patch).unwrap();
        assert_eq!(result.hunks.len(), 1);
        match &result.hunks[0] {
            Hunk::DeleteFile { path } => {
                assert_eq!(path, &PathBuf::from("old.txt"));
            }
            _ => panic!("expected DeleteFile"),
        }
    }

    #[test]
    fn test_parse_update_with_move() {
        let patch = "*** Begin Patch\n*** Update File: src.py\n*** Move to: dst.py\n@@\n-old\n+new\n*** End Patch";
        let result = parse_patch(patch).unwrap();
        assert_eq!(result.hunks.len(), 1);
        match &result.hunks[0] {
            Hunk::UpdateFile {
                path,
                move_path,
                chunks,
            } => {
                assert_eq!(path, &PathBuf::from("src.py"));
                assert_eq!(move_path, &Some(PathBuf::from("dst.py")));
                assert_eq!(chunks.len(), 1);
                assert_eq!(chunks[0].old_lines, vec!["old"]);
                assert_eq!(chunks[0].new_lines, vec!["new"]);
            }
            _ => panic!("expected UpdateFile"),
        }
    }

    #[test]
    fn test_parse_invalid_missing_begin() {
        let err = parse_patch("bad patch").unwrap_err();
        match err {
            ParseError::InvalidPatchError(message) => {
                assert!(message.contains("tag: parse.patch.missing_begin"));
                assert!(message.contains("*** Begin Patch"));
            }
            _ => panic!("expected InvalidPatchError"),
        }
    }

    #[test]
    fn test_parse_empty_patch() {
        let patch = "*** Begin Patch\n*** End Patch";
        let result = parse_patch(patch).unwrap();
        assert_eq!(result.hunks.len(), 0);
    }

    #[test]
    fn test_parse_empty_add_file_rejected() {
        let patch = "*** Begin Patch\n*** Add File: empty.txt\n*** End Patch";
        let err = parse_patch(patch).unwrap_err();
        match err {
            ParseError::InvalidHunkError { message, .. } => {
                assert!(message.contains("tag: parse.add_file.empty"));
                assert!(message.contains("must begin with '+'"));
            }
            _ => panic!("expected InvalidHunkError"),
        }
    }

    #[test]
    fn test_parse_empty_update_after_move_has_hint() {
        let patch = "*** Begin Patch\n*** Update File: src.txt\n*** Move to: dst.txt\n*** End Patch";
        let err = parse_patch(patch).unwrap_err();
        match err {
            ParseError::InvalidHunkError { message, .. } => {
                assert!(message.contains("tag: parse.update_file.empty_after_move"));
                assert!(message.contains("pure rename"));
            }
            _ => panic!("expected InvalidHunkError"),
        }
    }

    #[test]
    fn test_parse_invalid_update_line_has_hint() {
        let patch = "*** Begin Patch\n*** Update File: src.txt\n@@\nwat\n*** End Patch";
        let err = parse_patch(patch).unwrap_err();
        match err {
            ParseError::InvalidHunkError { message, .. } => {
                assert!(message.contains("tag: parse.update_chunk.invalid_line"));
                assert!(message.contains("must start with ' '"));
            }
            _ => panic!("expected InvalidHunkError"),
        }
    }

    #[test]
    fn test_parse_end_of_file_marker() {
        let patch = "*** Begin Patch\n*** Update File: f.txt\n@@\n-line\n+line2\n*** End of File\n*** End Patch";
        let result = parse_patch(patch).unwrap();
        match &result.hunks[0] {
            Hunk::UpdateFile { chunks, .. } => {
                assert!(chunks[0].is_end_of_file);
            }
            _ => panic!(),
        }
    }
}
