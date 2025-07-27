#![doc = include_str!("../docs/DOC/1.head.md")]
//! ## Example
//! ```rust
#![doc = include_str!("../examples/example-fibonacci.rs")]
//! ```
#![doc = include_str!("../docs/DOC/3.foot.md")]

mod group;
mod module;
mod namepath;
mod teardown;
mod test;
mod testing;
mod stepper;
mod util;

pub use crate::{
    group::{TestGroup, GroupBuilder, Group},
    module::{TestModule, ModuleBuilder, Module},
    namepath::{Namepath, RawNamepath},
    test::{Test, TestBuilder},
    testing::{TestingKind, UseCase, Testing, Testable},
    stepper::{Stepper, StepperBuilder, StepState},
};

pub(crate) use crate::{teardown::*, util::*};
pub(crate) use anyhow::{bail, Context};

#[cfg(feature = "tooling")]
pub use sourcetrait_tooling as tooling;

pub mod prelude {
    pub use crate as testing;
    pub use crate::Testing;
    pub use function_name::named;
    pub use sourcetrait_testing_macro::{benched, tested};

    #[cfg(feature = "tooling")]
    pub use crate::tooling as tooling;
}

pub(crate) mod strings {
    pub(crate) const TESTING: &'static str = "testing";
    pub(crate) const FIXTURES: &'static str = "fixtures";
}
