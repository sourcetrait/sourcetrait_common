use std::path::PathBuf;
use thiserror;
use toml;
use stdx::error::fs::FsErrMsg;
use crate::*;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}: {1}")]
    Io(FsErrMsg, PathBuf, #[source] std::io::Error),
    #[error("Unable to deserialize from TOML")]
    DeserializeTOML(#[source] toml::de::Error),
    #[error("Unable to serialize to RON")]
    SerializeTOML(#[source] toml::ser::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

impl Into<std::io::Error> for Error {
    fn into(self) -> std::io::Error {
        match self {
            Error::Io(.., error) => error,
            Error::DeserializeTOML(error) => std::io::Error::new(std::io::ErrorKind::InvalidData, error),
            Error::SerializeTOML(error) => std::io::Error::new(std::io::ErrorKind::InvalidData, error),
        }
    }
}