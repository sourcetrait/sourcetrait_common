use thiserror;
use toml;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Io(String, #[source] std::io::Error),
    #[error("{0}")]
    DeserializeTOML(String, #[source] toml::de::Error),
    #[error("{0}")]
    SerializeTOML(String, #[source] toml::ser::Error),
}

pub type Result<T> = std::result::Result<T, Error>;