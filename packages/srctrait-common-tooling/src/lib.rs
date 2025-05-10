#![doc = include_str!("../docs/DOC/1.head.md")]
#![doc = include_str!("../docs/DOC/3.foot.md")]

mod tar;

/// File tools
pub mod file {
    pub mod touch;

    pub use self::touch::*;
}

/// Path tools
pub mod path {
    pub mod diff;

    pub use self::diff::*;
}

pub use self::{file::*, path::*};
