use std::fs;
use strum;
use sourcetrait_dotrepo::{self as dotrepo, *}; 
use sourcetrait_testing::prelude::*;

pub static GROUP_REPO_MIXED_GIT: testing::Group = testing::group!("repos/mixed-git", Integration, {
    .using_fixture_dir()
    .using_temp_dir()
    .skip_temp_dir_teardown(true)
    .setup(|this| {
        fs::copy(
                this.fixture_dir(),
                this.temp_dir()
            )
            .unwrap();
        //anygit::init(this.temp_dir()).unwrap();
    })
});


pub fn dotrepo_mixed_git() -> DotRepo<MixedGitDotRepo> {
    DotRepoDir::new(GROUP_REPO_MIXED_GIT.fixture_dir().to_path_buf())
        .tenant(&DEF_MIXED_GIT)
        .unwrap()
}

pub(crate) const MYORG_MYSYS: &'static str = "myorg/mysys";

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, strum::Display, strum::AsRefStr, strum::EnumCount, strum::EnumIter)]
#[strum(serialize_all = "kebab-case")]
pub enum MixedGitDesignatorKind {
    Documents,
    GitSubmodule,
    GitTop,
    Movies,
    Music,
}

impl DesignatorKind for MixedGitDesignatorKind {}

impl DesignatorKindTraits<MixedGitDesignator> for MixedGitDesignatorKind {
    fn from_designator(value: &MixedGitDesignator) -> Self {
        match value {
            MixedGitDesignator::GitTop => Self::GitTop,
            MixedGitDesignator::GitSubmodule(_) => Self::GitSubmodule,
            MixedGitDesignator::Movies => Self::Movies,
            MixedGitDesignator::Music => Self::Music,
            MixedGitDesignator::Documents(_) => Self::Documents,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, strum::Display, strum::AsRefStr)]
#[strum(serialize_all = "kebab-case")]
pub enum MixedGitDesignator {
    Documents(String),
    GitSubmodule(String),
    GitTop,
    Movies,
    Music,
}

impl Designator for MixedGitDesignator {
    fn identifier(&self) -> Option<&str> {
        match self {
            Self::Documents(s) => Some(&s),
            Self::GitSubmodule(s) => Some(&s),
            Self::GitTop => None,
            Self::Movies => None,
            Self::Music => None,
        }
    }
}

impl DesignatorTraits<MixedGitDesignatorKind> for MixedGitDesignator {
    fn try_from_tuple<R: 'static + DotRepoType>(tuple: DesignatorTuple<MixedGitDesignatorKind>) -> dotrepo::RepoResult<R, Self> {
        let DesignatorTuple(kind, name) = tuple;
        match kind {
            MixedGitDesignatorKind::Documents => Ok(Self::Documents(name.unwrap())),
            MixedGitDesignatorKind::GitTop => Ok(Self::GitTop),
            MixedGitDesignatorKind::GitSubmodule => Ok(Self::GitSubmodule(name.unwrap())),
            MixedGitDesignatorKind::Movies => Ok(Self::Movies),
            MixedGitDesignatorKind::Music => Ok(Self::Music),
        }
    }
}

pub(crate) const DEF_MIXED_GIT: Definition<MixedGitDesignatorKind, MixedGitDesignator> = Definition::builder()
    .name("MixedGit")
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
        (MixedGitDesignatorKind::Documents, DesignatedDefinition::builder()
            .using_excludes()
            .using_local()
            .using_state()
            .build()
        ),
        (MixedGitDesignatorKind::GitSubmodule, DesignatedDefinition::builder()
            .using_excludes()
            .using_local()
            .using_state()
            .build()
        ),
        (MixedGitDesignatorKind::GitTop, DesignatedDefinition::builder()
            .using_excludes()
            .using_local()
            .using_state()
            .build()
        ),
        (MixedGitDesignatorKind::Music, DesignatedDefinition::builder()
            .using_excludes()
            .using_local()
            .using_state()
            .build()
        ),
        (MixedGitDesignatorKind::Movies, DesignatedDefinition::builder()
            .using_excludes()
            .using_local()
            .using_state()
            .build()
        ),
    ])
    .build();

#[derive(Debug, Clone, PartialEq)]
pub struct MixedGitDotRepo;
impl DotRepoType for MixedGitDotRepo {
    type DesignatorKind = MixedGitDesignatorKind;
    type Designator = MixedGitDesignator;
    const DEFINITION: Definition<Self::DesignatorKind, Self::Designator> = DEF_MIXED_GIT;
}

pub(crate) const EXPECTED_FILES_MIXED_GIT: [&'static str; 5] = [
    "audio/song.mp3",
    "include.txt",
    "stuff/a.incl",
    "stuff/docs/important/Important.txt",
    "video/movie.avi",
];

pub(crate) const EXPECTED_DIRS_MIXED_GIT: [&'static str; 5] = [
    "audio",
    "stuff",
    "stuff/docs",
    "stuff/docs/important",
    "video",
];

pub(crate) const EXPECTED_PATHS_MIXED_GIT: [&'static str; 10] = [
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

