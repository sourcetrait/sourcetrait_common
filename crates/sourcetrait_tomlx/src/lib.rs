pub mod error;
pub mod fromto;
pub mod starter;

pub use self::{
    error::*,
    fromto::{FromToml, ToToml},
    starter::ToStarterToml
};

pub mod prelude {
    pub use crate::fromto::{FromToml, ToToml};
}

pub(crate) const TOML: &'static str = "TOML";

pub(crate) use sourcetrait_stdx as stdx;
