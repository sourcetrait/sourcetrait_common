//! Normalizes paths beyond what std::path does. Namely, this means
//! that '..' is normalized, which, as std::path documentation points out,
//! could be somewhere else on the system due to symlinking.
//!
//! This is useful primarily for sanitizing paths from user input.
use std::path::{Component, Path, PathBuf};

/// Normalizes paths beyond what std::path does. Namely, this means
/// that '..' is normalized, which, as std::path documentation points out,
/// could be somewhere else on the system due to symlinking.
///
/// This is useful primarily for sanitizing paths from user input.
pub trait NormalizePath
where
    Self: AsRef<Path>
{
    /// Normalizes path components like '.' and '..'. Any traversal beyond '.'
    /// returns None. Use of of root or prefixes returns None.
    fn normalize_relative(&self) -> Option<PathBuf> {
        let mut components = self.as_ref().components();
        let mut normalized = PathBuf::new();

        while let Some(c) = components.next() {
            match c {
                Component::RootDir => return None,
                Component::Prefix(_) => return None,
                Component::CurDir => continue,
                Component::Normal(os_str) => normalized.push(os_str),
                Component::ParentDir => if !normalized.pop() {
                    return None;
                },
            }
        }

        Some(normalized)
    }
    
    /// Normalizes path components like '.' and '..'. Any traversal beyond '.'
    /// returns None. Use of of root or prefixes returns None. Joins the
    /// normalized path to the specified dir.
    fn normalize_relative_to(&self, dir: &Path) -> Option<PathBuf> {
        self.normalize_relative().map(|p| dir.join(p))
    }
}

impl NormalizePath for PathBuf {}
impl NormalizePath for Path {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_root() {
        assert_eq!(None, Path::new("/").normalize_relative());
        assert_eq!(None, Path::new("/foo").normalize_relative());
    }

    #[test]
    fn test_no_parent() {
        assert_eq!(None, Path::new("..").normalize_relative());
        assert_eq!(None, Path::new("foo/../..").normalize_relative());
        assert_eq!(None, Path::new("foo/bar/../../..").normalize_relative());
    }

    #[test]
    fn test_parent() {
        assert_eq!(Some(PathBuf::new()), Path::new("foo/..").normalize_relative());
        assert_eq!(Some(PathBuf::from("foo")), Path::new("foo/bar/..").normalize_relative());
        assert_eq!(Some(PathBuf::from("foo/cat")), Path::new("foo/bar/../cat").normalize_relative());
    }

    #[test]
    fn test_current() {
        assert_eq!(Some(PathBuf::new()), Path::new(".").normalize_relative());
        assert_eq!(Some(PathBuf::from("foo/.")), Path::new("foo").normalize_relative());
        assert_eq!(Some(PathBuf::from("foo/./bar")), Path::new("foo/bar").normalize_relative());
    }
}
