use std::{path::PathBuf, sync::LazyLock};
use sourcetrait_testing::{StepState, Stepper, StepperBuilder};
use crate::{self as gitparity, prelude::*};

pub struct UpstreamRepoStepperOptions {
    pub temp_dir: PathBuf,
    pub git_kind: gitparity::GitKind,
}

pub struct UpstreamRepoStepState {
    upstream_repo_dir: PathBuf,
}

pub static UPSTREAM_REPO_ALPHA_STEPPER: LazyLock<StepperBuilder<UpstreamRepoStepperOptions, UpstreamRepoStepState, GitParity>> = LazyLock::new(|| {
    Stepper::<UpstreamRepoStepperOptions, UpstreamRepoStepState, GitParity>::builder("upstream_repo_alpha")
        .init(|_test, o| {
            let state = UpstreamRepoStepState {
                upstream_repo_dir: o.temp_dir
                    .join(format!("upstream-repo-alpha-{}", o.git_kind)),
            };
            
            let git = GitParity::init_bare(
                o.git_kind,
                state.upstream_repo_dir.clone(),
                gitparity::MAIN,
            ).unwrap();
            
            assert_eq!(state.upstream_repo_dir, git.top_dir());
            assert_eq!(gitparity::MAIN, git.branch_current().unwrap());
            
            StepState(state, git) 
        })
        .finalize()
});
