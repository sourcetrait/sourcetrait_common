use std::borrow::Cow;
use crate::*;

#[non_exhaustive]
#[derive(Debug, Clone, Default)]
pub struct AddOptions {}

impl AddOptions {
    pub const DEFAULT: Self  = Self {};
}

#[non_exhaustive]
#[derive(Debug, Clone, Default)]
pub struct BranchCreateOptions<'a> {
    pub start_point: Option<Cow<'a, str>>,
    pub orphan: bool,
}

impl<'a> BranchCreateOptions<'a> {
    pub const DEFAULT: Self  = Self {
        start_point: None,
        orphan: false,
    };
    
    pub fn validate(self) -> Result<Self> {
        Ok(self)
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, Default)]
pub struct BranchDeleteOptions {}

impl BranchDeleteOptions {
    pub const DEFAULT: Self  = Self {};
}

#[non_exhaustive]
#[derive(Debug, Clone, Default)]
pub struct BranchMoveOptions {}

impl BranchMoveOptions {
    pub const DEFAULT: Self  = Self {};
}

#[non_exhaustive]
#[derive(Debug, Clone, Default)]
pub struct CloneOptions {
    pub env: Option<GitEnv>,
}

impl CloneOptions {
    pub const DEFAULT: Self  = Self { env: None };
}

#[non_exhaustive]
#[derive(Debug, Clone, Default)]
pub struct CommitOptions {}

impl CommitOptions {
    pub const DEFAULT: Self  = Self {};
}

#[non_exhaustive]
#[derive(Debug, Clone, Default)]
pub struct DiffStatusOptions {}

impl DiffStatusOptions {
    pub const DEFAULT: Self  = Self {};
}

#[non_exhaustive]
#[derive(Debug, Clone, Default)]
pub struct FetchOptions {
    pub all: bool,
    pub prune: bool,
}

impl FetchOptions {
    pub const DEFAULT: Self  = Self { all: false, prune: false };
}

#[non_exhaustive]
#[derive(Debug, Clone, Default)]
pub struct InitOptions {
    pub env: Option<GitEnv>,
}

impl InitOptions {
    pub const DEFAULT: Self  = Self { env: None };
}

#[non_exhaustive]
#[derive(Debug, Clone, Default)]
pub struct InitBareOptions {
    pub env: Option<GitEnv>,
}

impl InitBareOptions {
    pub const DEFAULT: Self  = Self { env: None };
}

#[non_exhaustive]
#[derive(Debug, Clone, Default)]
pub struct OpenOptions {
    pub env: Option<GitEnv>,
}

impl OpenOptions {
    pub const DEFAULT: Self  = Self { env: None };
}

#[non_exhaustive]
#[derive(Debug, Clone, Default)]
pub struct LogOptions {
    pub show_message: bool,
    pub show_signature_fingerprint: bool,
}

impl LogOptions {
    pub const DEFAULT: Self  = Self { show_message: false, show_signature_fingerprint: false };
    
    pub fn validate(self) -> Result<Self> {
        Ok(self)
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, Default)]
pub struct MergeOptions {
    pub auto_resolve_only: bool,
    pub fast_forward_only: bool,
}

impl MergeOptions {
    pub const DEFAULT: Self  = Self {
        auto_resolve_only: false,
        fast_forward_only: false,
    };
    
    pub fn validate(self) -> Result<Self> {
        if self.auto_resolve_only && self.fast_forward_only {
            return Err(Error::OptionsValidate("MergeOptions", "auto_resolve_only and fast_forward_only cannot both be set"));
        }
        
        Ok(self)
    }
}


#[non_exhaustive]
#[derive(Debug, Clone, Default)]
pub struct PullOptions<'a> {
    /// None: Defaults to the configured remote upstream or ORIGIN otherwise
    pub upstream: Option<Upstream<'a>>,
    pub fast_forward_only: bool,
    pub rebase: bool,
}

impl<'a> PullOptions<'a> {
    pub const DEFAULT: Self  = Self {
        upstream: None,
        fast_forward_only: false,
        rebase: false,
    };
    
    pub fn validate(self) -> Result<Self> {
        if self.fast_forward_only && self.rebase {
            return Err(Error::OptionsValidate("PullOptions", "fast_forward_only and rebase cannot both be set"));
        }

        Ok(self)
    }
}

#[derive(Debug, Clone, Default)]
pub struct PushOptions<'a> {
    /// None: Defaults to the configured remote upstream or ORIGIN otherwise
    pub upstream: Option<Upstream<'a>>,

    /// None: Defaults to the configured remote upstream  
    pub set_upstream: Option<Upstream<'a>>,
    
    // sets upstream remote to origin/<current branch name>
    pub auto_set_upstream: bool,
}

impl<'a> PushOptions<'a> {
    pub const DEFAULT: Self  = Self {
        upstream: None,
        set_upstream: None,
        auto_set_upstream: false,
    };
    
    pub fn validate(self) -> Result<Self> {
        if self.auto_set_upstream && (self.upstream.is_some() || self.set_upstream.is_some()) {
            return Err(Error::OptionsValidate("PushOptions", "auto_set_upstream cannot be set with upstream or set_upstream"));
        } else if self.set_upstream.is_some() && (self.upstream.is_some() || self.auto_set_upstream) {
            return Err(Error::OptionsValidate("PushOptions", "set_upstream cannot be set with upstream or auto_set_upstream"));
        } else if self.upstream.is_some() && (self.set_upstream.is_some() || self.auto_set_upstream) {
            return Err(Error::OptionsValidate("PushOptions", "upstream cannot be set with set_upstream or auto_set_upstream"));
        }

        Ok(self)
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, Default)]
pub struct RebaseOptions {}

impl RebaseOptions {
    pub const DEFAULT: Self  = Self {};
    
    pub fn validate(self) -> Result<Self> {
        Ok(self)
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum ResetKind {
    #[default]
    Default,
    Soft,
    Hard,
}

#[non_exhaustive]
#[derive(Debug, Clone, Default)]
pub struct ResetOptions<'a> {
    pub kind: ResetKind,
    pub to_rev: Option<&'a str>,
}

impl<'a> ResetOptions<'a> {
    pub const DEFAULT: Self  = Self {
        kind: ResetKind::Default,
        to_rev: None,
    };
    
    pub fn validate(self) -> Result<Self> {
        Ok(self)
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, Default)]
pub struct StatusOptions {}

impl StatusOptions {
    pub const DEFAULT: Self  = Self {};
}

#[non_exhaustive]
#[derive(Debug, Clone, Default)]
pub struct SwitchBranchOptions {}

impl SwitchBranchOptions {
    pub const DEFAULT: Self  = Self {};
}
