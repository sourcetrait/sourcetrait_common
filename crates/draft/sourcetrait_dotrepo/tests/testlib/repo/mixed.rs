use strum;
use sourcetrait_dotrepo::{self as dotrepo, *}; 
use sourcetrait_testing::prelude::*;

pub static GROUP_REPO_MIXED: testing::Group = testing::group!("repos/mixed", Integration, {
    .using_fixture_dir()
});

pub fn dotrepo_mixed() -> DotRepo<MixedDotRepo> {
    DotRepoDir::new(GROUP_REPO_MIXED.fixture_dir().to_path_buf())
        .tenant(&DEF_MIXED)
        .unwrap()
}

pub(crate) const MYORG_MYSYS: &'static str = "myorg/mysys";

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, strum::Display, strum::AsRefStr, strum::EnumCount, strum::EnumIter)]
#[strum(serialize_all = "kebab-case")]
pub enum TestDesignatorKind {
    Movies,
    Music,
    Documents
}

impl DesignatorKind for TestDesignatorKind {}

impl DesignatorKindTraits<TestDesignator> for TestDesignatorKind {
    fn from_designator(value: &TestDesignator) -> Self {
        match value {
            TestDesignator::Movies => Self::Movies,
            TestDesignator::Music => Self::Music,
            TestDesignator::Documents(_) => Self::Documents,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, strum::Display, strum::AsRefStr)]
#[strum(serialize_all = "kebab-case")]
pub enum TestDesignator {
    Movies,
    Music,
    Documents(String)
}

impl Designator for TestDesignator {
    fn identifier(&self) -> Option<&str> {
        match self {
            Self::Movies => None,
            Self::Music => None,
            Self::Documents(s) => Some(&s)
        }
    }
}

impl DesignatorTraits<TestDesignatorKind> for TestDesignator {
    fn try_from_tuple<R: 'static + DotRepoType>(tuple: DesignatorTuple<TestDesignatorKind>) -> dotrepo::RepoResult<R, Self> {
        let DesignatorTuple(kind, name) = tuple;
        match kind {
            TestDesignatorKind::Movies => Ok(Self::Movies),
            TestDesignatorKind::Music => Ok(Self::Music),
            TestDesignatorKind::Documents => Ok(Self::Documents(name.unwrap())),
        }
    }
}

pub(crate) const DEF_MIXED: Definition<TestDesignatorKind, TestDesignator> = Definition::builder()
    .relative_path(MYORG_MYSYS)
    .name("Mixed")
    .version(1,1,1)
    .top_definition(DesignatedDefinition::builder()
        .using_excludes()
    )
    .tenant_definitions(&[
        (TestDesignatorKind::Music, DesignatedDefinition::builder()
            .using_excludes()
            .build()
        ),
        (TestDesignatorKind::Movies, DesignatedDefinition::builder()
            .using_excludes()
            .build()
        ),
        (TestDesignatorKind::Documents, DesignatedDefinition::builder()
            .using_excludes()
            .build()
        )
        
    ])
    .build();

pub(crate) const DEF_FULL: Definition<TestDesignatorKind, TestDesignator> = Definition::builder()
    .name("Full")
    .relative_path(MYORG_MYSYS)
    .version(1,1,1)
    .top_definition(DesignatedDefinition::builder()
        .using_excludes()
        .using_local()
        .using_state()
    )
    .module_definition(DesignatedDefinition::builder()
        .using_excludes()
        .using_local()
        .using_state()
    )
    .tenant_definitions(&[
        (TestDesignatorKind::Music, DesignatedDefinition::builder()
            .using_excludes()
            .using_local()
            .using_state()
            .build()
        ),
        (TestDesignatorKind::Documents, DesignatedDefinition::builder()
            .using_excludes()
            .using_local()
            .using_state()
            .build()
        ),
        (TestDesignatorKind::Movies, DesignatedDefinition::builder()
            .using_excludes()
            .using_local()
            .using_state()
            .build()
        ),
        
    ])
    .build();

#[derive(Debug, Clone, PartialEq)]
pub struct MixedDotRepo;
impl DotRepoType for MixedDotRepo {
    type DesignatorKind = TestDesignatorKind;
    type Designator = TestDesignator;
    const DEFINITION: Definition<Self::DesignatorKind, Self::Designator> = DEF_MIXED;
}

pub(crate) const EXPECTED_FILES_MIXED: [&'static str; 5] = [
    "audio/song.mp3",
    "include.txt",
    "stuff/a.incl",
    "stuff/docs/important/Important.txt",
    "video/movie.avi",
];

pub(crate) const EXPECTED_DIRS_MIXED: [&'static str; 5] = [
    "audio",
    "stuff",
    "stuff/docs",
    "stuff/docs/important",
    "video",
];

pub(crate) const EXPECTED_PATHS_MIXED: [&'static str; 10] = [
    "audio",
    "audio/song.mp3",
    "include.txt",
    "stuff",
    "stuff/a.incl",
    "stuff/docs",
    "stuff/docs/important",
    "stuff/docs/important/Important.txt",
    "video",
    "video/movie.avi",
];

