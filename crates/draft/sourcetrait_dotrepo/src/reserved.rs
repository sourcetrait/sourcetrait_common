pub mod files;

use std::path::Path;
use strum;

#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::Display, strum::AsRefStr, strum::EnumIter)]
pub enum ReservedDirKind {
    #[strum(serialize = "state")]
    State,
    #[strum(serialize = "exclude")]
    Exclude,
    #[strum(serialize = "local")]
    Local,
    #[strum(serialize = "local/state")]
    LocalState,
    #[strum(serialize = "designator")]
    Designator
}

impl ReservedDirKind {
    pub fn is_top(self) -> bool {
        match self {
            Self::State => true,
            Self::Designator => true,
            _ => false
        }
    }
    
    pub fn relative_path(&self) -> &Path {
        Path::new(self.as_ref())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::Display, strum::AsRefStr)]
pub enum ReservedFileKind {
    #[strum(serialize = "state/version.semver")]
    SemVer,
    #[strum(serialize = "exclude.globs")]
    StandardExclude,
}

impl ReservedFileKind {
    pub fn relative_path(&self) -> &Path {
        Path::new(self.as_ref())
    }
}
