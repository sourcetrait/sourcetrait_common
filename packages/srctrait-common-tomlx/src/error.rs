use thiserror;
use toml;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    TomlFrom(#[from] toml::de::Error),
    #[error("{0}")]
    TomlTo(#[from] toml::ser::Error),
}

pub type Result<T> = std::result::Result<T, Error>;