mod testlib;

use std::{collections::HashSet, fs, path::Path, sync::LazyLock};
use semver::Version as SemVer;
use sourcetrait_testing::prelude::*;
use sourcetrait_dotrepo::*;
use testlib::{repo::mixed::*, utils::*};

static TESTING: testing::Module = testing::module!(Integration, {
    .using_temp_dir()
    //.skip_temp_dir_teardown(true)
});
    
#[tested]
fn test_construction() {
    let _test = testing::test!();
    
    // construction: DotRepo::new()
    let dotrepo_dir = DotRepoDir::new(GROUP_REPO_MIXED.fixture_dir().to_path_buf());
    let _repo = DotRepo::<MixedDotRepo>::new(dotrepo_dir, &DEF_MIXED).unwrap();
    
    // construction: DotRepoType::new()
    let dotrepo_dir = DotRepoDir::new(GROUP_REPO_MIXED.fixture_dir().to_path_buf());
    let repo = MixedDotRepo::new(dotrepo_dir).unwrap();
    
    assert_eq!(SemVer::new(1,1,1), repo.read_version().unwrap());
    
    // construction: DotRepoDir::tenant()
    let repo = DotRepoDir::new(GROUP_REPO_MIXED.fixture_dir().to_path_buf())
        .tenant::<MixedDotRepo>(&DEF_MIXED).unwrap();
    
    assert_eq!(SemVer::new(1,1,1), repo.read_version().unwrap());
}

#[tested]
fn test_ignores() {
    let _test = testing::test!();
    let repo = dotrepo_mixed();
    let actual_ignores = repo.read_excludes().unwrap();
    assert_eq!(2, actual_ignores.len());
    assert!(actual_ignores.matched(repo.dotrepo_dir().join("chicken.kfc"), false).is_ignore());
}

#[tested]
fn test_walk_current() {
    let _test = testing::test!();
    let repo = dotrepo_mixed();
    
    let actual_paths = repo.walk_current(WalkOff::default())
        .map(|entry| map_paths(entry, repo.current_dir())) 
        .collect::<Vec<_>>();
    
    assert_eq!(Vec::from(EXPECTED_PATHS_MIXED), actual_paths);
}

fn do_test_find_designated_top(repo: &DotRepo<MixedDotRepo>) {
    static EXPECTED_ANYKIND_2: LazyLock<Vec<(String, DesignatorMatches<MixedDotRepo>)>> = LazyLock::new(|| 
        Vec::from([
            ("audio".to_string(), DesignatorMatches::new_from(
                HashSet::new(),
                HashSet::new(),
                HashSet::from([TestDesignatorKind::Music]),
                HashSet::new(),
            )),
            ("video".to_string(), DesignatorMatches::new_from(
                HashSet::new(),
                HashSet::new(),
                HashSet::from([TestDesignatorKind::Movies]),
                HashSet::new(),
            )),
        ])
    );
        
    let actual_matches = repo.find_designated(TenantWalkOff::builder()
            .any_kind([
                TestDesignatorKind::Music,
                TestDesignatorKind::Movies
            ])
            .build().unwrap()
        )
        .map(|entry| map_designated(entry, repo.current_dir()))
        .collect::<Vec<_>>();
    
    // test find_designated() for any_kind with 2 out of 3 designators
    assert_eq!(*EXPECTED_ANYKIND_2, actual_matches);
    
    static EXPECTED_ALLKIND_MODULE_ANYKIND_2: LazyLock<Vec<(String, DesignatorMatches<MixedDotRepo>)>> = LazyLock::new(|| 
        Vec::from([
            ("audio".to_string(), DesignatorMatches::new_from(
                HashSet::from([StandardDesignatorKind::Module]),
                HashSet::new(),
                HashSet::from([TestDesignatorKind::Music]),
                HashSet::new(),
            )),
            ("video".to_string(), DesignatorMatches::new_from(
                HashSet::from([StandardDesignatorKind::Module]),
                HashSet::new(),
                HashSet::from([TestDesignatorKind::Movies]),
                HashSet::new(),
            )),
        ]
    ));
        
    let actual_matches = repo.find_designated(TenantWalkOff::builder()
            .all_standard_kind([StandardDesignatorKind::Module])
            .any_kind([
                TestDesignatorKind::Music,
                TestDesignatorKind::Movies
            ])
            .build().unwrap()
        )
        .map(|entry| map_designated(entry, repo.current_dir()))
        .collect::<Vec<_>>();
    
    // test find_designated() for all kind with std module, any kind with 2 out of 3 designators
    assert_eq!(*EXPECTED_ALLKIND_MODULE_ANYKIND_2, actual_matches);
    
    static EXPECTED_ALL_MODULE_ALLKIND_1: LazyLock<Vec<(String, DesignatorMatches<MixedDotRepo>)>> = LazyLock::new(|| 
        Vec::from([
            ("audio".to_string(), DesignatorMatches::new_from(
                HashSet::new(),
                HashSet::from([StandardDesignator::Module(Some("Audio".to_string()))]),
                HashSet::from([TestDesignatorKind::Music]),
                HashSet::new(),
            ))
        ]
    ));
    
    let actual_matches = repo.find_designated(TenantWalkOff::builder()
            .all_standard([StandardDesignator::Module(Some("Audio".to_string()))])
            .all_kind([TestDesignatorKind::Music])
            .build().unwrap()
        )
        .map(|entry| map_designated(entry, repo.current_dir()))
        .collect::<Vec<_>>();
    
    // test find_designated() for all std module, any kind with 1 out of 3 designators
    assert_eq!(*EXPECTED_ALL_MODULE_ALLKIND_1, actual_matches);
    
    static EXPECTED_ALL_1: LazyLock<Vec<(String, DesignatorMatches<MixedDotRepo>)>> = LazyLock::new(|| 
        Vec::from([
            ("stuff/docs/important".to_string(), DesignatorMatches::new_from(
                HashSet::new(),
                HashSet::new(),
                HashSet::new(),
                HashSet::from([TestDesignator::Documents("important".to_string())]),
            )),
        ]
    ));
    
    let actual_matches = repo.find_designated(TenantWalkOff::builder()
            .all([TestDesignator::Documents("important".to_string())])
            .build().unwrap()
        )
        .map(|entry| map_designated(entry, repo.current_dir()))
        .collect::<Vec<_>>();
    
    // test find_designated(): all with tenant designator
    assert_eq!(*EXPECTED_ALL_1, actual_matches);
}

