#![doc = include_str!("../docs/DOC/1.head.md")]
#![doc = include_str!("../docs/DOC/3.foot.md")]

mod tar;

/// Path tools
pub mod path {
    pub mod diff;

    pub use self::diff::*;
}

pub use self::path::*;
