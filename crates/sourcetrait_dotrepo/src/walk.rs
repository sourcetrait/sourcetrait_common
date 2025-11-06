use std::{collections::HashSet, fs, path::Path};
use ignore::gitignore::Gitignore;
use walkdir::WalkDir;
use crate::*;

#[derive(Debug, derive_builder::Builder)]
#[builder(pattern = "owned")]
#[builder(default)]
pub struct WalkOff {
    sort_by: SortBy,
    follow_links: bool,
    filter_excludes: bool,
    skip_git_dirs: bool,
    skip_gitignore_files: bool,
    skip_dotrepo_dirs: bool,
    skip_dotrepo_self: bool,
}

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub enum SortBy {
    #[default]
    None,
    Name
}

impl WalkOff {
    pub fn builder() -> WalkOffBuilder {
        WalkOffBuilder::default()
    }
    
    pub(crate) fn using_excludes(&self) -> bool {
        self.filter_excludes
    }
}

impl Default for WalkOff {
    fn default() -> Self {
        Self {
            sort_by: SortBy::Name,
            follow_links: true,
            filter_excludes: true,
            skip_git_dirs: true,
            skip_gitignore_files: true,
            skip_dotrepo_dirs: true,
            skip_dotrepo_self: true,
        }
    }
}

#[derive(Debug)]
pub struct TenantWalkOff<R: 'static + DotRepoType> {
    all_standard_kind: Option<HashSet<StandardDesignatorKind>>,
    all_kind: Option<HashSet<R::DesignatorKind>>,
    all_standard: Option<HashSet<StandardDesignator>>,
    all: Option<HashSet<R::Designator>>,
    any_standard_kind: Option<HashSet<StandardDesignatorKind>>,
    any_kind: Option<HashSet<R::DesignatorKind>>,
    any_standard: Option<HashSet<StandardDesignator>>,
    any: Option<HashSet<R::Designator>>,
    is_some: bool
}

#[derive(Debug)]
pub struct TenantWalkOffBuilder<R: 'static + DotRepoType> {
    all_standard_kind: Option<HashSet<StandardDesignatorKind>>,
    all_kind: Option<HashSet<R::DesignatorKind>>,
    all_standard: Option<HashSet<StandardDesignator>>,
    all: Option<HashSet<R::Designator>>,
    any_standard_kind: Option<HashSet<StandardDesignatorKind>>,
    any_kind: Option<HashSet<R::DesignatorKind>>,
    any_standard: Option<HashSet<StandardDesignator>>,
    any: Option<HashSet<R::Designator>>
}

impl<R: 'static + DotRepoType> Default for TenantWalkOffBuilder<R>{
    fn default() -> Self {
        Self {
            all_standard_kind: None,
            all_kind: None,
            all_standard: None,
            all: None,
            any_standard_kind: None,
            any_kind: None,
            any_standard: None,
            any: None,
        }
    }
}

impl<R: 'static + DotRepoType> TenantWalkOffBuilder<R>{
    pub fn all_standard_kind<const N: usize>(mut self, set: [StandardDesignatorKind;N]) -> Self {
        self.all_standard_kind = Some(HashSet::from(set));
        self
    }
    
    pub fn all_kind<const N: usize>(mut self, set: [R::DesignatorKind;N]) -> Self {
        self.all_kind = Some(HashSet::from(set));
        self
    }
    
    pub fn any_standard_kind<const N: usize>(mut self, set: [StandardDesignatorKind;N]) -> Self {
        self.any_standard_kind = Some(HashSet::from(set));
        self
    }
    
    pub fn any_kind<const N: usize>(mut self, set: [R::DesignatorKind;N]) -> Self {
        self.any_kind = Some(HashSet::from(set));
        self
    }
    
    pub fn all_standard<const N: usize>(mut self, set: [StandardDesignator;N]) -> Self {
        self.all_standard = Some(HashSet::from(set));
        self
    }
    
    pub fn all<const N: usize>(mut self, set: [R::Designator;N]) -> Self {
        self.all = Some(HashSet::from(set));
        self
    }
    
    pub fn any_standard<const N: usize>(mut self, set: [StandardDesignator;N]) -> Self {
        self.any_standard = Some(HashSet::from(set));
        self
    }
    
    pub fn any<const N: usize>(mut self, set: [R::Designator;N]) -> Self {
        self.any = Some(HashSet::from(set));
        self
    }
    
    pub fn build(self) -> RepoResult<R, TenantWalkOff<R>> {
        let is_some = self.all.is_some() || self.any.is_some()
            || self.all_standard.is_some() || self.any_standard.is_some()
            || self.all_kind.is_some() || self.any_kind.is_some()
            || self.all_standard_kind.is_some() || self.any_standard_kind.is_some();

        Ok(TenantWalkOff {
            all_standard_kind: self.all_standard_kind,
            all_kind: self.all_kind,
            all_standard: self.all_standard,
            all: self.all,
            any_standard_kind: self.any_standard_kind,
            any_kind: self.any_kind,
            any_standard: self.any_standard,
            any: self.any,
            is_some,
        })
    }
}

