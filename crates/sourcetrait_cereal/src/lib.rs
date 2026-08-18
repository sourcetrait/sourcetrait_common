pub(crate) mod data;
pub mod bitcoded;

pub use crate::{
    data::{
        Data,
        DataCopy,
        DataEq,
        DataCopyEq,
        Archive,
    },
};

pub mod prelude {
    pub use crate::{
        Data,
        DataCopy,
        DataEq,
        DataCopyEq,
        Archive,
    };
}

pub(crate) use std::{
    fmt::Debug,
};
