use crate::*;

/// Describes the normalized namepath of a [TestModule], [TestGroup], or [Test].
///
/// Module and Test reflect their [Rust path](https://doc.rust-lang.org/reference/paths.html).
///
/// Group uses an arbitrary path.
///
/// Components such as '::tests' are removed.
///
/// Module: Created using [module_path!()]
///
/// Test: Created using [module_path!()] and [function_name]
///
/// Group: Created using an arbitrary slash-separated slug (e.g., "foo/hat-cat")
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Namepath {
    pub(crate) full_path: PathBuf,
    pub(crate) raw: RawNamepath
}

impl Namepath {
    pub fn new_module(package_name: &'static str, use_case: UseCase, module_path: &'static str) -> anyhow::Result<Self> {
        let raw = RawNamepath {
            kind: TestingKind::Module,
            use_case,
            package_name,
            path: module_path,
            name: None,
        };

        let full_path = normalize_path(&raw)?;

        Ok(Self {
            full_path,
            raw,
        })
    }

    pub fn new_group(package_name: &'static str, use_case: UseCase, path: &'static str) -> anyhow::Result<Self> {
        let raw = RawNamepath {
            kind: TestingKind::Group,
            use_case,
            package_name,
            path,
            name: None,
        };

        let full_path = normalize_path(&raw)?;

        Ok(Self {
            full_path,
            raw,
        })
    }

    pub fn new_test(package_name: &'static str, use_case: UseCase, module_path: &'static str, function_name: &'static str) -> anyhow::Result<Self> {
        let raw = RawNamepath {
            kind: TestingKind::Test,
            use_case,
            package_name,
            path: module_path,
            name: Some(function_name),
        };

        let full_path = normalize_path(&raw)?;

        Ok(Self {
            full_path,
            raw,
        })
    }

    /// The normalized path.
    /// Eg., `use-case/module-path../function-name`
    pub fn path(&self) -> &Path {
        let mut components = self.full_path.components();
        components.next();
        components.as_path()
    }

    /// The normalized path, including its package name.
    /// Eg., `package-name/use-case/module-path../function-name`
    pub fn full_path(&self) -> &Path {
        &self.full_path
    }

    pub fn full_path_to_squashed_slug(&self) -> String {
        self.full_path
            .to_str().expect("Invalid namepath")
            .replace("/", "_")
    }

    /// The crate name or equivalent
    pub fn package_name(&self) -> &str {
        self.full_path.components()
            .next().expect("Invalid path")
            .as_os_str().to_str().expect("Invalid path")
    }

    /// The kind of testing model this namepath refers to
    pub fn kind(&self) -> TestingKind {
        self.raw.kind
    }

    // The testing use-case
    pub fn use_case(&self) -> UseCase {
        self.raw.use_case
    }

    /// The base name
    /// Eg., a path of `unit/mymod/foo/bar` has the name: `bar`
    pub fn name(&self) -> &str {
        self.full_path.components()
            .last().expect("Invalid namepath")
            .as_os_str().to_str().expect("Invalid path")
    }

    pub fn raw(&self) -> &RawNamepath {
        &self.raw
    }
}

impl Display for Namepath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.path().display().fmt(f)
    }
}

impl Hash for Namepath {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.full_path().hash(state);
    }
}

/// Retains the original elements used to construct a [Namepath]
///
/// Primarily used for debugging namepath problems.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawNamepath {
    pub kind: TestingKind,
    pub use_case: UseCase,
    pub package_name: &'static str,
    pub path: &'static str,
    pub name: Option<&'static str>
}

impl Display for RawNamepath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{};{};{};{}", self.kind, self.use_case, self.package_name, self.path)?;

        if let Some(name) = &self.name {
            write!(f, ";{}", name)?;
        }

        Ok(())
    }
}

