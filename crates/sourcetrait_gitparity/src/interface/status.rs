use std::{collections::{hash_map, HashMap}, path::{Path, PathBuf}, sync::Arc};
use crate::*;

pub trait StatusTrait<T> {
    fn into_changes(self) -> HashMap<Arc<PathBuf>, T>;
    fn changes(&self) -> &HashMap<Arc<PathBuf>, T>;
    fn changes_iter(&self) -> hash_map::Iter<'_, Arc<PathBuf>, T>;
}

pub trait PathStatusTrait {
    fn path(&self) -> &Path;
    fn code_x(&self) -> Option<StatusCode>;
    fn code_y(&self) -> Option<StatusCode>;
    fn original_path(&self) -> Option<&Path>;
    
    fn is_conflicted(&self) -> bool {
        return self.code_y() == Some(StatusCode::Unmerged) || self.code_x() == Some(StatusCode::Unmerged);
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum StatusCode {
    Added,
    Deleted,
    Ignored,
    Modified,
    Renamed,
    TypeChanged,
    Unmerged,
    Untracked,
}

impl StatusCode {
    pub fn try_from_char(c: char) -> Result<Option<Self>> {
        match c {
            'A' => Ok(Some(StatusCode::Added)),
            'C' => Ok(Some(StatusCode::Added)),
            'D' => Ok(Some(StatusCode::Deleted)),
            'I' => Ok(Some(StatusCode::Ignored)),
            'M' => Ok(Some(StatusCode::Modified)),
            'R' => Ok(Some(StatusCode::Renamed)),
            'T' => Ok(Some(StatusCode::TypeChanged)),
            'U' => Ok(Some(StatusCode::Unmerged)),
            '?' => Ok(Some(StatusCode::Untracked)),
            ' ' => Ok(None),
            // err on 'X' and 'B' from git diff 
            _ => Err(Error::GitStatusParse)
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Status {
    changes: HashMap<Arc<PathBuf>, PathStatus>
}

impl Status {
    pub fn new(changes: HashMap<Arc<PathBuf>, PathStatus>) -> Self {
        Self { changes }
    }
    
    /// Returns None if there are no conflicts
    pub fn into_conflicts(self) -> Option<Self> {
        let changes: HashMap<Arc<PathBuf>, PathStatus> = self.changes.into_iter()
            .filter(|(_, chg)| chg.is_conflicted())
            .collect();
        
        if changes.is_empty() {
            None
        } else {
            Some(Self { changes} )
        }
    }

    pub fn has_conflicts(&self) -> bool {
        self.changes.iter().any(|(_, chg)| chg.is_conflicted())
    }
    
    pub fn is_unmodified(&self) -> bool {
        self.changes.is_empty()
    }
}

impl Status {
    pub fn from_cli(s: &str) -> Result<Self> {
        let items = s.lines()
            .map(|line| PathStatus::from_cli(line))
            .collect::<Result<Vec<_>>>()?;
        
        let changes = items.into_iter()
            .map(|s| (Arc::clone(&s.path), s))
            .collect::<HashMap<_,_>>();
        
        Ok(Self::new(changes))
    }
}

impl StatusTrait<PathStatus> for Status {
    fn changes(&self) -> &HashMap<Arc<PathBuf>, PathStatus> {
        &self.changes
    }
    
    fn into_changes(self) -> HashMap<Arc<PathBuf>, PathStatus> {
        self.changes
    }
    
    fn changes_iter(&self) -> hash_map::Iter<'_, Arc<PathBuf>, PathStatus> {
        self.changes.iter()
    }
}


#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct PathStatus {
    path: Arc<PathBuf>,
    x: Option<StatusCode>,
    y: Option<StatusCode>,
    original_path: Option<PathBuf>,
}

impl PathStatus {
    fn from_cli(line: &str) -> Result<Self> {
        let mut char_indices = line.char_indices();
        let (_, code_x) = char_indices
            .next()
            .ok_or_else(|| Error::GitStatusParse)?;
        let (_, code_y) = char_indices
            .next()
            .ok_or_else(|| Error::GitStatusParse)?;
        
        let (x,y) = (
            StatusCode::try_from_char(code_x)?,
            StatusCode::try_from_char(code_y)?,
        );
        
        let (path_idx, _) = char_indices
            .next()
            .ok_or_else(|| Error::GitStatusParse)?;
        
        let line = &line[path_idx..].trim();
        
        let items = splitty::split_unquoted_whitespace(line)
            .unwrap_quotes(true)
            .collect::<Vec<_>>();
        
        let (path, original_path) = match items.len() {
            1 => (PathBuf::from(items[0]), None),
            3 => ( 
                PathBuf::from(items[2]),
                Some(PathBuf::from(items[0]))
            ),
            _ => {
                return Err(Error::GitStatusParse);
            }
        };

        let path = Arc::new(path);
        
        Ok(PathStatus {
            path,
            x,
            y,
            original_path,
        })
    }
}

impl PathStatus {
    pub(crate) fn new_arc(
        path: Arc<PathBuf>,
        code_index: Option<StatusCode>,
        code_working_tree: Option<StatusCode>,
        original_path: Option<PathBuf>
    ) -> Self {
        Self {
            path,
            x: code_index,
            y: code_working_tree,
            original_path,
        }
    }
    
    pub fn new(
        path: PathBuf,
        code_index: Option<StatusCode>,
        code_working_tree: Option<StatusCode>,
        original_path: Option<PathBuf>
    ) -> Self {
        let path = Arc::new(path);

        Self {
            path,
            x: code_index,
            y: code_working_tree,
            original_path,
        }
    }
    
    pub fn code_index(&self) -> Option<StatusCode> {
        self.x
    }
    
    pub fn code_working_tree(&self) -> Option<StatusCode> {
        self.y
    }
}

impl PathStatusTrait for PathStatus {
    fn path(&self) -> &Path {
        &self.path
    }

    fn code_x(&self) -> Option<StatusCode> {
        self.x
    }

    fn code_y(&self) -> Option<StatusCode> {
        self.y
    }

    fn original_path(&self) -> Option<&Path> {
        self.original_path.as_deref()
    }
    
}

#[derive(Debug, Clone)]
pub struct DiffStatus {
    pub(crate) changes: HashMap<Arc<PathBuf>, PathDiffStatus>
}

impl DiffStatus {
    pub fn new(changes: HashMap<Arc<PathBuf>, PathDiffStatus>) -> Self {
        Self { changes }
    }
    
    pub fn has_changes(&self) -> bool {
        !self.changes.is_empty()
    }
}

impl StatusTrait<PathDiffStatus> for DiffStatus {
    fn into_changes(self) -> HashMap<Arc<PathBuf>, PathDiffStatus> {
        self.changes
    }

    fn changes(&self) -> &HashMap<Arc<PathBuf>, PathDiffStatus> {
        &self.changes
    }

    fn changes_iter(&self) -> hash_map::Iter<'_, Arc<PathBuf>, PathDiffStatus> {
        self.changes.iter()
    }
}


#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct PathDiffStatus {
    pub(crate) path: Arc<PathBuf>,
    pub(crate) code: StatusCode,
    pub(crate) original_path: Option<PathBuf>,
}

impl PathDiffStatus {
    pub fn new(
        path: PathBuf,
        code: StatusCode,
        original_path: Option<PathBuf>
    ) -> Self {
        let path = Arc::new(path);

        Self {
            path,
            code,
            original_path,
        }
    }

    pub fn code(&self) -> StatusCode {
        self.code
    }
}


impl PathStatusTrait for PathDiffStatus {
    fn path(&self) -> &Path {
        &self.path
    }

    fn code_x(&self) -> Option<StatusCode> {
        Some(self.code)
    }

    fn code_y(&self) -> Option<StatusCode> {
        None
    }

    fn original_path(&self) -> Option<&Path> {
        self.original_path.as_deref()
    }
}

