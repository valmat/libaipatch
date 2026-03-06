#![allow(dead_code)]
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static COUNTER: AtomicU64 = AtomicU64::new(0);

pub struct TempDir {
    path: PathBuf,
}

impl TempDir {
    pub fn new() -> std::io::Result<Self> {
        let id = COUNTER.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "libaipatch-itest-{}-{}",
            std::process::id(),
            id
        ));
        if path.exists() {
            std::fs::remove_dir_all(&path)?;
        }
        std::fs::create_dir_all(&path)?;
        Ok(Self { path })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Entry {
    File(Vec<u8>),
    Dir,
}

pub fn snapshot_dir(root: &Path) -> std::io::Result<BTreeMap<PathBuf, Entry>> {
    let mut entries = BTreeMap::new();
    if root.is_dir() {
        snapshot_dir_recursive(root, root, &mut entries)?;
    }
    Ok(entries)
}

fn snapshot_dir_recursive(
    base: &Path,
    dir: &Path,
    entries: &mut BTreeMap<PathBuf, Entry>,
) -> std::io::Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let rel = path.strip_prefix(base).unwrap().to_path_buf();
        let metadata = std::fs::metadata(&path)?;
        if metadata.is_dir() {
            entries.insert(rel.clone(), Entry::Dir);
            snapshot_dir_recursive(base, &path, entries)?;
        } else if metadata.is_file() {
            entries.insert(rel, Entry::File(std::fs::read(&path)?));
        }
    }
    Ok(())
}

pub fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    if !src.is_dir() {
        return Ok(());
    }

    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let dest_path = dst.join(entry.file_name());
        let metadata = std::fs::metadata(&path)?;
        if metadata.is_dir() {
            std::fs::create_dir_all(&dest_path)?;
            copy_dir_recursive(&path, &dest_path)?;
        } else if metadata.is_file() {
            if let Some(parent) = dest_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::copy(&path, &dest_path)?;
        }
    }

    Ok(())
}
