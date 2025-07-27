#![doc = include_str!("../docs/DOC/1.head.md")]
#![doc = include_str!("../docs/DOC/3.foot.md")]

/// Error and Result utilities
pub mod error {
    pub mod fs;
}
 
/// Filesystem utilities
pub mod fs {
    pub mod find;
    
    pub use find::*;
}

pub mod option {
    pub mod either;
}

/// Path utilities
pub mod path {
    pub mod normalize;
    pub mod tree;
}
