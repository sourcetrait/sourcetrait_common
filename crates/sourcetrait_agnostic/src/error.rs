use crate::*;

#[derive(Debug, snafu::Snafu)]
pub enum CrossError {
    Cached { why: CachedErrWhy, miss: CacheMiss },
    Cmd { kind: CmdKind, output: std::process::Output },
    CmdCall { kind: CmdKind, source: std::io::Error },
    CmdChecked { kind: CmdKind, check: CmdCheck, output: std::process::Output },
    #[snafu(display("{source}"))]
    Copy { source: CopyError },
    #[snafu(display("Permission denied for {kind}"))]
    Denied { kind: CrossErr },
    #[snafu(display("Environment variable: {var}"))]
    EnvVar { var: String, source: std::env::VarError },
    #[snafu(display("Expected: {noun}"))]
    Expected { noun: CrossErr },
    Incapable { capability: Capability },
    Input { kind: ErrInput },
    Internal { message: String },
    #[snafu(display("System call error for {noun}: {source}"))]
    SysCall { source: std::io::Error, noun: CrossErr },
    #[snafu(display("IO error {{\n  op: {op}\n  err: {source}\n}}"))]
    Io { op: ErrIo, source: std::io::Error },
    Lock { noun: CrossErr },
    #[snafu(display("Not found: {noun}"))]
    NotFound { noun: CrossErr },
    String,
    Unavailable,
    Unsupported,
    UnsupportedDir { kind: Dir },
}

pub type CrossResult<T> = Result<T, CrossError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::Display)]
pub enum CrossErr {
    AccessCache,
    Copy,
    Domain,
    DomainAuthority,
    Editor,
    File,
    Hostname,
    NetCache,
    PathsCache,
    UiCache,
    UnixID,
    User,
    UserGroup,
    Which,
    WindowsSID,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CachedError {
    why: CachedErrWhy, 
    miss: CacheMiss
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CachedErrWhy {
    Cached,
    Cmd,
    CmdCall,
    CmdChecked,
    Copy,
    Denied,
    EnvVar,
    Expected,
    Incapable,
    Input,
    Internal,
    Io,
    Lock,
    NotFound,
    String,
    SysCall,
    Unavailable,
    Unsupported,
    UnsupportedDir,
}

impl From<&CrossError> for CachedErrWhy {
    fn from(err: &CrossError) -> Self {
        match err {
            CrossError::Cached {..} => Self::Cached,
            CrossError::Cmd {..} => Self::Cmd,
            CrossError::CmdCall {..} => Self::CmdCall,
            CrossError::CmdChecked {..} => Self::CmdChecked,
            CrossError::Copy {..} => Self::Copy,
            CrossError::Denied {..} => Self::Denied,
            CrossError::EnvVar {..} => Self::EnvVar,
            CrossError::Expected {.. } => Self::Expected,
            CrossError::Incapable {..} => Self::Incapable,
            CrossError::Input {..} => Self::Input,
            CrossError::Internal {..} => Self::Internal,
            CrossError::Io {..} => Self::Io,
            CrossError::Lock {..} => Self::Lock,
            CrossError::NotFound {..} => Self::NotFound,
            CrossError::String {..} => Self::String,
            CrossError::SysCall {..} => Self::SysCall,
            CrossError::Unavailable {..} => Self::Unavailable,
            CrossError::Unsupported {..} => Self::Unsupported,
            CrossError::UnsupportedDir {..} => Self::UnsupportedDir,
        } 
    }
}
impl CachedError {
    pub(crate) fn new(why: CachedErrWhy) -> Self {
        Self { why, miss: CacheMiss::new() }
    }
}

pub(crate) type CachedResult<T> = Result<CacheValue<T>, CachedError>;

#[derive(Debug, Clone, PartialEq, Eq, derive_more::Display)]
pub enum ErrIo {
    #[display("CreateFile: {}", _0.to_string_lossy())]
    CreateFile(PathBuf),
    #[display("CreateDir: {}", _0.to_string_lossy())]
    CreateDir(PathBuf),
    #[display("ReadFile: {}", _0.to_string_lossy())]
    ReadFile(PathBuf),
    #[display("WriteFile: {}", _0.to_string_lossy())]
    WriteFile(PathBuf),
    #[display("ReadDir: {}", _0.to_string_lossy())]
    ReadDir(PathBuf),
    #[display("WriteDir: {}", _0.to_string_lossy())]
    WriteDir(PathBuf),
    #[display("DeleteFile: {}", _0.to_string_lossy())]
    DeleteFile(PathBuf),
    #[display("DeleteDir: {}", _0.to_string_lossy())]
    DeleteDir(PathBuf),
    #[display("CopyFile: {{\n    from: {src}\n    to: {dst}\n  }}",
        src = _0.to_string_lossy(), dst = _1.to_string_lossy()
    )]
    CopyFile(PathBuf, PathBuf),
}

