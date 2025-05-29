//! Filesystem find utilities
use std::path::{PathBuf, Path};
use std::fs;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum TypicalSearch {
    /// Find a .git directory
    Git,
    /// Find a .repo directory
    Repo,
}

pub const GIT_DIR: &'static str = ".git";
pub const REPO_DIR: &'static str = ".repo";

/// Walks up the directory tree, searching for the directory that contains the
/// search criteria.
pub fn find_parent_typical(from_dir: &Path, typical: TypicalSearch) -> Option<PathBuf> {
    match typical {
        TypicalSearch::Git => find_parent_dir(from_dir, GIT_DIR),
        TypicalSearch::Repo => find_parent_dir(from_dir, REPO_DIR)
    }
}

/// Walks up the directory tree, searching for the directory that contains the
/// file named as specified.
pub fn find_parent_file(from_dir: &Path, filename: &str) -> Option<PathBuf> {
    find_parent_path(from_dir, filename, true)
}

/// Walks up the directory tree, searching for the directory that contains the
/// sub-directory named as specified.
pub fn find_parent_dir(from_dir: &Path, dir_name: &str) -> Option<PathBuf> {
    find_parent_path(from_dir, dir_name, false)
}

fn find_parent_path(from_dir: &Path, name: &str, is_file: bool) -> Option<PathBuf> {
    if !is_file && from_dir.file_name().is_some_and(|d| d == name) {
        return Some(from_dir.to_path_buf())
    }

    let mut cur_dir = from_dir;
    loop {
        let filepath = cur_dir.join(name);
        if let Ok(meta) = fs::metadata(&filepath) {
            if is_file {
                if meta.is_file() {
                    break Some(cur_dir.to_path_buf());
                }
            } else {
                if meta.is_dir() {
                    break Some(cur_dir.to_path_buf());
                }
            }
        }

        cur_dir = match cur_dir.parent() {
            Some(dir) => dir,
            None => break None
        }
    }
}
