//! Utilities for standardized Errors

use std::fmt::Display;

/// Displays: "Unable to <operation> <from/to> <file type> <file/directory>"
#[derive(Debug, Copy, Clone)]
pub enum FsErrMsg {
    CreateDir,
    ReadFile(&'static str),
    WriteFile(&'static str),
    AccessFile(&'static str),
}

impl Display for FsErrMsg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CreateDir => write!(f, "Unable to create directory"),
            Self::ReadFile(filetype) => write!(f, "Unable to read from {filetype} file"),
            Self::WriteFile(filetype) => write!(f, "Unable to write to {filetype} file"),
            Self::AccessFile(filetype) => write!(f, "No permission to access {filetype} file"),
        }
    }
}
