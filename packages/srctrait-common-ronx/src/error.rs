use std::path::PathBuf;
use thiserror;
use ron;
use stdx::error::fs::FsErrMsg;
use crate::*;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}: {1}")]
    Io(FsErrMsg, PathBuf, #[source] std::io::Error),
    #[error("Unable to deserialize from RON")]
    DeserializeRON(#[source] ron::error::SpannedError),
    #[error("Unable to serialize to RON")]
    SerializeRON(#[source] ron::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
