use crate::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum EnvSpec {
    Xdg,
    DotSys,
}

impl EnvSpec {
    pub const ENV_SPEC: &'static str =  "ENV_SPEC";
    pub const XDG: &'static str = "xdg";
    pub const DOTSYS: &'static str = "dotsys";
    
    pub const fn env_var_name() -> &'static str { Self::ENV_SPEC }
    pub const fn env_value(&self) -> &'static str {
        match self {
            Self::Xdg => Self::XDG,
            Self::DotSys => Self::DOTSYS,
        }
    }
}

pub enum SDir {
    Temporary,
    Execute,
    Library,
    Setting,
    Asset,
    Package,
    Variable,
}

/// User Directories: Internal software operation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum UDir {
    Cache,
    Config,
    Data,
    Temporary,
    Execute,
    Library,
    Setting,
    Asset,
    Package,
    Variable,
    SecretCache,
    SecretConfig,
    SecretData,
    SecretKey,
    ShellKey,
    Memory,
    Transitive,
}

impl UDir {
    /// `/home/ulric/.cache`, `/home/ulric/.sys/cache`
    pub const CACHE: &'static str = "UDIR_CACHE";
    /// `/home/ulric/.config`, `/home/ulric/.config`
    pub const CONFIG: &'static str = "UDIR_CONFIG";
    /// `/home/ulric/.local/share`, `/home/ulric/.sys/data`
    pub const DATA: &'static str = "UDIR_DATA";
    /// `/tmp`, `/home/ulric/tmp`
    pub const TEMPORARY: &'static str = "UDIR_TEMPORARY";
    /// `/home/ulric/.local/bin`, `/home/ulric/.sys/install/bin`
    pub const EXECUTE: &'static str = "UDIR_EXECUTE";
    /// `/home/ulric/.sys/install/lib`
    pub const LIBRARY: &'static str = "UDIR_LIBRARY";
    /// `/home/ulric/.sys/install/etc`
    pub const SETTING: &'static str = "UDIR_SETTING";
    /// `/home/ulric/.sys/install/share`
    pub const ASSET: &'static str = "UDIR_ASSET";
    /// `/home/ulric/.sys/install/opt`
    pub const PACKAGE: &'static str = "UDIR_PACKAGE";
    /// `/home/ulric/.sys/install/var`
    pub const VARIABLE: &'static str = "UDIR_VARIABLE";
    /// `/home/ulric/.sys/secret/cache`
    pub const SECRET_CACHE: &'static str = "UDIR_SECRET_CACHE";
    /// `/home/ulric/.sys/secret/config`
    pub const SECRET_CONFIG: &'static str = "UDIR_SECRET_CONFIG";
    /// `/home/ulric/.sys/secret/data`
    pub const SECRET_DATA: &'static str = "UDIR_SECRET_DATA";
    /// `/home/ulric/.sys/secret/key`
    pub const SECRET_KEY: &'static str = "UDIR_SECRET_KEY";
    /// `/home/ulric/.ssh`
    pub const SHELL_KEY: &'static str = "UDIR_SHELL_KEY";
    /// `/home/ulric/.sys/mess`
    pub const MESS: &'static str = "UDIR_MESS";
    /// `/dev/shm/ulric`
    pub const MEMORY: &'static str = "UDIR_MEMORY";
    /// `/run/user/1000`
    pub const TRANSITIVE: &'static str = "UDIR_TRANSITIVE";
}
    
