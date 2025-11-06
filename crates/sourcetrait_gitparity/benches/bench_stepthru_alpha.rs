#![cfg_attr(feature = "nightly", feature(test))]

#[cfg(feature = "nightly")]
mod benches {
    extern crate test;
    use std::sync::atomic::AtomicUsize;
    use sourcetrait_testing::prelude::*;
    use sourcetrait_gitparity::prelude::*;
    use sourcetrait_gitparity::testlib::stepper::repo_alpha::{
        upstream::{UPSTREAM_REPO_ALPHA_STEPPER, UpstreamRepoStepperOptions},
        downstream::{DownstreamRepoStepperOptions, DOWNSTREAM_REPO_ALPHA_STEPPER}
    };
    
    static TESTING: testing::Module = testing::module!(Benchmark, {
        .using_fixture_dir()
        .using_temp_dir()
    });
    
    fn bench_stepper_alpha(test: &testing::Test, git_kind_up: GitKind, git_kind_down: GitKind) {
        static BENCH_ITER: AtomicUsize = AtomicUsize::new(0);
        let bench_iter = BENCH_ITER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        
        let upstream_git = UPSTREAM_REPO_ALPHA_STEPPER.run(test.as_testable(), UpstreamRepoStepperOptions {
            temp_dir: test.temp_dir().join(format!("bench-iter-{bench_iter}")),
            git_kind: git_kind_up,
        });
        
        DOWNSTREAM_REPO_ALPHA_STEPPER.run(test.as_testable(), DownstreamRepoStepperOptions {
            temp_dir: test.temp_dir().join(format!("bench-iter-{bench_iter}")),
            git_kind: git_kind_down,
            upstream_repo_dir: upstream_git.top_dir().to_path_buf(),
        });
    }
    
    #[benched]
    fn bench_stepper_alpha_libc(b: &mut test::Bencher) {
        let test = testing::test!({
            .using_temp_dir()
        });
        
        b.iter(|| bench_stepper_alpha(&test, GitKind::LibC, GitKind::LibC));
    }
    
    #[benched]
    fn bench_stepper_alpha_cli(b: &mut test::Bencher) {
        let test = testing::test!({
            .using_temp_dir()
        });
        
        b.iter(|| bench_stepper_alpha(&test, GitKind::Cli, GitKind::Cli));
    }
}
