use std::io::Write;
use sourcetrait_testing::prelude::*;
use sourcetrait_gitparity::{
    prelude::*,
    testlib::stepper::repo_alpha::{downstream::*, upstream::*}
};

static TESTING: testing::Module = testing::module!(Integration, {
    .using_fixture_dir()
    .using_temp_dir()
    .setup(|_| {
        env_logger::Builder::new()
            .format(|buf, record| {
                writeln!(buf, "{}: {}", record.level(), record.args())
            })
            .parse_default_env()
            .init();
            
        println!("env_logger is available with: RUST_LOG=debug");
    })
    .skip_temp_dir_teardown(true)
});

#[tested]
fn test_stepper_alpha_gitc() {
    let test = testing::test!({
        .using_temp_dir()
    });
    
    test_stepper_alpha(&test, GitKind::GitC, GitKind::GitC);
}

#[tested]
fn test_stepper_alpha_cli() {
    let test = testing::test!({
        .using_temp_dir()
    });
    
    test_stepper_alpha(&test, GitKind::Cli, GitKind::Cli);
}

#[ignore]
#[tested]
fn test_stepper_alpha_gitc_to_cli() {
    let test = testing::test!({
        .using_temp_dir()
    });
    
    test_stepper_alpha(&test, GitKind::GitC, GitKind::Cli);
}

#[ignore]
#[tested]
fn test_stepper_alpha_cli_to_gitc() {
    let test = testing::test!({
        .using_temp_dir()
    });
    
    test_stepper_alpha(&test, GitKind::Cli, GitKind::GitC);
}
    
fn test_stepper_alpha(test: &testing::Test, git_kind_up: GitKind, git_kind_down: GitKind) {
    let upstream_git = UPSTREAM_REPO_ALPHA_STEPPER.run(test.as_testable(), UpstreamRepoStepperOptions {
        temp_dir: test.temp_dir().into(),
        git_kind: git_kind_up,
    });
    
    DOWNSTREAM_REPO_ALPHA_STEPPER.run(test.as_testable(), DownstreamRepoStepperOptions {
        temp_dir: test.temp_dir().into(),
        git_kind: git_kind_down,
        upstream_repo_dir: upstream_git.top_dir().to_path_buf(),
    });
}
