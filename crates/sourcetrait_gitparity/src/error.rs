use std::{fmt::Display, path::PathBuf};
use thiserror;
#[allow(unused_imports)] use crate::*;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Unable to open git repository: {0}")]
    Open(PathBuf, #[source] ErrSrc, Option<String>),
    #[error("Unable to initialize git repository: {0}")]
    Init(PathBuf, #[source] ErrSrc, Option<String>),
    #[error("Unable to clone git repository: {0}")]
    Clone(PathBuf, #[source]ErrSrc, Option<String>),
    #[error("Unable to add path(s) to git repository: {0}")]
    Add(String, #[source]ErrSrc, Option<String>),
    #[error("Git branch not found: {0}")]
    BranchNotFound(String, #[source]ErrSrc),
    #[error("Unable to configure git branch: {0}")]
    BranchConfigure(String, #[source]ErrSrc, Option<String>),
    #[error("Unable to switch to git branch: {0}")]
    BranchSwitch(String, #[source]ErrSrc, Option<String>),
    #[error("Unable to create git branch `{0}` from `{1}`")]
    BranchCreate(String, String, #[source]ErrSrc),
    #[error("Unable to delete git branch: {0}")]
    BranchDelete(String, #[source]ErrSrc),
    #[error("Unable to commit to git repository")]
    Commit(#[source] ErrSrc, Option<String>),
    #[error("Unable to connect to remote git repository: {0}")]
    Connect(String, #[source] ErrSrc, Option<String>),
    #[error("Unable to read git config for: {0}")]
    Config(String, #[source]ErrSrc),
    #[error("Unable to diff between: {0} and {1}{err}", err = .3.as_ref()
        .and_then(|s| Some([": ", s].concat())).unwrap_or_default())]
    Diff(String, String, #[source] ErrSrc, Option<String>),
    #[error("Unable to fetch for git repository")]
    Fetch(#[source]ErrSrc, Option<String>),
    #[error("Unable to move git repository path: {0} to {1}")]
    Move(String, String, #[source]ErrSrc, Option<String>),
    #[error("Unable to pull for git repository")]
    Pull(#[source]ErrSrc, Option<String>),
    #[error("{0} is not fast-forwardable for git branch: {1}")]
    FastForward(&'static str, String),
    #[error("Aborted {0} due to conflict in git branch: {1}")]
    Conflict(&'static str, String),
    #[error("Unable to review logs for git repository")]
    Log(#[source] ErrSrc, Option<String>),
    #[error("Aborted merge from: {0}{err}", err = .2.as_ref()
        .and_then(|s| Some([" :: ", s].concat())).unwrap_or_default())]
    MergeAborted(String, #[source] ErrSrc, Option<String>),
    #[error("Unable to merge (unaborted) from: {0}{err}", err = .2.as_ref()
        .and_then(|s| Some([" :: ", s].concat())).unwrap_or_default())]
    MergeUnaborted(String, #[source] ErrSrc, Option<String>),
    #[error("Unable to push git repository")]
    Push(#[source]ErrSrc, Option<String>),
    #[error("Path not found in git repository: {0} : {1}")]
    PathNotFound(String, String, #[source] ErrSrc, Option<String>),
    #[error("Invalid {0}: {1}")]
    OptionsValidate(&'static str, &'static str),
    #[error("Git remote not found: {0}")]
    RemoteNotFound(String, #[source]ErrSrc),
    #[error("Unable to reset branch{}", .0.as_ref()
        .map_or(String::new(), |s| [": ", s].concat()))]
    Reset(Option<String>, #[source] ErrSrc, Option<String>),
    #[error("Git revision not found: {0}")]
    RevNotFound(String, #[source] ErrSrc, Option<String>),
    #[error("Git upstream not configured for branch: {0}")]
    UpstreamNotFound(String, #[source]ErrSrc),
    #[error("Unexpected git repository state: {0}")]
    State(StateErr, #[source] ErrSrc),

    #[error("{0}")]
    FromStr(String),
    #[error("Unable to run command: git")]
    GitCmdRun,
    #[error("Git command error ({0}){err}", err = .1.as_ref()
        .and_then(|s| Some([": ", s].concat())).unwrap_or_default())]
    GitCmd(i32, Option<String>),
    #[error("Failed to parse git status")]
    GitStatusParse,
    #[error("Failed to parse git log")]
    GitLogParse,
}

#[derive(Debug)]
pub enum ErrSrc {
    None,
    Io(std::io::Error),
    Lib(Box<crate::Error>),
    #[cfg(feature = "gitc")]
    GitC(git2::Error)
}

impl std::error::Error for ErrSrc {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::None => None,
            Self::Io(e) => e.source(),
            Self::Lib(e) => e.source(),
            Self::GitC(e) => e.source(),
        }
    }
}

impl Display for ErrSrc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => Ok(()),
            Self::Io(e) => e.fmt(f),
            Self::Lib(e) => e.fmt(f),
            Self::GitC(e) => e.fmt(f),
        }
    }
}

#[derive(Debug, Copy, Clone, strum::Display)]
pub enum StateErr {
    #[strum(to_string = "Index does not exist")]
    IndexNotFound,
    #[strum(to_string = "Failed to access internals file")]
    InternalsFileIO,
    #[strum(to_string = "Unexpected status of repository")]
    UnexpectedStatus,
    #[strum(to_string = "HEAD does not exist")]
    HeadNotFound,
    #[strum(to_string = "Upstream does not exist")]
    UpstreamNotFound,
    #[strum(to_string = "HEAD branch name not available")]
    HeadNameNotFound,
    #[strum(to_string = "HEAD commit not available")]
    HeadCommitNotFound,
    #[strum(to_string = "Referenced commit not available")]
    ReferenceCommitNotFound,
    #[strum(to_string = "Current branch not available")]
    CurrentBranchNotFound,
    #[strum(to_string = "No Git branches do not exist")]
    BranchesNotFound,
    #[strum(to_string = "Branch name is not UTF8")]
    UnsupportedBranchName,
    #[strum(to_string = "Abort failed after conflict")]
    AbortFailed,
}

pub type Result<T> = std::result::Result<T, Error>;
