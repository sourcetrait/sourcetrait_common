use std::{ops::{Deref, DerefMut}, path::PathBuf};
use crate::*;

pub enum GitParity {
    Cli(GitCli),
    #[cfg(feature = "libc")]
    LibC(GitLibC)
}

#[derive(Debug, Clone, Copy, strum::Display)]
#[strum(serialize_all = "kebab-case")]
pub enum GitKind {
    Cli,
    #[cfg(feature = "libc")]
    #[strum(serialize = "libc")]
    LibC,
}

impl GitParity {
    pub fn clone(kind: GitKind, repository: &str, top_dir: PathBuf) -> Result<Self> {
        Ok(match kind {
            GitKind::Cli => Self::Cli(GitCli::clone(repository, top_dir)?),
            #[cfg(feature = "libc")]
            GitKind::LibC => Self::LibC(GitLibC::clone(repository, top_dir)?),
        })
    }
    
    pub fn clone_with(
        kind: GitKind,
        repository: &str,
        top_dir: PathBuf,
        options: CloneOptions
    ) -> Result<Self> {
        Ok(match kind {
            GitKind::Cli => Self::Cli(GitCli::clone_with(repository, top_dir, options)?),
            #[cfg(feature = "libc")]
            GitKind::LibC => Self::LibC(GitLibC::clone_with(repository, top_dir, options)?),
        })
    }
    
    pub fn init(
        kind: GitKind,
        top_dir: PathBuf,
        initial_branch_name: &str,
    ) -> Result<Self> {
        Ok(match kind {
            GitKind::Cli => Self::Cli(GitCli::init(top_dir, initial_branch_name)?),
            #[cfg(feature = "libc")]
            GitKind::LibC => Self::LibC(GitLibC::init(top_dir, initial_branch_name)?),
        })
    }
    
    pub fn init_with(
        kind: GitKind,
        top_dir: PathBuf,
        initial_branch_name: &str,
        options: InitOptions,
    ) -> Result<Self> {
        Ok(match kind {
            GitKind::Cli => Self::Cli(GitCli::init_with(top_dir, initial_branch_name, options)?),
            #[cfg(feature = "libc")]
            GitKind::LibC => Self::LibC(GitLibC::init_with(top_dir, initial_branch_name, options)?),
        })
    }
    
    pub fn init_bare(
        kind: GitKind,
        top_dir: PathBuf,
        initial_branch_name: &str,
    ) -> Result<Self> {
        Ok(match kind {
            GitKind::Cli => Self::Cli(GitCli::init_bare(top_dir, initial_branch_name)?),
            #[cfg(feature = "libc")]
            GitKind::LibC => Self::LibC(GitLibC::init_bare(top_dir, initial_branch_name)?),
        })
    }
    
    pub fn init_bare_with(
        kind: GitKind,
        top_dir: PathBuf,
        initial_branch_name: &str,
        options: InitBareOptions
    ) -> Result<Self> {
        Ok(match kind {
            GitKind::Cli => Self::Cli(GitCli::init_bare_with(top_dir, initial_branch_name, options)?),
            #[cfg(feature = "libc")]
            GitKind::LibC => Self::LibC(GitLibC::init_bare_with(top_dir, initial_branch_name, options)?),
        })
    }
    
    pub fn open(kind: GitKind, path: PathBuf) -> Result<Self> {
        Ok(match kind {
            GitKind::Cli => Self::Cli(GitCli::open(path)?),
            #[cfg(feature = "libc")]
            GitKind::LibC => Self::LibC(GitLibC::open(path)?),
        })
    }
    
    pub fn open_with(kind: GitKind, path: PathBuf, options: OpenOptions) -> Result<Self> {
        Ok(match kind {
            GitKind::Cli => Self::Cli(GitCli::open_with(path, options)?),
            #[cfg(feature = "libc")]
            GitKind::LibC => Self::LibC(GitLibC::open_with(path, options)?),
        })
    }
}

//todo: remove this and replace with static dispatch
impl Deref for GitParity {
    type Target = dyn WorkingRepo;

    fn deref(&self) -> &Self::Target {
        match self {
            GitParity::Cli(cli) => cli,
            #[cfg(feature = "libc")]
            GitParity::LibC(lib) => lib,
        }
    }
}

//todo: remove this and replace with static dispatch
impl DerefMut for GitParity {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            GitParity::Cli(cmd) => cmd,
            #[cfg(feature = "libc")]
            GitParity::LibC(lib) => lib,
        }
    }
}
