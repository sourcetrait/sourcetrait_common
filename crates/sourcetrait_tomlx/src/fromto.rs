//! Serializes and deserializes TOML between strings and files for a Type.
//!
//! ### Roundtrip
//! For exact round-tripping on Option fields, use on each:
//!
//! `#[serde(skip_serializing_if = "Option::is_none")]`
//!
//! Even easier, use the `serde_with` crate against the type itself:
//!
//! [docs.rs / serde_with / skip_serializing_none](https://docs.rs/serde_with/latest/serde_with/attr.skip_serializing_none.html)
//!
//! For an example, check out the integration test in this crate: `fromto.rs`

use std::{fs, path::Path};
use serde;
use toml;
use stdx::error::fs::FsErrMsg;
use crate::*;

/// Deserializes from a TOML string or file to Self
/// 
/// See [module](self) for details.
pub trait FromToml {
    /// Deserializes from a TOML string to Self
    fn from_toml(toml: &str) -> Result<Self>
    where
        Self: serde::de::DeserializeOwned
    {
        toml::from_str(&toml)
            .map_err(|e| Error::DeserializeTOML(e))
    }

    /// Deserializes from a TOML file (typically `.toml`) to Self
    fn from_toml_file(toml_file: &Path) -> Result<Self>
    where
        Self: serde::de::DeserializeOwned
    {
        fs::read_to_string(toml_file)
            .map_err(|e| Error::Io(FsErrMsg::ReadFile(TOML), toml_file.to_path_buf(), e))
            .and_then(|toml| Self::from_toml(&toml))
    }
}

/// Serializes self into a TOML string or file
/// 
/// See [module](self) for details.
pub trait ToToml {
    /// Serializes self to a TOML string
    fn to_toml(&self) -> Result<String>
    where
        Self: serde::ser::Serialize
    {
        toml::to_string_pretty(self)
            .map_err(|e| Error::SerializeTOML(e))
    }

    /// Serializes self to a TOML file (typically `.toml`)
    fn to_toml_file(&self, toml_file: &Path) -> Result<()>
    where
        Self: serde::ser::Serialize
    {
        if let Some(parent) = toml_file.parent() && !parent.exists() {
            fs::create_dir_all(parent).map_err(|e| Error::Io(FsErrMsg::CreateDir, parent.to_path_buf(), e))?;
        }
        
        Self::to_toml(&self)
            .and_then(|toml| fs::write(toml_file, toml)
                .map_err(|e| Error::Io(FsErrMsg::WriteFile(TOML), toml_file.to_path_buf(), e)))
    }
}
