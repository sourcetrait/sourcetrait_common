#![doc = include_str!("../docs/DOC/1.head.md")]
#![doc = include_str!("../docs/DOC/3.foot.md")]


pub mod error;
pub mod fromto;

pub use self::{
    error::{Error, Result},
    fromto::{FromRon, ToRon}
};

pub mod prelude {
    pub use crate::{FromRon, ToRon};
}
