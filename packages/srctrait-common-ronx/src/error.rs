use thiserror;
use ron;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Ron(#[from] ron::Error),
    #[error("{0}")]
    RonSpanned(#[from] ron::error::SpannedError),
}

pub type Result<T> = std::result::Result<T, Error>;
