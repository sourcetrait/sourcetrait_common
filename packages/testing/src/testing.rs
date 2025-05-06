use std::{fmt::Display, path::Path};
use crate::*;

/// Common to all testing models: [TestModule], [TestGroup], and [Test].
pub trait Testing {
    /// The testing use-case
    fn use_case(&self)-> UseCase;

    /// The namepath
    fn namepath(&self) -> &Namepath;

    /// The canonical fixture directory, if configured to use one.
    /// By default, this is based on the use-case and namepath
    fn fixture_dir(&self) -> &Path;

    /// The canonical temporary directory, if configured to use one.
    /// By default, this is based on the use-case and namepath.
    /// The directory is created on construction and deleted upon destruction.
    fn temp_dir(&self) -> &Path;
}

/// The type of testing being performed
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum UseCase {
    /// Unit tests
    Unit,
    /// Integration tests
    Integration,
}

impl Display for UseCase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UseCase::Unit => write!(f, "unit"),
            UseCase::Integration => write!(f, "integration"),
        }
    }
}

/// The type of testing model
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestingKind {
    /// [TestModule]
    Module,
    /// [TestGroup]
    Group,
    /// [Test]
    Test
}

impl Display for TestingKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TestingKind::Module => write!(f, "module"),
            TestingKind::Group => write!(f, "group"),
            TestingKind::Test => write!(f, "test"),
        }
    }
}