impl<R: 'static + DotRepoType> TenantWalkOff<R> {
    pub fn builder() -> TenantWalkOffBuilder<R> {
        TenantWalkOffBuilder::default()
    }
}

impl<R: 'static + DotRepoType> Default for TenantWalkOff<R> {
    fn default() -> Self {
        Self {
            all_standard_kind: None,
            all_kind: None,
            all_standard: None,
            all: None,
            any_standard_kind: None,
            any_kind: None,
            any_standard: None,
            any: None,
            is_some: false,
        }
    }
}

pub(crate) fn build_repository_walker(
    walk_dir: &Path,
    tenant_relative_path: &Path,
    excludes: Option<Gitignore>,
    walkoff: WalkOff
) -> impl Iterator<Item = walkdir::Result<walkdir::DirEntry>> {
    let mut walk = WalkDir::new(walk_dir)
        .follow_links(walkoff.follow_links);
    
    walk = match walkoff.sort_by {
        SortBy::Name => walk.sort_by_file_name(),
        SortBy::None => walk,
    };
    
    let filter_entry = walkoff.skip_dotrepo_dirs
        || walkoff.skip_git_dirs
        || walkoff.skip_dotrepo_self
        || walkoff.filter_excludes;
        
    let repo_relpath = Path::new(DOTREPO).join(tenant_relative_path);
        
    walk.into_iter()
        .filter_entry(move |entry| if filter_entry {
            let filename = entry.file_name();
            let path = entry.path();
            if entry.file_type().is_dir() {
                if walkoff.skip_dotrepo_dirs && filename == DOTREPO {
                    false
                } else if walkoff.skip_git_dirs && filename == DOTGIT {
                    false
                } else if walkoff.skip_dotrepo_self && path.ends_with(&repo_relpath) {
                    false
                } else if walkoff.filter_excludes && excludes.as_ref().expect("excludes").matched(path, true).is_ignore() {
                    false
                } else {
                    true
                }
            } else if walkoff.skip_gitignore_files && filename == DOTGITIGNORE {
                false
            } else if walkoff.filter_excludes && excludes.as_ref().expect("excludes").matched(path, false).is_ignore() {
                false
            } else {
                true
            }
        } else {
            true
        })
        .filter(move |entry| entry.as_ref().is_ok_and(|entry| entry.path() != walk_dir))
}

