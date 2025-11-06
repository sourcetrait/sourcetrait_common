#![doc = include_str!("../docs/DOC/1.head.md")]
#![doc = include_str!("../docs/DOC/3.foot.md")]

#![allow(dead_code)] // DRAFT

pub mod definition;
pub mod designator;
pub mod basedir;
pub mod error;
pub mod integration;
pub mod reserved;
pub mod tenant;
pub mod walk;

pub mod strings {
    pub const DOTREPO: &'static str = ".repo";
    pub const GLOBS: &'static str = "globs";
    pub(crate) const DOTGIT: &'static str = ".git";
    pub(crate) const DOTGITIGNORE: &'static str = ".gitignore";
}

pub use self::{
    basedir::*,
    designator::*,
    definition::*,
    designator::{standard::*, composite::*},
    error::*,
    integration::{*, git::*},
    reserved::{*, files::*},
    strings::*,
    tenant::*,
    walk::*,
};