/// Sanitizes paths if they've been created using module_path!() and function_name!().
/// Strips the crate name prefix and the test/tests suffix.
/// If the path is from lib.rs, the crate name is returned.
/// Primarily for module and Test. Group uses plain strings.
fn normalize_path(raw: &RawNamepath) -> anyhow::Result<PathBuf> {
    static REGEX_NAMESPACE_BENCHMARK: LazyLock<regex::Regex> = LazyLock::new(|| {
        regex::Regex::new(r"^(.+?)(?:::benches)?$").unwrap()
    });
    
    static REGEX_NAMESPACE_INTEGRATION: LazyLock<regex::Regex> = LazyLock::new(|| {
        regex::Regex::new(r"^(.+?)(?:::tests)?$").unwrap()
    });

    static REGEX_NAMESPACE_UNIT: LazyLock<regex::Regex> = LazyLock::new(|| {
        regex::Regex::new(r"^\w+::(.+?)(?:::tests)?$").unwrap()
    });

    // Group doesn't use a module_path!() / function_name!()
    let path = if raw.kind == TestingKind::Group {
        &raw.path
    } else {
        let captures = match raw.use_case {
            UseCase::Integration | UseCase::System | UseCase::Example
                => REGEX_NAMESPACE_INTEGRATION.captures(&raw.path),
            UseCase::Unit => REGEX_NAMESPACE_UNIT.captures(&raw.path),
            UseCase::Benchmark => REGEX_NAMESPACE_BENCHMARK.captures(&raw.path),
        };

        match captures {
            Some(captures) => captures.get(1).unwrap().as_str(),
            None => ""
        }
    };

    let full_path = format!("{package}/{use_case}/{path}{snake_name}",
        package = &raw.package_name,
        use_case = &raw.use_case,
        path = &path
            .replace("::", "/")
            .replace("-", "_"),
        snake_name = raw.name.as_ref()
            .map(|name| format!("/{}", name.replace("-", "_")))
            .unwrap_or_default()
    );

    Ok(PathBuf::from(full_path))
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    // NAMEPATH TESTING
    // See the namepaths.rs integration test for the master copy
    static TESTING: testing::Module = testing::module!(Unit);

    const GROUP_NAME: &'static str = "namepath_group/uno/dos";
    static GROUP: testing::Group = testing::group!(GROUP_NAME, Unit);

    #[named]
    #[test]
    fn test_unit_namepath() {
        const EXPECTED_PACKAGE_NAME: &'static str = "sourcetrait_testing";
        const EXPECTED_USE_CASE: testing::UseCase = testing::UseCase::Unit;

        const EXPECTED_MODULE_KIND: testing::TestingKind = testing::TestingKind::Module;
        const EXPECTED_MODULE_FULL_PATH: &'static str = "sourcetrait_testing/unit/namepath";
        const EXPECTED_MODULE_PATH: &'static str = "unit/namepath";
        const EXPECTED_MODULE_NAME: &'static str = "namepath";
        const EXPECTED_MODULE_RAW: &'static str = "module;unit;sourcetrait_testing;sourcetrait_testing::namepath::tests";

        const EXPECTED_GROUP_KIND: testing::TestingKind = testing::TestingKind::Group;
        const EXPECTED_GROUP_FULL_PATH: &'static str = "sourcetrait_testing/unit/namepath_group/uno/dos";
        const EXPECTED_GROUP_PATH: &'static str = "unit/namepath_group/uno/dos";
        const EXPECTED_GROUP_NAME: &'static str = "dos";
        const EXPECTED_GROUP_RAW: &'static str = "group;unit;sourcetrait_testing;namepath_group/uno/dos";

        const EXPECTED_TEST_KIND: testing::TestingKind = testing::TestingKind::Test;
        const EXPECTED_TEST_FULL_PATH: &'static str = "sourcetrait_testing/unit/namepath/test_unit_namepath";
        const EXPECTED_TEST_PATH: &'static str = "unit/namepath/test_unit_namepath";
        const EXPECTED_TEST_NAME: &'static str = "test_unit_namepath";
        const EXPECTED_TEST_RAW: &'static str = "test;unit;sourcetrait_testing;sourcetrait_testing::namepath::tests;test_unit_namepath";

        // Module
        assert_eq!(EXPECTED_PACKAGE_NAME, TESTING.namepath().package_name());
        assert_eq!(EXPECTED_MODULE_KIND, TESTING.namepath().kind());
        assert_eq!(EXPECTED_USE_CASE, TESTING.namepath().use_case());
        assert_eq!(EXPECTED_MODULE_FULL_PATH, TESTING.namepath().full_path().to_string_lossy());
        assert_eq!(EXPECTED_MODULE_PATH, TESTING.namepath().path().to_string_lossy());
        assert_eq!(EXPECTED_MODULE_NAME, TESTING.namepath().name());
        assert_eq!(EXPECTED_MODULE_RAW, TESTING.namepath().raw().to_string());

        // Group
        assert_eq!(EXPECTED_PACKAGE_NAME, GROUP.namepath().package_name());
        assert_eq!(EXPECTED_GROUP_KIND, GROUP.namepath().kind());
        assert_eq!(EXPECTED_USE_CASE, GROUP.namepath().use_case());
        assert_eq!(EXPECTED_GROUP_FULL_PATH, GROUP.namepath().full_path().to_string_lossy());
        assert_eq!(EXPECTED_GROUP_PATH, GROUP.namepath().path().to_string_lossy());
        assert_eq!(EXPECTED_GROUP_NAME, GROUP.namepath().name());
        assert_eq!(EXPECTED_GROUP_RAW, GROUP.namepath().raw().to_string());

        // Test
        let test = testing::test!();
        assert_eq!(EXPECTED_PACKAGE_NAME, test.namepath().package_name());
        assert_eq!(EXPECTED_TEST_KIND, test.namepath().kind());
        assert_eq!(EXPECTED_USE_CASE, test.namepath().use_case());
        assert_eq!(EXPECTED_TEST_FULL_PATH, test.namepath().full_path().to_string_lossy());
        assert_eq!(EXPECTED_TEST_PATH, test.namepath().path().to_string_lossy());
        assert_eq!(EXPECTED_TEST_NAME, test.namepath().name());
        assert_eq!(EXPECTED_TEST_RAW, test.namepath().raw().to_string());
    }
}
