#![allow(dead_code)] // DRAFT

pub mod imp {
    pub mod cli;
    
    #[cfg(feature = "gitc")]
    pub mod gitc;
}

pub mod error;
pub mod interface;
pub mod parity;

pub mod prelude {
    pub use crate::{
        parity::{GitParity, GitKind},
        types::GitEnv,
        interface::{
            WorkingRepo, GitInterface, GitInterfaceConstruct,
            status::{
                StatusTrait,
                PathStatusTrait,
            },
        },
    };
    
    pub use std::ops::Deref;
    pub use std::str::FromStr;
}

pub use crate::{
    parity::*,
    imp::cli::*,
    error::*,
    interface::{
        *,
        options::*,
        log::*,
        status::*,
        types::*
    },
};

#[cfg(feature = "gitc")]
pub use crate::imp::gitc::*;

pub const HEAD: &'static str = "HEAD";
pub const MAIN: &'static str = "main";
pub const MASTER: &'static str = "master";
pub const ORIGIN: &'static str = "origin";
pub const PATHSPEC_ALL: &'static str = "."; 

pub(crate) const DOT_GIT: &'static str = ".git";
pub(crate) const GIT: &'static str = "git";
pub(crate) const MERGE_HEAD: &'static str = "MERGE_HEAD";
pub(crate) const MERGE_MSG: &'static str = "MERGE_MSG";
pub(crate) const MERGE_MODE: &'static str = "MERGE_MODE";

pub(crate) const REFLOG_FAST_FORWARD: &'static str = "Fast-forward";

pub(crate) const PULL: &'static str = "Pull";

pub(crate) use sourcetrait_stdx as stdx;
pub(crate) use std::str::FromStr;

#[cfg(feature = "testlib")]
pub mod testlib;
