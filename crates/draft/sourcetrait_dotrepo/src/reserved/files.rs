use std::{borrow::Cow, collections::HashSet, fs, io, marker::PhantomData, path::{Path, PathBuf}};
use crate::*;
use semver::Version;
use strum::IntoEnumIterator;
use walkdir::WalkDir;

pub struct SemVerFile(pub Version);
impl SemVerFile {
    pub fn version(&self) -> &Version {
        &self.0
    }
    
    pub fn write(&self, repo_dir: &Path) -> io::Result<()> {
        let filepath = repo_dir.join(ReservedFileKind::SemVer.as_ref());
        fs::write(filepath, &self.0.to_string())
    }
    
    pub fn read(repo_dir: &Path) -> io::Result<Self> {
        let filepath = repo_dir.join(ReservedFileKind::SemVer.as_ref());
        let contents = fs::read_to_string(&filepath)
            .map_err(|e| io::Error::new(e.kind(), format!("Unable to read from version file: {filepath:?} :: {e}")))?;
        let semver = Version::parse(&contents)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData,
                format!("Unable to parse semantic version file: {filepath:?} :: {e}")))?;
        
        Ok(Self(semver))
    }
}

impl From<SemVerFile> for Version {
    fn from(value: SemVerFile) -> Self {
        value.0
    }
}

pub struct StandardExcludeFile<'a>(pub Cow<'a, str>);
impl<'a> StandardExcludeFile<'a> {
    pub fn contents(&self) -> &str {
        &self.0
    }
    
    pub fn write(&self, tenant_dir: &Path) -> io::Result<()> {
        let filepath = tenant_dir.join(ReservedFileKind::StandardExclude.as_ref());
        fs::write(&filepath, &*self.0)
            .map_err(|e| io::Error::new(e.kind(),
                format!("Unable to write to standard exclude file: {filepath:?} :: {e}")))?; 
    
        Ok(())
    }
    
    pub fn read(tenant_dir: &Path) -> io::Result<Self> {
        let filepath = tenant_dir.join(ReservedFileKind::StandardExclude.as_ref());
        let contents = fs::read_to_string(&filepath)
            .map_err(|e| io::Error::new(e.kind(),
                format!("Unable to read from standard exclude file: {filepath:?} :: {e}")))?;
        
        Ok(Self(Cow::Owned(contents)))
    }
}

pub struct DesignatorExcludeFile<'a,DK,D>
where
    DK: DesignatorKind,
    D: DesignatorTraits<DK>
{
    designated: &'a Designated<D>,
    contents: Cow<'a, str>,
    marker: PhantomData<DK>
}

impl<'a,DK,D> DesignatorExcludeFile<'a,DK,D>
where
    DK: DesignatorKind,
    D: DesignatorTraits<DK>
{
    pub fn new(designated: &'a Designated<D>, contents: Cow<'a, str>) -> Self {
        Self {
            designated,
            contents,
            marker: PhantomData
        }
    }
    
    pub fn designated(&self) -> &Designated<D> {
        &self.designated
    }
    
    pub fn contents(&self) -> &str {
        &self.contents
    }
    
    pub fn write(&self, repo_dir: &Path) -> io::Result<()> {
        let filepath = repo_dir.join(ReservedDirKind::Exclude.relative_path())
            .join(format!("{}.{}", self.designated().filename(), GLOBS));
        fs::write(&filepath, self.contents())
            .map_err(|e| io::Error::new(e.kind(),
                format!("Unable to write to designator exclude file: {filepath:?} :: {e}")))?;
    
        Ok(())
    }
    
    pub fn read(designated: &'a Designated<D>, repo_dir: &Path) -> io::Result<Self> {
        let filepath = repo_dir.join(ReservedDirKind::Exclude.relative_path())
            .join(format!("{}.{}", designated.filename(), GLOBS));
        let contents = fs::read_to_string(&filepath)
            .map_err(|e| io::Error::new(e.kind(),
                format!("Unable to read from designator exclude file: {filepath:?} :: {e}")))?;
        
        Ok(Self::new(designated, Cow::Owned(contents)))
    }
}

pub(crate) trait MapResultIO<T> {
    fn map_err_io<F, O: FnOnce(io::Error, Option<PathBuf>) -> F>(self, op: O) -> std::result::Result<T, F>;
}

impl MapResultIO<walkdir::DirEntry> for walkdir::Result<walkdir::DirEntry> {
    fn map_err_io<F, O: FnOnce(io::Error, Option<PathBuf>) -> F>(self, op: O) -> std::result::Result<walkdir::DirEntry, F> {
        match self {
            Ok(t) => Ok(t),
            Err(e) => {
                let path = e.path().map(|p| p.to_path_buf());
                let path_str = path.as_ref()
                    .map(|p| format!(": {}", p.to_string_lossy().to_string()))
                    .unwrap_or_default();
                
                if e.io_error().is_some() {
                    let e = e.into_io_error().expect("io error");
                    Err(op(io::Error::new(e.kind(), format!("Unable to access path{path_str} :: {e}")), path))
                } else {
                    Err(op(io::Error::new(io::ErrorKind::TooManyLinks, format!("Unable to access path{path_str} :: {e}")), path))
                }
            }
        }
    }
}