#[tested]
fn test_find_designated() {
    let _test = testing::test!();
    let repo = dotrepo_mixed();
    do_test_find_designated_top(&repo);
}

fn do_test_read_designations_top(repo: &DotRepo<MixedDotRepo>) {
    static EXPECTED: LazyLock<HashSet<Designated<TestDesignator>>> = LazyLock::new(|| 
        HashSet::from([
            Designated::Standard(StandardDesignator::Top),
        ])
    );
    
    assert_eq!(*EXPECTED, repo.read_designations().unwrap());
}

#[tested]
fn test_read_designations() {
    let _test = testing::test!();
    let repo = dotrepo_mixed();
    do_test_read_designations_top(&repo);
}

#[tested]
fn test_init() {
    let test = testing::test!({
        .using_temp_dir()
    });
    
    let repo_top = DotRepo::init(
        DotRepoDir::new(test.temp_dir().to_path_buf()),
        &DEF_FULL,
        Some(HashSet::from([
            Designated::Standard(StandardDesignator::Top)]
        )),
    )
    .unwrap();
    
    let _dotrepo_audio = repo_top.create(
        Path::new("audio"),
        Some(HashSet::from([
            Designated::Standard(StandardDesignator::Module(Some("Audio".to_string()))),
            Designated::Tenant(TestDesignator::Music)
        ]))
    ).unwrap();
    
    assert!(test.temp_dir().join("audio").exists());
    
    let _dotrepo_docs = repo_top.create(
        Path::new("stuff/docs/important"),
        Some(HashSet::from([
            Designated::Tenant(TestDesignator::Documents("important".to_string())),
        ]))
    ).unwrap();
    
    assert!(test.temp_dir().join("stuff/docs/important").exists());
    
    let _dotrepo_video = repo_top.create(
        Path::new("video"),
        Some(HashSet::from([
            Designated::Standard(StandardDesignator::Module(Some("Video".to_string()))),
            Designated::Tenant(TestDesignator::Movies)
        ]))
    ).unwrap();
    
    assert!(test.temp_dir().join("video").exists());
    
    for dir_path in EXPECTED_DIRS_MIXED {
        let dir = test.temp_dir().join(dir_path);
        if !dir.is_dir() {
            fs::create_dir_all(dir).unwrap();
        }
    }
    
    for file_path in EXPECTED_FILES_MIXED {
        let file = test.temp_dir().join(file_path);
        if !file.is_file() {
            fs::write(file, "").unwrap();
        }
    }
    
    let actual_paths = repo_top.walk_current(WalkOff::default())
        .map(|entry| map_paths(entry, repo_top.current_dir()))
        .collect::<Vec<_>>();
    
    assert_eq!(Vec::from(EXPECTED_PATHS_MIXED), actual_paths);
    
    do_test_find_designated_top(&repo_top);
    do_test_read_designations_top(&repo_top);
}
