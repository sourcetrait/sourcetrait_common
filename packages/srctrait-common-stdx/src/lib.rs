#![doc = include_str!("../docs/DOC/1.head.md")]
#![doc = include_str!("../docs/DOC/3.foot.md")]

/// Path utilities
pub mod path {
    pub mod normalize;
    pub mod tree;
}

/// Filesystem utilities
pub mod fs {
    pub mod find;
}