pub struct RepositoryExcludeFiles(Vec<PathBuf>);
impl RepositoryExcludeFiles {
    pub fn files(&self) -> &Vec<PathBuf> {
        &self.0
    }
    
    pub fn find(repository_dir: &Path, tenant_subdir: &Path) -> io::Result<Self> {
        let tenant_relpath = PathBuf::from(DOTREPO).join(tenant_subdir);
        let tenant_walk = WalkDir::new(repository_dir)
            .into_iter()
            .filter_entry(|entry| entry.file_type().is_dir())
            .filter(|entry| entry.as_ref().is_ok_and(|entry| {
                entry.path().ends_with(&tenant_relpath) 
            }));
        
        let standard_exclude_relpath = ReservedFileKind::StandardExclude.as_ref();
        
        let mut files = Vec::new();
        for tenant_entry in tenant_walk {
            let tenant_entry = tenant_entry
                .map_err_io(|e, _| e)?;
            
            let standard_filepath = tenant_entry.path().join(standard_exclude_relpath);
            if standard_filepath.exists() {
                files.push(standard_filepath);
            }
            
            let excludes_dir = tenant_entry.path().join(ReservedDirKind::Exclude.as_ref());
            let excludes_walk = WalkDir::new(excludes_dir)
                .into_iter()
                .filter(|entry| entry.as_ref().is_ok_and(|entry| {
                    entry.file_type().is_file()
                    && entry.path().extension().is_some_and(|ext| ext == GLOBS)
                }));
            
            for excludes_entry in excludes_walk {
                let excludes_entry = excludes_entry
                    .map_err_io(|e, _| e)?;
                
                files.push(excludes_entry.into_path())
            }
        }
        
        Ok(Self(files))
    }
}

pub struct DesignatorFiles<'a, R: 'static + DotRepoType> {
    designated: Cow<'a, HashSet<Designated<R::Designator>>>,
    marker: PhantomData<R::DesignatorKind>
}

impl<'a, R: 'static + DotRepoType> DesignatorFiles<'a,R> {
    pub fn new(designated: Cow<'a, HashSet<Designated<R::Designator>>>) -> Self {
        Self {
            designated,
            marker: PhantomData
        }
    }
    
    pub fn designated(&self) -> &Cow<'a, HashSet<Designated<R::Designator>>> {
        &self.designated
    }
    
    pub fn take_designated(self) -> HashSet<Designated<R::Designator>> {
        self.designated.into_owned()
    }
    
    pub fn read(tenant_dir: &Path) -> RepoResult<R, Self> {
        let designator_dir = tenant_dir.join(ReservedDirKind::Designator.relative_path());
        let mut designated_set = HashSet::new();
        let read_dir_it = fs::read_dir(&designator_dir)
            .map_err(|e| RepoError::Io(format!("Unable to search designator directory: {designator_dir:?}"), e))?;
        
        'outer: for entry in read_dir_it {
            let entry = entry
                .map_err(|e| RepoError::Io(format!("Unable to access entry within designator directory: {designator_dir:?}"), e))?;
            
            if entry.file_type().is_ok_and(|filetype| filetype.is_file()) {
                let file_name = entry.file_name();
                let name = file_name.to_string_lossy();
                let identifier: Option<String> = fs::read_to_string(entry.path())
                    .ok()
                    .and_then(|ident| Some(ident.trim().into()));
                
                for kind in StandardDesignatorKind::iter() {
                    if kind.name() == name {
                        let designator = StandardDesignator::try_from_tuple(DesignatorTuple(kind, identifier))?;
                        designated_set.insert(Designated::Standard(designator));
                        continue 'outer;
                    }
                }
                
                for kind in R::DesignatorKind::iter() {
                    if kind.name() == name {
                        let designator = R::Designator::try_from_tuple(DesignatorTuple(kind, identifier))?;
                        designated_set.insert(Designated::Tenant(designator));
                        continue 'outer;
                    }
                }
            }
        }
        
        Ok(Self::new(Cow::Owned(designated_set)))
    }
    
    pub fn write(&self, tenant_dir: &Path) -> RepoResult<R, ()> {
        let designator_dir = tenant_dir.join(ReservedDirKind::Designator.relative_path());
        
        if !designator_dir.exists() {
            fs::create_dir_all(&designator_dir)
                .unwrap();
        }
        
        for designated in &*self.designated {
            let filepath = designator_dir.join(designated.filename());
            fs::write(filepath, designated.identifier().unwrap_or_default())
                .unwrap();
        }
        
        Ok(())
    }
}