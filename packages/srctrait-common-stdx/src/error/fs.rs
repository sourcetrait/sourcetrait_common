//! Utilities for standardized Errors

use std::fmt::Display;

/// Displays: "Unable to <operation> <from/to> <file type> <file/directory>"
#[derive(Debug, Copy, Clone)]
pub enum FsErrMsg {
    ReadFile(&'static str),
    WriteFile(&'static str)
}

impl Display for FsErrMsg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ReadFile(filetype) => write!(f, "Unable to read from {filetype} file"),
            Self::WriteFile(filetype) => write!(f, "Unable to write to {filetype} file"),
        }
    }
}
