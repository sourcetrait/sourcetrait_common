use std::{fmt::Display, path::Path};
use crate::*;

/// Common to all testing models: [TestModule], [TestGroup], and [Test].
pub trait Testing {
    /// The canonical fixture directory, if configured to use one.
    /// By default, this is based on the use-case and namepath
    fn fixture_dir(&self) -> &Path;
    
    /// Retrieves the kind of testing model
    fn kind(&self) -> TestingKind;
    
    /// The namepath
    fn namepath(&self) -> &Namepath;

    /// The canonical temporary directory, if configured to use one.
    /// By default, this is based on the use-case and namepath.
    /// The directory is created on construction and deleted upon destruction.
    fn temp_dir(&self) -> &Path;
    
    /// The testing use-case
    fn use_case(&self)-> UseCase;
}

/// The type of testing being performed
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum UseCase {
    /// Unit tests
    Unit,
    /// Integration tests
    Integration,
    /// Examples
    Example,
    /// Benchmarks
    Benchmark,
}

impl Display for UseCase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UseCase::Unit => write!(f, "unit"),
            UseCase::Integration => write!(f, "integration"),
            UseCase::Example => write!(f, "example"),
            UseCase::Benchmark => write!(f, "benchmark"),
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

#[derive(Debug)]
pub enum Testable<'a, 'module, 'func> {
    Module(&'a TestModule),
    Group(&'a TestGroup),
    Test(&'a Test<'module, 'func>),
}

impl<'a, 'module, 'func> Testable<'a, 'module, 'func> {
    pub fn module(&self) -> Option<&'a TestModule> {
        match self {
            Self::Module(module) => Some(module),
            _ => None
        }
    }
    
    pub fn group(&self) -> Option<&'a TestGroup> {
        match self {
            Self::Group(group) => Some(group),
            _ => None
        }
    }
    
    pub fn test(&self) -> Option<&'a Test<'module, 'func>> {
        match self {
            Self::Test(test) => Some(test),
            _ => None
        }
    }
}

impl Testing for Testable<'_, '_, '_> {
    fn fixture_dir(&self) -> &Path {
        match self {
            Self::Module(module) => module.fixture_dir(),
            Self::Group(group) => group.fixture_dir(),
            Self::Test(test) => test.fixture_dir(),
        }
    }
    
    fn kind(&self) -> TestingKind {
        match self {
            Self::Module(_) => TestingKind::Module,
            Self::Group(_) => TestingKind::Group,
            Self::Test(_) => TestingKind::Test,
        }
    }

    fn namepath(&self) -> &Namepath {
        match self {
            Self::Module(module) => module.namepath(),
            Self::Group(group) => group.namepath(),
            Self::Test(test) => test.namepath(),
        }
    }
    
    fn temp_dir(&self) -> &Path {
        match self {
            Self::Module(module) => module.temp_dir(),
            Self::Group(group) => group.temp_dir(),
            Self::Test(test) => test.temp_dir(),
        }
    }

    fn use_case(&self) -> UseCase {
        match self {
            Self::Module(module) => module.use_case(),
            Self::Group(group) => group.use_case(),
            Self::Test(test) => test.use_case(),
        }
    }
}