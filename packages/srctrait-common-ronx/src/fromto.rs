//! Serializes and deserializes between strings and files for a Type.
//!
//! Uses options specifed in [ronx_options].
//!

use std::path::Path;
use ron;
use serde;
use crate::*;

/// RON options used to serialize / deserialize RON:
/// - Unwrap newtypes
/// - Unwrap variant newtypes
/// - Implicit Some
pub fn ronx_options() -> ron::Options {
    ron::Options::default().with_default_extension(
        ron::extensions::Extensions::UNWRAP_NEWTYPES
        | ron::extensions::Extensions::UNWRAP_VARIANT_NEWTYPES
        | ron::extensions::Extensions::IMPLICIT_SOME
    )
}

/// Serializes and deserializes between strings and files for a Type.
/// Uses options specifed in [ronx_options].
///
/// ### Roundtrip
/// For exact round-tripping on Option fields, use on each:
///
/// `#[serde(skip_serializing_if = "Option::is_none")]`
///
/// Even easier, use the `serde_with` crate against the type itself:
///
/// [docs.rs / serde_with / skip_serializing_none](https://docs.rs/serde_with/latest/serde_with/attr.skip_serializing_none.html)
///
/// Note that RON pretty serializing adds commas to everything, so you'll need
/// those as well.
///
/// For an example, check out the integration test in this crate: `fromto.rs`
pub trait FromRonToRon {
    /// Deserializes from a RON string to a Type
    fn from_ron(ron: &str) -> Result<Self>
    where
        Self: serde::de::DeserializeOwned
    {
        let this = ronx_options().from_str(&ron)?;
        Ok(this)
    }

    /// Deserializes from a file (typically `.ron`) in RON format to a Type
    fn from_ron_file(ron_filepath: &Path) -> Result<Self>
    where
        Self: serde::de::DeserializeOwned
    {
        let ron = std::fs::read_to_string(ron_filepath)?;
        let this = ronx_options().from_str(&ron)?;
        Ok(this)
    }

    /// Serializes from a Type to a RON string
    fn to_ron(&self) -> Result<String>
    where
        Self: serde::ser::Serialize
    {
        let options = ronx_options();
        let config = ::ron::ser::PrettyConfig::default()
            .extensions(options.default_extensions);
        let ron = options.to_string_pretty(self, config)?;
        Ok(ron)
    }

    /// Serializes from a Type to a file (typically `.ron`)
    fn to_ron_file(&self, filepath: &Path) -> Result<()>
    where
        Self: serde::ser::Serialize
    {
        let config = ::ron::ser::PrettyConfig::default();
        let ron = ronx_options().to_string_pretty(self, config)?;
        std::fs::write(filepath, ron)?;
        Ok(())
    }
}
