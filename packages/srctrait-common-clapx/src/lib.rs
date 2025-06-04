#![doc = include_str!("../docs/DOC/1.head.md")]
#![doc = include_str!("../docs/DOC/3.foot.md")]

pub mod error;
pub mod style;
pub mod subcmd;

pub use self::{error::*, style::{*, styl}};