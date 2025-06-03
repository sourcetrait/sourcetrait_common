use thiserror;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    //#[error("{0}")]
    //Io(#[from] std::io::Error)
}

pub type Result<T> = std::result::Result<T, Error>;