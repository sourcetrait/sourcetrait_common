//! Compares differences between two paths
use std::{fmt::Display, fs::File, io::{self, BufReader, Read}, path::{Path, PathBuf}};
use walkdir::{self, WalkDir};
use srctrait_common_stdx::path::tree::{PathTree, PathTreeTrait};


/// Compares differences between two subject paths.
pub fn path_diff<P1,P2>(first: P1, second: P2) -> io::Result<Option<Vec<Difference>>>
where
    P1: AsRef<Path>,
    P2: AsRef<Path>
{
    _path_diff(first.as_ref(), second.as_ref(), false)
}

/// Determines whether two paths differ or not
///
/// Returns immediately after the first difference is found.
pub fn paths_differ<P1,P2>(first: P1, second: P2) -> io::Result<bool>
where
    P1: AsRef<Path>,
    P2: AsRef<Path>
{
    _path_diff(first.as_ref(), second.as_ref(), true)
        .map(|o| o.is_some())
}

/// The subject path in order as passed by argument
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Subject {
    First,
    Second
}

impl Display for Subject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Subject::First => write!(f, "first"),
            Subject::Second => write!(f, "second"),
        }
    }
}

/// Describes a difference between two [Subject] paths.
///
/// All paths are relative to the subjects,
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Difference {
    /// One argument is a file and the other is a directory
    SubjectTypesDiffer,
    /// One path is a file and the other is a directory
    TypesDiffer(PathBuf),
    /// [Subject] is missing a file that the other has
    FileMissing(PathBuf, Subject),
    /// [Subject] is missing a directory that the other has
    DirectoryMissing(PathBuf, Subject),
    /// File is different between subjects
    FileDiffers(PathBuf),
}

impl Display for Difference {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Difference::SubjectTypesDiffer => write!(f, "One argument is a file and the other is a directory"),
            Difference::TypesDiffer(path) => write!(f, "Path type differs: {}", path.display()),
            Difference::FileMissing(path, subject) => write!(f, "File is missing in {subject}: {}", path.display()),
            Difference::DirectoryMissing(path, subject) => write!(f, "Directory is missing in {subject}: {}", path.display()),
            Difference::FileDiffers(path) => write!(f, "File differs: {}", path.display()),
        }
    }
}

/// early: returns on first difference
fn _path_diff(first: &Path, second: &Path, early: bool)  -> io::Result<Option<Vec<Difference>>> {
    if !first.exists() {
        return Err(io::Error::new(io::ErrorKind::NotFound, format!("First path not found: {}", first.display())))
    } else if !second.exists() {
        return Err(io::Error::new(io::ErrorKind::NotFound, format!("Second path not found: {}", second.display())))
    }

    match (first.is_dir(), second.is_dir()) {
        (false, false) => {
            return compare_files(second, first, second)
                .map(|o| o.map(|d| vec![d]))
        },
        (true, false) => return Ok(Some(vec![Difference::SubjectTypesDiffer])),
        (false, true) => return Ok(Some(vec![Difference::SubjectTypesDiffer])),
        (true, true) => {},
    }

    let mut differences: Vec<Difference> = Vec::new();
    let mut paths_compared = PathTree::new_relative();
    let mut paths_missing = Vec::new();

    for first_entry in walk(first).into_iter() {
        let first_entry = first_entry.map_err(|e| walk_err(e))?;
        let relpath = rel(first, first_entry.path());
        let second_path = second.join(&relpath);

        paths_compared.insert(&relpath);

        if let Some(parent) = second_path.parent() {
            let parent_rel = rel(second, parent);
            if parent != second && is_path_missing(&parent_rel, &paths_missing) {
                continue;
            }
        }

        let second_meta = match second_path.metadata() {
            Ok(m) => m,
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                if first_entry.file_type().is_dir() {
                    paths_missing.push(relpath.to_path_buf());
                    differences.push(Difference::DirectoryMissing(relpath.to_path_buf(), Subject::Second));
                    match early {
                        true => return Ok(Some(differences)),
                        false => continue
                    }
                } else {
                    paths_missing.push(relpath.to_path_buf());
                    differences.push(Difference::FileMissing(relpath.to_path_buf(), Subject::Second));
                    match early {
                        true => return Ok(Some(differences)),
                        false => continue
                    }
                }
            },
            Err(e) => return Err(e)
        };

        let first_is_dir = first_entry.file_type().is_dir();
        let second_is_dir = second_meta.is_dir();
        match (first_is_dir, second_is_dir) {
            (true, true) => {},
            (true, false) => {
                differences.push(Difference::TypesDiffer(relpath.to_path_buf()));
                paths_missing.push(relpath.to_path_buf());
                if early {
                    return Ok(Some(differences));
                }
            },
            (false, true) => {
                differences.push(Difference::TypesDiffer(relpath.to_path_buf()));
                paths_missing.push(relpath.to_path_buf());
                if early {
                    return Ok(Some(differences));
                }
            },
            (false, false) => {
                let diff = compare_files(&relpath, first_entry.path(), &second_path)?;
                if let Some(diff) = diff {
                    differences.push(diff);
                    if early {
                        return Ok(Some(differences));
                    }
                }
            },
        }
    }

    for second_entry in walk(second).into_iter() {
        let second_entry = second_entry.map_err(|e| walk_err(e))?;
        let relpath = rel(second, second_entry.path());
        if paths_compared.contains(&relpath) {
            continue;
        }

        let first_path = first.join(&relpath);

        if let Some(parent) = first_path.parent() {
            let parent_rel = rel(first, parent);
            if parent != first && is_path_missing(&parent_rel, &paths_missing) {
                continue;
            }
        }

        match first_path.metadata() {
            Ok(_) => unreachable!(),
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                if second_entry.file_type().is_dir() {
                    differences.push(Difference::DirectoryMissing(relpath.to_path_buf(), Subject::First));
                    paths_missing.push(relpath.to_path_buf());
                    match early {
                        true => return Ok(Some(differences)),
                        false => continue
                    }
                } else {
                    differences.push(Difference::FileMissing(relpath.to_path_buf(), Subject::First));
                    match early {
                        true => return Ok(Some(differences)),
                        false => continue
                    }
                }
            },
            Err(e) => return Err(e)
        }
    }

    if differences.is_empty() {
        Ok(None)
    } else {
        Ok(Some(differences))
    }
}

