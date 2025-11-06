//use crate::*;

#[derive(Debug, Clone, snafu::Snafu)]
pub enum TwoStrError {
    IntoCString,
    Utf8,
}

pub type TwoStrResult<T> = Result<T, TwoStrError>;