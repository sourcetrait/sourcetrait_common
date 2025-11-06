pub(crate) mod group;
pub(crate) mod helper {
    pub(crate) mod process;
}
pub(crate) mod module;
pub(crate) mod namepath;
pub(crate) mod teardown;
pub(crate) mod test;
pub(crate) mod testing;
pub(crate) mod stepper;
pub(crate) mod util;

pub use crate::{
    group::{TestGroup, GroupBuilder, Group},
    helper::{
        process::TestOutputTrait,
    },
    module::{
        TestModule, ModuleBuilder, Module,
        TestModuleWith, ModuleBuilderWith, ModuleWith,
    },
    namepath::{Namepath, RawNamepath},
    test::{Test, TestBuilder, TestWith, TestBuilderWith},
    testing::{TestingKind, UseCase, Testing, Testable},
    stepper::{Stepper, StepperBuilder, StepState},
};

#[cfg(feature = "tooling")]
pub use sourcetrait_tooling as tooling;

pub mod prelude {
    pub use crate as testing;
    pub use crate::{
        Testing,
        TestOutputTrait,
    };
    pub use function_name::named;
    pub use sourcetrait_testing_macro::{benched, tested};

    #[cfg(feature = "tooling")]
    pub use crate::tooling as tooling;
}

pub(crate) mod strings {
    pub(crate) const TESTING: &'static str = "testing";
    pub(crate) const FIXTURES: &'static str = "fixtures";
}

pub(crate) use crate::{
    teardown::*, util::*
};

pub(crate) use anyhow::{bail, Context};
pub(crate) use indexmap::IndexMap;

pub(crate) use std::{
    ffi::OsStr,
    fmt::Display,
    hash::Hash,
    ops::Deref,
    path::{Path, PathBuf},
    sync::{Arc, LazyLock, Mutex},
};
