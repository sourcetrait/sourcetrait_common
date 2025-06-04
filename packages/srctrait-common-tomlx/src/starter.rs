//! Serializes / Deserializes between Self and an example TOML file with
//! configuration values commented out.
//! 
//! This is inteded to be used once per config as a first-run that introduces
//! defaults to the user.
//! 
//! Any comments remaining after the the user makes their edits are removed.
//! 
//! Trimming comments out, is potentially destructive, as the user may create
//! comments of their own, so we attempt to format our comments in a way that
//! we, hopefully, do not trample on the user's.
//! 
//! This is also the reason why we only use this method as a first-run. The
//! split-view method is preferred, if the user's $EDITOR supports it (and
//! they know how to use split-view in that editor).

use std::{fs, path::Path};
use crate::*;

pub trait ToStarterToml: ToToml {
    fn to_starter_toml(&self, intro: Option<&str>) -> Result<String>
    where
        Self: serde::ser::Serialize
    {
        let intro = intro.map(comment_out_text).unwrap_or_default();
        let content = comment_out_data(&self.to_toml()?);
        Ok(format!("{intro}{content}"))
    }
    
    fn to_starter_toml_file(&self, toml_file: &Path, intro: Option<&str>) -> Result<()>
    where
        Self: serde::ser::Serialize
    {
        let toml = self.to_starter_toml(intro)?;
        fs::write(toml_file, toml)
            .map_err(|e| Error::Io(format!("Unable to write to TOML file: {}", toml_file.display()), e))?;
        
        Ok(())
    }
}

pub fn trim_starter_toml_comments(toml: &str) -> String {
    toml
        .lines()
        .filter(|line| !line.starts_with('#') || line.starts_with("# "))
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn trim_starter_toml_file_comments(toml_file: &Path) -> Result<()> {
    let toml = fs::read_to_string(toml_file)
        .map_err(|e| Error::Io(format!("Unable to read from TOML file: {}", toml_file.display()), e))?;
    
    fs::write(toml_file, trim_starter_toml_comments(&toml))
        .map_err(|e| Error::Io(format!("Unable to write to TOML file: {}", toml_file.display()), e))?;
    
    Ok(())
}

pub const GENERIC_INTRO: Option<&'static str> = Some("Uncomment the settings that you wish to change.\nThe rest will be removed after you save this file.");

fn comment_out_data(toml: &str) -> String {
    toml
        .lines()
        .map(|line| format!("#{line}"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn comment_out_text(toml: &str) -> String {
    let mut toml = toml
        .lines()
        .map(|line| format!("#- {line}"))
        .collect::<Vec<_>>()
        .join("\n");
    
    toml.push_str("\n#\n");
    toml
}
