use crate::*;

#[derive(Debug, snafu::Snafu)]
pub enum BridgeError {
    EnvVar { var: String, source: std::env::VarError },
    Incapable { capability: Capability },
    NotFound { noun: BridgeErr },
    SysCall { noun: BridgeErr, source: std::io::Error },
    String,
    Cmd { kind: CmdKind, output: std::process::Output },
    CmdCall { kind: CmdKind, source: std::io::Error },
    CmdChecked { kind: CmdKind, check: CmdCheck, output: std::process::Output },
    Copy { source: CopyError },
    Expected { noun: BridgeErr },
    /*
    Denied { kind: ErrNoun },
    Input { kind: ErrInput },
    Internal { message: String },
    Io { op: ErrIo, source: std::io::Error },
    Lock { noun: ErrNoun },
    String,
    Unavailable,
    Unsupported,
    UnsupportedDir { kind: Dir },*/
}

pub type BridgeResult<T> = Result<T, BridgeError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BridgeErr {
    User,
    UserGroup,
    Hostname,
    Copy,
    File,
    WindowsSID,
    UnixID,

    /*Editor,
    Which,
    Domain,
    DomainAuthority,
    UiCache,
    NetCache,
    AccessCache,
    PathsCache,*/
}

impl BridgeError {
    #[track_caller]
    pub const fn incapable(capability: Capability) -> Self {
        Self::Incapable { capability }
    }
    
    #[track_caller]
    pub const fn err_incapable<T>(capability: Capability) -> BridgeResult<T> {
        Err(Self::incapable(capability))
    }
    
    #[track_caller]
    pub const fn not_found(noun: BridgeErr) -> Self {
        Self::NotFound { noun }
    }
    
    #[track_caller]
    pub const fn err_not_found<T>(noun: BridgeErr) -> BridgeResult<T> {
        Err(Self::not_found(noun))
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
    pub fn err_env_var<T, S: Into<String>>(varname: S, source: env::VarError) -> BridgeResult<T> {
        Err(Self::env_var(varname, source))
    }
    #[track_caller]
    pub const fn err_cmd_call<T>(kind: CmdKind, source: std::io::Error) -> BridgeResult<T> {
        Err(Self::CmdCall { kind, source })
    }
    
    #[track_caller]
    pub const fn cmd_call(kind: CmdKind, source: std::io::Error) -> Self {
        Self::CmdCall { kind, source }
    }
    
    #[track_caller]
    pub const fn err_sys_call<T>(noun: BridgeErr, source: std::io::Error) -> BridgeResult<T> {
        Err(Self::SysCall { noun, source })
    }
    
    #[track_caller]
    pub const fn sys_call(noun: BridgeErr, source: std::io::Error) -> Self {
        Self::SysCall { noun, source }
    }
    
    #[track_caller]
    pub const fn err_cmd<T>(kind: CmdKind, output: std::process::Output) -> BridgeResult<T> {
        Err(Self::Cmd { kind, output })
    }
    
    #[track_caller]
    pub const fn cmd(kind: CmdKind, output: std::process::Output) -> Self {
        Self::Cmd { kind, output }
    }
    
    #[track_caller]
    pub const fn cmd_checked(kind: CmdKind, check: CmdCheck, output: std::process::Output) -> Self {
        Self::CmdChecked { kind, check, output }
    }

    /*
    pub const fn err_io<T>(op: ErrIo, source: std::io::Error) -> BridgeResult<T> {
        Err(Self::Io { op, source })
    }
    
    pub const fn io(op: ErrIo, source: std::io::Error) -> Self {
        Self::Io { op, source }
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
    pub const fn err_unsupported<T>() -> BridgeResult<T> {
        Err(Self::Unsupported)
    }
    
    #[track_caller]
    pub const fn unsupported_dir(kind: Dir) -> Self {
        Self::UnsupportedDir { kind }
    }

    #[track_caller]
    pub const fn err_string<T>() -> BridgeResult<T> {
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
    pub const fn err_input<T>(kind: ErrInput) -> BridgeResult<T> {
        Err(Self::Input { kind })
    }
    
    #[track_caller]
    pub fn internal<S: Into<String>>(message: S) -> Self {
        Self::Internal { message: message.into() }
    }

    #[track_caller]
    pub fn err_internal<T, S: Into<String>>(message: S) -> BridgeResult<T> {
        Err(Self::Internal { message: message.into() })
    }
    
    #[track_caller]
    pub const fn err_denied<T>(kind: ErrNoun) -> BridgeResult<T> {
        Err(Self::Denied { kind })
    }
    
    #[track_caller]
    pub const fn denied(kind: ErrNoun) -> Self {
        Self::Denied { kind }
    }
    
    
    #[track_caller]
    pub const fn lock(noun: ErrNoun) -> Self {
        Self::Lock { noun }
    }
    */
}

impl From<CopyError> for BridgeError {
    fn from(source: CopyError) -> Self {
        Self::Copy { source }
    }
}
