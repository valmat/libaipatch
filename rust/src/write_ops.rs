//! Safe write operations for the commit phase of applying a patch.
//!
//! All writes happen only after the full validation phase (check) has passed.

use std::path::Path;

use crate::errors::{io_error, AiPatchError};

/// Write a new file with the given contents.
/// Creates parent directories as needed.
pub(crate) fn commit_add(path: &Path, contents: &str) -> Result<(), AiPatchError> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)
                .map_err(|e| io_error(format!("create parent dirs for {}", path.display()), e))?;
        }
    }
    std::fs::write(path, contents)
        .map_err(|e| io_error(format!("write file {}", path.display()), e))
}

/// Update a file in place (or move it to a new path) with new contents.
/// Uses a temp file + rename for atomicity where possible.
pub(crate) fn commit_update(
    path: &Path,
    new_content: &str,
    move_path: Option<&Path>,
) -> Result<(), AiPatchError> {
    let dest = move_path.unwrap_or(path);

    // Ensure the destination's parent directory exists.
    if let Some(parent) = dest.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)
                .map_err(|e| io_error(format!("create parent dirs for {}", dest.display()), e))?;
        }
    }

    // Write to a temporary file next to the destination, then rename.
    // This provides atomic-ish replacement on the same filesystem.
    let dest_dir = dest
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));

    let tmp_path = write_to_tempfile(dest_dir, new_content)?;

    // Rename temp → destination.
    std::fs::rename(&tmp_path, dest).map_err(|e| {
        // Clean up temp file on failure.
        let _ = std::fs::remove_file(&tmp_path);
        io_error(
            format!(
                "rename temp file to {}",
                dest.display()
            ),
            e,
        )
    })?;

    // If this was a move, remove the original source.
    if move_path.is_some() {
        std::fs::remove_file(path)
            .map_err(|e| io_error(format!("remove source file {}", path.display()), e))?;
    }

    Ok(())
}

/// Delete a file.
pub(crate) fn commit_delete(path: &Path) -> Result<(), AiPatchError> {
    std::fs::remove_file(path)
        .map_err(|e| io_error(format!("delete file {}", path.display()), e))
}

/// Write content to a freshly created temp file in `dir`, returning its path.
fn write_to_tempfile(dir: &Path, content: &str) -> Result<std::path::PathBuf, AiPatchError> {
    use std::io::Write;

    // Generate a unique temp name using process id + a counter.
    static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let id = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let tmp_name = format!(".aipatch_tmp_{}_{}", std::process::id(), id);
    let tmp_path = dir.join(tmp_name);

    let mut file = std::fs::File::create(&tmp_path)
        .map_err(|e| io_error(format!("create temp file {}", tmp_path.display()), e))?;

    file.write_all(content.as_bytes())
        .map_err(|e| io_error(format!("write temp file {}", tmp_path.display()), e))?;

    Ok(tmp_path)
}
