#![doc = include_str!("../docs/DOC/1.head.md")]
#![doc = include_str!("../docs/DOC/3.foot.md")]

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