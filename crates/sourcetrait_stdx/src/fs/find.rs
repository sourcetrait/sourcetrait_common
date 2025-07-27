//! Filesystem find utilities
use std::path::{PathBuf, Path};
use std::fs;

/// Walks up the directory tree, searching for the directory that contains the
/// file named as specified.
pub fn find_parent_file(from_dir: &Path, filename: &str) -> Option<PathBuf> {
    let mut cur_dir = from_dir;
    loop {
        let filepath = cur_dir.join(filename);
        if let Ok(meta) = fs::metadata(&filepath) {
            if meta.is_file() {
                break Some(cur_dir.to_path_buf());
            } else {
                break None;
            }
        }

        cur_dir = match cur_dir.parent() {
            Some(dir) => dir,
            None => break None
        }
    }
}
/// Walks up the directory tree, searching for the directory that contains the
/// sub-directory named as specified.
pub fn find_parent_dir(from_dir: &Path, dir_name: &str) -> Option<PathBuf> {
    if from_dir.file_name().is_some_and(|d| d == dir_name) {
        return Some(from_dir.to_path_buf())
    }

    let mut cur_dir = from_dir;
    loop {
        let path = cur_dir.join(dir_name);
        if let Ok(meta) = fs::metadata(&path) {
            if meta.is_dir() {
                break Some(cur_dir.to_path_buf());
            } else {
                break None;
            }
        }

        cur_dir = match cur_dir.parent() {
            Some(dir) => dir,
            None => break None
        }
    }
}

