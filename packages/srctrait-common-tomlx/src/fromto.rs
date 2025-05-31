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

use std::path::Path;
use serde;
use toml;
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
        let this = toml::from_str(&toml)?;
        Ok(this)
    }

    /// Deserializes from a TOML file (typically `.toml`) to Self
    fn from_toml_file(toml_filepath: &Path) -> Result<Self>
    where
        Self: serde::de::DeserializeOwned
    {
        let toml = std::fs::read_to_string(toml_filepath)?;
        let this = Self::from_toml(&toml)?;
        Ok(this)
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
        let toml = toml::to_string_pretty(self)?;
        Ok(toml)
    }

    /// Serializes self to a TOML file (typically `.toml`)
    fn to_toml_file(&self, filepath: &Path) -> Result<()>
    where
        Self: serde::ser::Serialize
    {
        let toml = Self::to_toml(&self)?;
        std::fs::write(filepath, toml)?;
        Ok(())
    }
}
