//! Utilities for managing a `.repo` directory structure

use std::{ops::Deref, path::{Path, PathBuf}};
use crate::*;

#[derive(Debug, Clone, PartialEq)]
pub struct DotRepoDir(PathBuf);

impl Deref for DotRepoDir {
    type Target = Path;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DotRepoDir {
    pub fn new(current_dir: PathBuf) -> Self {
        Self(current_dir.join(DOTREPO))
    }
    
    pub fn current_dir(&self) -> &Path {
        &self.0.parent().expect("parent exists")
    }
    
    pub fn find_top<'a>(current_dir: &'a Path, tenant_path: &Path) -> Option<&'a Path> {
        let search_dir = PathBuf::from(DOTREPO)
            .join(tenant_path)
            .join(ReservedDirKind::Designator.relative_path())
            .join(StandardDesignatorKind::Top.name());
        
        let mut cur_dir = current_dir;
        loop {
            if cur_dir.join(&search_dir).is_file() {
                return Some(cur_dir)
            }
    
            cur_dir = match cur_dir.parent() {
                Some(dir) => dir,
                None => break None
            }
        }
    }
    
    pub fn tenant<R: 'static + DotRepoType>(
        self,
        definition: &'static Definition<R::DesignatorKind,R::Designator>
    ) -> RepoResult<R, DotRepo<R>> {
        DotRepo::new(self, definition)
    }
}

