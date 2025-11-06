use crate::*;

pub trait PathsComponentTrait {
    fn dir(&self, cross_dir: Dir) -> CrossResult<PathBuf>;
    
    fn home_dir(&self) -> CrossResult<PathBuf>;
    
    /// System paths as specified by the environment, in order.
    fn env_paths(&self) -> CrossResult<Arc<Vec<PathBuf>>>;
    
    fn subdir<P>(&self, cross_dir: Dir, subdir: P) -> CrossResult<PathBuf>
    where
        P: AsRef<Path> + Into<PathBuf>;
    
    /// Forces the use of XDG specifications regardless of OS.
    fn xdg_dir<P>(&self, xdg_dir: XdgDir) -> CrossResult<PathBuf>;
    
    /// Forces the use of XDG specifications regardless of OS.
    fn xdg_subdir<P>(&self, xdg_dir: XdgDir, subdir: P) -> CrossResult<PathBuf>
    where
        P: AsRef<Path> + Into<PathBuf>;
    
    fn base_paths(&self) -> CrossResult<BasePaths>;
}

#[allow(private_bounds)]
pub struct StandardPathsComponent<LOOKUP: PathsComponentLookup>(pub(crate) LOOKUP);

#[allow(private_bounds)]
impl<LOOKUP: PathsComponentLookup> StandardPathsComponent<LOOKUP> {
    fn lookup(&self) -> &LOOKUP { &self.0 }
    
    fn determined_base_dirs<'mutex, 'lock>(&self, lock: &'mutex mut StaticCacheLock<'lock, PathsCache>) -> CrossResult<&'mutex Arc<directories::BaseDirs>> {
        cache_locked_value_mut(lock)?
            .base_dirs
            .determine(|| match directories::BaseDirs::new() {
                Some(dirs) => Ok(Arc::new(dirs)),
                None => Err(CrossError::env_var_none(ENV_HOME)),
            })
    }
}

impl<LOOKUP: PathsComponentLookup> PathsComponentTrait for StandardPathsComponent<LOOKUP> {
    fn dir(&self, cross_dir: Dir) -> CrossResult<PathBuf> {
        let dir = {
            let mut paths_cache_lock = paths_cache_lock()?;
            let base_dirs = self.determined_base_dirs(&mut paths_cache_lock)?; 
            let dir = match cross_dir {
                Dir::HomeCache => base_dirs.cache_dir(),
                Dir::HomeConfig => base_dirs.config_dir(),
                Dir::HomeData => base_dirs.data_dir(),
                Dir::HomeState => {
                    base_dirs.state_dir()
                        .ok_or_else(|| CrossError::unsupported_dir(cross_dir))?
                },
            };
            
            dir.to_path_buf()
        };
        
        Ok(dir)
    }
    
    fn home_dir(&self) -> CrossResult<PathBuf> {
        let path = {
            let mut paths_cache_lock = paths_cache_lock()?;
            let base_dirs = self.determined_base_dirs(&mut paths_cache_lock)?; 
            base_dirs.home_dir().to_path_buf()
        };
        
        Ok(path)
    }
    
    fn env_paths(&self) -> CrossResult<Arc<Vec<PathBuf>>> {
        let result = {
            let mut paths_cache_lock = paths_cache_lock()?;
            cache_locked_value_mut(&mut paths_cache_lock)?
                .env_paths
                .determine(|| {
                    self.lookup().lookup_env_paths()
                        .map(Arc::new)
                        .map_err(|e| e.into())
                })
                .map(Arc::clone)
        };
        
        result
    }
    
    fn subdir<P>(&self, cross_dir: Dir, subdir: P) -> CrossResult<PathBuf>
    where
        P: AsRef<Path> + Into<PathBuf>
    {
        Ok(self.dir(cross_dir)?.join(subdir))
    }
    
    fn xdg_dir<P>(&self, xdg_dir: XdgDir) -> CrossResult<PathBuf> {
        let result = {
            let mut paths_cache_lock = paths_cache_lock()?;
            let base_dirs = self.determined_base_dirs(&mut paths_cache_lock)?; 
            xdg_dir.homed(base_dirs.home_dir())
        };

        result
    }
    
    fn xdg_subdir<P>(&self, xdg_dir: XdgDir, subdir: P) -> CrossResult<PathBuf>
    where
        P: AsRef<Path> + Into<PathBuf>
    {
        let result = {
            let mut paths_cache_lock = paths_cache_lock()?;
            let base_dirs = self.determined_base_dirs(&mut paths_cache_lock)?; 
            xdg_dir.join_homed(base_dirs.home_dir(), subdir)
        };

        result
    }
    
    fn base_paths(&self) -> CrossResult<BasePaths> {
        let base_dirs = {
            let mut paths_cache_lock = paths_cache_lock()?;
            let base_dirs = self.determined_base_dirs(&mut paths_cache_lock)?; 
            Arc::clone(base_dirs)
        };
        
        Ok(BasePaths(Either::A(base_dirs)))
    }
}

#[derive(Debug)]
struct PathsCache {
    base_dirs: CacheDetermined<Arc<directories::BaseDirs>>,
    env_paths: CacheDetermined<Arc<Vec<PathBuf>>>,
}

impl PathsCache {
    const fn default_const() -> Self {
        Self {
           base_dirs: None,
           env_paths: None, 
        }
    }
}
    
fn paths_cache() -> &'static StaticCache<PathsCache> {
    static CACHE: LazyLock<StaticCache<PathsCache>> = LazyLock::new(|| { 
        new_static_cache_value(PathsCache::default_const())
    });
    
    &CACHE
}

fn paths_cache_lock<'lock>() -> CrossResult<StaticCacheLock<'lock, PathsCache>> {
    paths_cache().lock().map_err(|_| CrossError::lock(CrossErr::PathsCache))
}