impl UDir {
    pub const fn env_var_name(&self) -> &'static str {
        match self {
            Self::Temporary => Self::UDIR_TEMPORARY,
            Self::Execute => Self::UDIR_EXECUTE,
            Self::Library => Self::UDIR_LIBRARY,
            Self::Asset => Self::UDIR_ASSET,
            Self::Package => Self::UDIR_PACKAGE,
            Self::SecretCache => Self::UDIR_SECRET_CACHE,
            Self::SecretConfig => Self::UDIR_SECRET_CONFIG,
            Self::SecretData => Self::UDIR_SECRET_DATA,
            Self::Memory => Self::UDIR_MEMORY,
        }
    }
    
    /// Variant path based on the provided a home dir.
    /// Only uses shell-expansion if necessary.
    pub fn homed(&self, home: &Path) -> CrossResult<PathBuf> {
        let path = match self {
            Self::TemporaryHome => env::var_os(Self::ENV_TMP_HOME).map(PathBuf::from).unwrap_or_else(|| home.join(Self::DEFAULT_TMP_HOME)),
            Self::ExecuteHome => env::var_os(Self::ENV_EXECUTE_HOME).map(PathBuf::from).unwrap_or_else(|| home.join(Self::DEFAULT_EXECUTE_HOME)),
        };
        
        // avoid shell-expansion when possible        
        let lossy = path.to_string_lossy();  
        if lossy.contains(['$', '~']) {
            shellexpand::full(lossy.as_ref())
                .map(|p| PathBuf::from(p.as_ref()))
                .map_err(|e| CrossError::env_var(e.var_name, e.cause))
        } else {
            Ok(path)
        }
    }
    
    /// Variant path based on the provided a home dir and the suffix subdir.
    pub fn join_homed<P2>(&self, home: &Path, subdir: P2) -> CrossResult<PathBuf>
    where
        P2: AsRef<Path> + Into<PathBuf>,
    {
        Ok(self.homed(home)?.join(subdir))
    }
}

pub enum MDir {
}

pub enum UEnv {
    MDir(MDir),
    UDir(UDir),
}

impl UEnv {
    pub const fn env_var_name(&self) -> &'static str {
        match self {
            Self::UDir(udir) => udir.env_var_name(),
            _ => todo!(),
        }
    }
}

#[inline]
pub fn uenv_spec() -> Cow<'static, str> {
    ::std::env::var(EnvSpec::ENV_SPEC)
        .map(|v| Cow::Owned(v))
        .unwrap_or_else(|_| Cow::Borrowed("xdg"))
}

#[inline]
pub fn uenv_var_enum(kind: UEnv) -> Result<Cow<'static, str>, ::std::env::VarError> {
    match ::std::env::var(kind.env_var_name()) {
        Ok(v) => Ok(Cow::Owned(v)),
        Err(::std::env::VarError::NotPresent) => match kind {
            UEnv::UDir(UDir::Asset) => Ok(Cow::Borrowed("foo")),
            _ => todo!()
        },
        Err(e @ ::std::env::VarError::NotUnicode(_)) => Err(e),
    }
}

#[inline]
pub fn uenv_var(key: &str) -> Result<Cow<'static, str>, ::std::env::VarError> {
    match ::std::env::var(key) {
        Ok(v) => Ok(Cow::Owned(v)),
        Err(::std::env::VarError::NotPresent) => match key {
            UDir::ASSET => Ok(Cow::Borrowed("foo")),
            _ => todo!()
        },
        Err(e @ ::std::env::VarError::NotUnicode(_)) => Err(e),
    }
}

pub fn uenv_expand(s: &str) -> Result<Cow<'_, str>, String> {
    #[inline]
    pub fn lookup_home_dir() -> Option<String> {
        ::std::env::home_dir()
            .map(|v| v.into_string().expect("UTF8"))
    }
    
    #[inline]
    pub fn lookup_uenv_var(key: &str) -> Result<Option<Cow<'static, str>>, String> {
        match uenv_var(key) {
            Ok(v) => Ok(Some(v)),
            Err(::std::env::VarError::NotPresent) => Ok(None),
            Err(::std::env::VarError::NotUnicode(name)) => Err(name.into_string().expect("UTF8")),
        }
    }
    
    match shellexpand::full_with_context(s, lookup_home_dir, lookup_uenv_var) {
        Ok(v) => Ok(v),
        Err(e) => Err(e.var_name),
    }
}