use std::{borrow::Cow, collections::HashSet, fs, path::{Path, PathBuf}};
use semver::Version as SemVer;
use ignore::gitignore::{Gitignore, GitignoreBuilder};
use strum::IntoEnumIterator;
use crate::*;

pub trait DotRepoType: 'static + std::fmt::Debug + Clone {
    type DesignatorKind: DesignatorKind + DesignatorKindTraits<Self::Designator>;
    type Designator: Designator + DesignatorTraits<Self::DesignatorKind>;
    const DEFINITION: Definition<Self::DesignatorKind, Self::Designator>;
    
    fn new(dotrepo_dir: DotRepoDir) -> RepoResult<Self, DotRepo<Self>> {
        DotRepo::new(dotrepo_dir, &Self::DEFINITION)
    }
}

#[derive(Debug)]
pub struct DotRepo<R: 'static + DotRepoType> {
    dotrepo_dir: DotRepoDir,
    tenant_dir: PathBuf,
    top_dir: PathBuf,
    definition: &'static Definition<R::DesignatorKind, R::Designator>
}

impl<R: 'static + DotRepoType> DotRepo<R> {
    pub fn new(dotrepo_dir: DotRepoDir, definition: &'static Definition<R::DesignatorKind,R::Designator>) -> RepoResult<R, Self> {
        let tenant_path = Path::new(definition.subdir);
        let tenant_dir = dotrepo_dir.join(tenant_path);
        let top_dir = DotRepoDir::find_top(dotrepo_dir.current_dir(), tenant_path)
            .ok_or_else(|| RepoError::Topless(dotrepo_dir.current_dir().into()))?
            .to_path_buf();
        
        Ok(Self {
            dotrepo_dir,
            tenant_dir,
            top_dir,
            definition
        })
    }
    
    pub fn create(&self, current_path: &Path, designated: Option<HashSet<Designated<R::Designator>>>) -> RepoResult<R, Self> {
        Self::init(DotRepoDir::new(self.top_dir.join(current_path)), self.definition, designated)
    }

    pub fn init(
        dotrepo_dir: DotRepoDir,
        definition: &'static Definition<R::DesignatorKind,R::Designator>,
        designated: Option<HashSet<Designated<R::Designator>>>
    ) -> RepoResult<R, Self> {
        let current_dir = dotrepo_dir.current_dir();
        let tenant_path = definition.tenant_path();
        let tenant_dir = dotrepo_dir.join(tenant_path);
        let is_top = designated.as_ref().is_some_and(|set| set.contains(&Designated::Standard(StandardDesignator::Top)));
        let top_dir = DotRepoDir::find_top(current_dir, tenant_path);
        let top_dir = if is_top {
            if top_dir.is_some() {
                return Err(RepoError::TopAlreadyExists(current_dir.into()));
            } else {
                current_dir.to_path_buf()
            }
        } else {
            if let Some(top_dir) = top_dir {
                top_dir.to_path_buf()
            } else {
                return Err(RepoError::Topless(current_dir.into()));
            }
        };
        
        if !tenant_dir.is_dir() {
            fs::create_dir_all(&tenant_dir)
                .unwrap();
        }
        
        if is_top {
            // some directories are created for top-level regardless of configuration
            for reserved_dir_kind in ReservedDirKind::iter() {
                if !reserved_dir_kind.is_top() {
                    continue;
                }
                
                let reserved_dir = tenant_dir.join(reserved_dir_kind.relative_path());
                fs::create_dir_all(reserved_dir)
                    .unwrap();
            }
            
            // state directory is required for top primarily to store this file
            SemVerFile(SemVer::new(definition.semver.0, definition.semver.1, definition.semver.2))
                .write(&tenant_dir)
                .unwrap();
            
            // write an empty standard exclude file for the user in top if we use excludes at all
            if definition.designated_top.excludes {
                StandardExcludeFile(Cow::Borrowed(""))
                    .write(&tenant_dir)
                    .unwrap();
            }
        }
        
        if let Some(designated_set) = designated {
            
            // create all designator files
            DesignatorFiles::new(Cow::Borrowed(&designated_set))
                .write(&tenant_dir)?;
            
            for designated in designated_set {
                let def = definition.designated(&designated);
                
                // create the rest of the directories as configured
                let mut dirs = Vec::new();
                if def.local {
                    dirs.push(ReservedDirKind::Local.relative_path().to_path_buf());
                }
                if def.state {
                    if def.local {
                        dirs.push(ReservedDirKind::LocalState.relative_path().join(designated.filename()));
                    }
                    
                    dirs.push(ReservedDirKind::State.relative_path().join(designated.filename()));
                }
                
                for dir in dirs {
                    let dir = tenant_dir.join(dir);
                    fs::create_dir_all(dir)
                        .unwrap();
                }
                
                // init standard ignores
                if def.excludes {
                    if let Some(default_excludes) = definition.designated_top.default_excludes {
                        DesignatorExcludeFile::new(&designated, Cow::Borrowed(default_excludes))
                            .write(&tenant_dir)
                            .unwrap()
                    }
                }
            }
        }
        
        Ok(Self {
            dotrepo_dir,
            tenant_dir,
            top_dir,
            definition
        })
    }
    
    pub fn exists<P: AsRef<Path>>(dotrepo_dir: &DotRepoDir, dotrepo_subdir: P) -> bool {
        dotrepo_dir.is_dir() && dotrepo_dir.join(dotrepo_subdir).is_dir()
    }
    
    pub fn validate_exists<P: AsRef<Path>>(dotrepo_dir: &DotRepoDir, dotrepo_subdir: P) -> RepoResult<R, ()> {
        if !dotrepo_dir.is_dir() {
            panic!(".repo directory does not exist");
        } else if !dotrepo_dir.join(dotrepo_subdir).is_dir() {
            panic!(".repo subdirectory does not exist");
        }
        
        Ok(())
    }
    
    pub fn dotrepo_dir(&self) -> &Path {
        &self.dotrepo_dir
    }
    
    pub fn tenant_dir(&self) -> &Path {
        &self.tenant_dir
    }
    
    pub fn tenant_relative_path(&self) -> &Path {
        self.definition.tenant_path()
    }
    
    pub fn current_dir(&self) -> &Path {
        self.dotrepo_dir.parent().expect("parent exists")
    }
    
    pub fn top_dir(&self) -> &Path {
        &self.top_dir
    }
    
    pub fn excludes_dir(&self) -> PathBuf {
        self.tenant_dir.join(ReservedDirKind::Exclude.relative_path())
    }
    
    pub fn state_dir(&self) -> PathBuf {
        self.tenant_dir.join(ReservedDirKind::State.relative_path())
    }
    
    pub fn designator_dir(&self) -> PathBuf {
        self.tenant_dir.join(ReservedDirKind::Designator.relative_path())
    }
    
    pub fn read_version(&self) -> RepoResult<R, SemVer> {
        Ok(SemVerFile::read(&self.tenant_dir).unwrap().into())
    }
    
    pub fn read_excludes(&self) -> RepoResult<R, Gitignore> {
        let excludes = self.read_excludes_builder()
            .unwrap()
            .build()
            .unwrap();
        
        Ok(excludes)
    }
    
    fn read_excludes_builder(&self) -> RepoResult<R, GitignoreBuilder> {
        let exclude_files = RepositoryExcludeFiles::find(self.top_dir(), self.definition.tenant_path())
            .unwrap();
        
        let mut builder = GitignoreBuilder::new(self.current_dir());
        for exclude_file in exclude_files.files() {
            if let Some(_e) = builder.add(exclude_file) {
                panic!("error adding ignore");
            }
        }
        
        Ok(builder)
    }
    
    pub fn read_designations(&self) -> RepoResult<R, HashSet<Designated<R::Designator>>> {
        let designator_files = DesignatorFiles::read(self.tenant_dir())?;
        Ok(designator_files.take_designated())
    }
    
    pub fn walk_current(
        &self,
        walkoff: WalkOff
    ) -> impl Iterator<Item = walkdir::Result<walkdir::DirEntry>> {
        let excludes = if walkoff.using_excludes() {
            Some(self.read_excludes()
                .unwrap())
        } else {
            None
        };
        
        build_repository_walker(
            self.current_dir(),
            self.tenant_relative_path(),
            excludes,
            walkoff
        )
    }
    
    pub fn walk_top(
        &self,
        walkoff: WalkOff
    ) -> impl Iterator<Item = walkdir::Result<walkdir::DirEntry>> {
        let excludes = if walkoff.using_excludes() {
            Some(self.read_excludes()
                .unwrap())
        } else {
            None
        };
        
        build_repository_walker(
            self.top_dir(),
            self.tenant_relative_path(),
            excludes,
            walkoff
        )
    }
    
    pub fn find_designated(
        &self,
        walkoff: TenantWalkOff<R>,
    ) -> impl Iterator<Item = walkdir::Result<(walkdir::DirEntry, DesignatorMatches<R>)>> {
        build_tenant_walker(
            self.current_dir(),
            self.tenant_relative_path(),
            walkoff,
        )
    }
    
    /*pub fn find_designated_from_top(
        &self,
        designated: Option<HashSet<StandardDesignatorKind>>,
        designator: Option<StandardDesignator>,
    ) -> impl Iterator<Item = walkdir::Result<walkdir::DirEntry>> {
        build_tenant_walker(
            self.top_dir(),
            self.tenant_relative_path(),
            designated,
            designator
        )
    }*/
}