impl ErrIo {
    pub fn copy_file<P1, P2>(source: P1, dest: P2) -> Self
    where
        P1: Into<PathBuf>,
        P2: Into<PathBuf>,
    {
        Self::CopyFile(source.into(), dest.into())
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrInput {
    Username,
}

impl<T> From<CrossError> for CrossResult<T> {
    fn from(err: CrossError) -> Self {
        Err(err)
    }
}

impl From<CachedError> for CrossError {
    fn from(err: CachedError) -> Self {
        CrossError::Cached { why: err.why, miss: err.miss }
    }
}

impl From<&CachedError> for CrossError {
    fn from(err: &CachedError) -> Self {
        CrossError::Cached { why: err.why, miss: err.miss }
    }
}

impl From<&mut CachedError> for CrossError {
    fn from(err: &mut CachedError) -> Self {
        CrossError::Cached { why: err.why, miss: err.miss }
    }
}

impl From<&CrossError> for CachedError {
    fn from(err: &CrossError) -> Self {
        Self::new(err.into())
    }
}

impl<T> From<CachedError> for CrossResult<T> {
    fn from(err: CachedError) -> Self {
        Err(CrossError::Cached { why: err.why, miss: err.miss })
    }
}

impl CrossError {
    #[track_caller]
    pub const fn err_io<T>(op: ErrIo, source: std::io::Error) -> CrossResult<T> {
        Err(CrossError::Io { op, source })
    }
    
    pub const fn io(op: ErrIo, source: std::io::Error) -> Self {
        Self::Io { op, source }
    }
    
    #[track_caller]
    pub const fn err_not_found<T>(noun: CrossErr) -> CrossResult<T> {
        Err(CrossError::NotFound { noun })
    }
    
    #[track_caller]
    pub const fn not_found(noun: CrossErr) -> Self {
        Self::NotFound { noun }
    }
    
    #[track_caller]
    pub const fn err_cmd_call<T>(kind: CmdKind, source: std::io::Error) -> CrossResult<T> {
        Err(CrossError::CmdCall { kind, source })
    }
    
    #[track_caller]
    pub const fn cmd_call(kind: CmdKind, source: std::io::Error) -> Self {
        Self::CmdCall { kind, source }
    }
    
    #[track_caller]
    pub const fn err_cmd<T>(kind: CmdKind, output: std::process::Output) -> CrossResult<T> {
        Err(CrossError::Cmd { kind, output })
    }
    
    #[track_caller]
    pub const fn cmd(kind: CmdKind, output: std::process::Output) -> Self {
        Self::Cmd { kind, output }
    }
    
    #[track_caller]
    pub const fn cmd_checked(kind: CmdKind, check: CmdCheck, output: std::process::Output) -> Self {
        Self::CmdChecked { kind, check, output }
    }
    
    #[track_caller]
    pub fn env_var<S: Into<String>>(varname: S, source: std::env::VarError) -> Self {
        Self::EnvVar { var: format!("${}", varname.into()), source }
    }
    
    #[track_caller]
    pub fn env_var_none<S: Into<String>>(varname: S) -> Self {
        Self::EnvVar { var: format!("${}", varname.into()), source: env::VarError::NotPresent }
    }
    
    #[track_caller]
    pub fn err_env_var<T, S: Into<String>>(varname: S, source: std::env::VarError) -> CrossResult<T> {
        Err(Self::EnvVar { var: varname.into(), source })
    }
    
    #[track_caller]
    pub const fn err_unsupported<T>() -> CrossResult<T> {
        Err(Self::Unsupported)
    }
    
    #[track_caller]
    pub const fn unsupported_dir(kind: Dir) -> Self {
        Self::UnsupportedDir { kind }
    }

    #[track_caller]
    pub const fn err_string<T>() -> CrossResult<T> {
        Err(Self::String)
    }

    #[track_caller]
    pub const fn string() -> Self {
        Self::String
    }

    #[track_caller]
    pub const fn input(kind: ErrInput) -> Self {
        Self::Input { kind }
    }

    #[track_caller]
    pub const fn err_input<T>(kind: ErrInput) -> CrossResult<T> {
        Err(Self::Input { kind })
    }
    
    #[track_caller]
    pub fn internal<S: Into<String>>(message: S) -> Self {
        Self::Internal { message: message.into() }
    }

    #[track_caller]
    pub fn err_internal<T, S: Into<String>>(message: S) -> CrossResult<T> {
        Err(Self::Internal { message: message.into() })
    }
    
    #[track_caller]
    pub const fn err_denied<T>(kind: CrossErr) -> CrossResult<T> {
        Err(CrossError::Denied { kind })
    }
    
    #[track_caller]
    pub const fn denied(kind: CrossErr) -> Self {
        Self::Denied { kind }
    }
    
    #[track_caller]
    pub const fn incapable(capability: Capability) -> Self {
        Self::Incapable { capability }
    }
    
    #[track_caller]
    pub const fn err_incapable<T>(capability: Capability) -> CrossResult<T> {
        Err(Self::Incapable { capability })
    }
    
    #[track_caller]
    pub const fn lock(noun: CrossErr) -> Self {
        Self::Lock { noun }
    }
}

impl From<BridgeError> for CrossError {
    fn from(e: BridgeError) -> Self {
        match e {
            BridgeError::EnvVar { var, source } => CrossError::EnvVar { var, source },
            BridgeError::Incapable { capability } => CrossError::Incapable { capability },
            BridgeError::NotFound { noun } => CrossError::NotFound { noun: noun.into() },
            BridgeError::SysCall { source, noun } => CrossError::SysCall { source, noun: noun.into() },
            BridgeError::String => CrossError::String,
            BridgeError::Cmd { kind, output } => CrossError::Cmd { kind, output },
            BridgeError::CmdCall { kind, source } => CrossError::CmdCall { kind, source },
            BridgeError::CmdChecked { kind, check, output } => CrossError::CmdChecked { kind, check, output },
            BridgeError::Copy { source } => CrossError::Copy { source },
            BridgeError::Expected { noun } => CrossError::Expected { noun: noun.into() },
        }
    }
}

impl From<BridgeErr> for CrossErr {
    fn from(e: BridgeErr) -> Self {
        match e {
            BridgeErr::User => Self::User,
            BridgeErr::UserGroup => Self::UserGroup,
            BridgeErr::Hostname => Self::Hostname,
            BridgeErr::Copy => Self::Copy,
            BridgeErr::File => Self::File,
            BridgeErr::WindowsSID => Self::WindowsSID,
            BridgeErr::UnixID => Self::UnixID,
        }
    }
}

impl CrossError {
    pub fn source_io_kind(&self) -> Option<io::ErrorKind> {
        match self {
            CrossError::CmdCall { source, .. } => Some(source.kind()),
            CrossError::Copy { source: CopyError::Clean { source, .. } | CopyError::Dirty { source, .. } } => Some(source.kind()),
            CrossError::SysCall { source, .. } => Some(source.kind()),
            CrossError::Io { source, .. } => Some(source.kind()),
            _ => None,
        }
    }
    
    pub fn source_io_permission_denied(&self) -> bool {
        match self.source_io_kind() {
            Some(io::ErrorKind::PermissionDenied) => true,
            _ => false,
        }
    }
}