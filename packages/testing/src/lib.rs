#![doc = include_str!("../docs/DOC/1.head.md")]
//! ## Example
//! ```rust
#![doc = include_str!("../examples/example-fibonacci.rs")]
//! ```
#![doc = include_str!("../docs/DOC/3.foot.md")]

mod namepath;
mod test;
mod group;
mod module;
mod testing;
mod util;
mod teardown;
pub mod tooling;

pub use crate::{
    namepath::{Namepath, RawNamepath},
    group::{TestGroup, GroupBuilder, Group},
    module::{TestModule, ModuleBuilder, Module},
    test::{Test, TestBuilder},
    testing::{TestingKind, UseCase, Testing},
};


pub(crate) use crate::{teardown::*, util::*, tooling::*};
pub(crate) use anyhow::{bail, Context};

pub mod prelude {
    pub use crate as testing;
    pub use crate::tooling as tooling;
    pub use crate::Testing;
    pub use function_name::named;
    pub use asmov_common_testing_macro::tested;
}

pub(crate) mod strings {
    pub(crate) const TESTING: &'static str = "testing";
    pub(crate) const FIXTURES: &'static str = "fixtures";
}
