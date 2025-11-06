pub(crate) mod data;

pub use crate::{
    data::*,
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

pub use sourcetrait_cereal_macro::*;

pub(crate) use std::{
    fmt::Debug,
    hash::Hash,
};
