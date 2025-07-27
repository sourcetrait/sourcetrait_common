//! Serializes and deserializes between strings and files for a Type.
//! Uses options specifed in [ronx_options].
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
//! Note that RON pretty serializing adds commas to everything, so you'll need
//! those as well.
//!
//! For an example, check out the integration test in this crate: `fromto.rs`

use crate::*;

/// RON options used to serialize / deserialize RON:
/// - unwrap newtypes and variant newtypes
/// - implicit Some
/// - explicit struct names
pub fn ronx_options() -> ron::Options {
    ron::Options::default().with_default_extension(
        ron::extensions::Extensions::UNWRAP_NEWTYPES
        | ron::extensions::Extensions::UNWRAP_VARIANT_NEWTYPES
        | ron::extensions::Extensions::IMPLICIT_SOME
        | ron::extensions::Extensions::EXPLICIT_STRUCT_NAMES
    )
}

/// Deserializes from a RON string or file to Self
/// 
/// See [module](self) for details.
pub trait FromRon {
    /// Deserializes from a RON string to Self
    fn from_ron(ron: &str) -> Result<Self>
    where
        Self: for<'de> serde::de::Deserialize<'de>
    {
        ronx_options()
            .from_str(&ron)
            .map_err(|e| Error::DeserializeRON(e))
    }

    /// Deserializes from a RON file (typically `.ron`) to Self
    fn from_ron_file(ron_filepath: &Path) -> Result<Self>
    where
        Self: for<'de> serde::de::Deserialize<'de>
    {
        fs::read_to_string(ron_filepath)
            .map_err(|e| Error::Io(FsErrMsg::ReadFile(RON), ron_filepath.to_path_buf(), e))
            .and_then(|s| Self::from_ron(&s))
    }
}

/// Serializes self to a RON string or file
/// 
/// See [module](self) for details.
pub trait ToRon {
    /// Serializes self to a RON string
    fn to_ron(&self) -> Result<String>
    where
        Self: serde::ser::Serialize
    {
        let options = ronx_options();
        let config = ::ron::ser::PrettyConfig::default()
            .struct_names(true)
            .extensions(options.default_extensions);
        
        options.to_string_pretty(self, config)
            .map_err(|e| Error::SerializeRON(e))
    }

    /// Serializes self to a RON file (typically `.ron`)
    fn to_ron_file(&self, ron_file: &Path) -> Result<()>
    where
        Self: serde::ser::Serialize
    {
        if let Some(parent) = ron_file.parent() && !parent.exists() {
            fs::create_dir_all(parent).map_err(|e| Error::Io(FsErrMsg::CreateDir, parent.to_path_buf(), e))?;
        }
        
        self.to_ron()
            .and_then(|ron| {
                fs::write(ron_file, ron)
                    .map_err(|e| Error::Io(FsErrMsg::WriteFile(RON), ron_file.to_path_buf(), e))
            })
    }
}