pub(crate) fn build_tenant_walker<R: 'static + DotRepoType>(
    walk_dir: &Path,
    tenant_relative_path: &Path,
    walkoff: TenantWalkOff<R>,
) -> impl Iterator<Item = walkdir::Result<(walkdir::DirEntry, DesignatorMatches<R>)>> {
    let walk = WalkDir::new(walk_dir)
        .sort_by_file_name();
    
    let repo_relpath = Path::new(DOTREPO).join(tenant_relative_path);
        
    walk.into_iter()
        .filter_entry(|entry| entry.file_type().is_dir())
        .filter_map(move |entry| {
            let entry = if let Ok(entry) = entry {
                entry
            } else {
                return None;
            };
            
            if !entry.path().join(&repo_relpath).is_dir() {
                return None;
            }
            
            let mut matches = DesignatorMatches::builder();
            if walkoff.is_some {
                let designator_dir = entry.path()
                    .join(&repo_relpath)
                    .join(ReservedDirKind::Designator.relative_path());
                
                if let Some(all_std_kind_set) = walkoff.all_standard_kind.as_ref() {
                    for designator_kind in all_std_kind_set {
                        if designator_dir.join(designator_kind.filename()).is_file() {
                            matches.insert_standard_kind(*designator_kind);
                        } else {
                            return None;
                        }
                    }
                }
                
                if let Some(all_kind_set) = walkoff.all_kind.as_ref() {
                    for designator_kind in all_kind_set {
                        if designator_dir.join(designator_kind.filename()).is_file() {
                            matches.insert_tenant_kind(*designator_kind);
                        } else {
                            return None;
                        }
                    }
                }
                
                if let Some(all_std_set) = walkoff.all_standard.as_ref() {
                    for designator in all_std_set {
                        let filepath = designator_dir.join(designator.filename());
                        if !filepath.is_file() {
                            return None;
                        } else if let Some(ident) = designator.identifier() {
                            if !fs::read_to_string(filepath).is_ok_and(|line| line.trim() == ident) {
                                return None;
                            }
                        }
                        
                        matches.insert_standard_designator(designator.clone());
                    }
                }
                
                if let Some(all_set) = walkoff.all.as_ref() {
                    for designator in all_set {
                        let filepath =  designator_dir.join(designator.filename());
                        if !filepath.is_file() {
                            return None;
                        } else if let Some(ident) = designator.identifier() {
                            if !fs::read_to_string(filepath).is_ok_and(|line| line.trim() == ident) {
                                return None;
                            }
                        }
                        
                        matches.insert_tenant_designator(designator.clone());
                    }
                }
                
                if let Some(any_std_kind_set) = walkoff.any_standard_kind.as_ref() {
                    let mut found = false;
                    for designator_kind in any_std_kind_set {
                        if designator_dir.join(designator_kind.filename()).is_file() {
                            matches.insert_standard_kind(*designator_kind);
                            found = true;
                        }
                    };
                    
                    if !found {
                        return None;
                    }
                }
                
                if let Some(any_kind_set) = walkoff.any_kind.as_ref() {
                    let mut found = false;
                    for designator_kind in any_kind_set {
                        if designator_dir.join(designator_kind.filename()).is_file() {
                            matches.insert_tenant_kind(*designator_kind);
                            found = true;
                        }
                    };
                    
                    if !found {
                        return None;
                    }
                }
                
                if let Some(any_std_set) = walkoff.any_standard.as_ref() {
                    let mut found = false;
                    for designator in any_std_set {
                        let filepath =  designator_dir.join(designator.filename());
                        if !filepath.is_file() {
                            continue;
                        } else if let Some(ident) = designator.identifier() {
                            if !fs::read_to_string(filepath).is_ok_and(|line| line.trim() == ident) {
                                continue;
                            }
                        }
                        
                        matches.insert_standard_designator(designator.clone());
                        found = true;
                        break;
                    }
                    
                    if !found {
                        return None;
                    }
                }
                
                if let Some(any_set) = walkoff.any.as_ref() {
                    let mut found = false;
                    for designator in any_set {
                        let filepath =  designator_dir.join(designator.filename());
                        if !filepath.is_file() {
                            continue;
                        } else if let Some(ident) = designator.identifier() {
                            if !fs::read_to_string(filepath).is_ok_and(|line| line.trim() == ident) {
                                continue;
                            }
                        }
                        
                        matches.insert_tenant_designator(designator.clone());
                        found = true;
                        break;
                    }
                    
                    if !found {
                        return None;
                    }
                }
                
                Some(Ok((entry, matches.build().expect("built"))))
            } else {
                Some(Ok((entry, matches.build().expect("built"))))
            }
        })
}
