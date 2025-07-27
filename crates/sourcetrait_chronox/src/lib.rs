#![doc = include_str!("../docs/DOC/1.head.md")]
#![doc = include_str!("../docs/DOC/3.foot.md")]

pub mod format;
pub mod parse;

pub use self::{format::datetime::*, parse::date::*};