fn is_path_missing(path: &Path, paths_missing: &Vec<PathBuf>) -> bool {
    for path_missing in paths_missing {
        if path.strip_prefix(path_missing).is_ok() {
            return true;
        }
    }

    false
}

fn walk_err(e: walkdir::Error) -> io::Error {
    if e.io_error().is_some() {
        e.into_io_error().unwrap()
    } else {
        io::Error::new(io::ErrorKind::TooManyLinks, e.to_string())
    }
}

fn walk(dir: &Path) -> WalkDir {
    WalkDir::new(dir)
        .follow_links(true)
        .sort_by_file_name()
}

fn compare_files(relpath: &Path, first: &Path, second: &Path) -> io::Result<Option<Difference>> {
    const MAX_BUF_SIZE: usize = 8388608; // MiB

    let (first_file, second_file) = match (File::open(first), File::open(second)) {
        (Ok(_), Err(e)) => return match e.kind() {
            io::ErrorKind::NotFound => Ok(Some(Difference::FileMissing(relpath.to_path_buf(), Subject::Second))),
            _ => Err(e)
        },
        (Err(e), Ok(_)) => return match e.kind() {
            io::ErrorKind::NotFound => Ok(Some(Difference::FileMissing(relpath.to_path_buf(), Subject::First))),
            _ => Err(e)
        },
        (Err(e1), Err(e2)) => return match e1.kind() {
            io::ErrorKind::NotFound => Err(e2),
            _ => Err(e1)
        },
        (Ok(first), Ok(second)) => (first, second),
    };

    let mut remaining = first.metadata()?.len() as usize;
    if remaining != second.metadata()?.len() as usize {
        return Ok(Some(Difference::FileDiffers(relpath.to_path_buf())));
    }

    let mut first_buf_reader = BufReader::new(first_file);
    let mut second_buf_reader = BufReader::new(second_file);

    while remaining > 0 {
        let buf_size = std::cmp::min(MAX_BUF_SIZE, remaining);
        let mut first_buf = vec![0; buf_size];
        let mut second_buf = vec![0; buf_size];

        first_buf_reader.read_exact(&mut first_buf)?;
        second_buf_reader.read_exact(&mut second_buf)?;

        if first_buf != second_buf {
            return Ok(Some(Difference::FileDiffers(relpath.to_path_buf())));
        }

        remaining -= buf_size;
    }

    Ok(None)
}

fn rel(base: &Path, child: &Path) -> PathBuf {
    child.strip_prefix(base)
        .map_or_else(|_| child.to_path_buf(), |p| p.to_path_buf())
}
