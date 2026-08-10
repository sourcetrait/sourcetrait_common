use std::{collections::HashMap, fs, path::PathBuf, sync::{Arc, LazyLock}};
use sourcetrait_testing::{StepState, Stepper, StepperBuilder};
use sourcetrait_stdx as stdx;
use crate::{self as gitparity, assert_expected_commits, prelude::*, Commit, GitOID, LogOptions, Resolution, HEAD, PATHSPEC_ALL};
#[allow(unused_imports)]
use crate:: dbg_commit_log;
use crate::testlib::util::*;

pub struct DownstreamRepoStepperOptions {
    pub temp_dir: PathBuf,
    pub git_kind: gitparity::GitKind,
    pub upstream_repo_dir: PathBuf,
}

pub struct RepoDownstreamStepState {
    git_kind: gitparity::GitKind,
    downstream_repo_dir: PathBuf,
    upstream_repo_dir: PathBuf,
    git_clone: Option<GitParity>,
}

impl RepoDownstreamStepState {
    fn git_cloned(&self) -> &GitParity {
        self.git_clone.as_ref().expect("git_clone to exist")
    }
    
    fn _git_cloned_mut(&mut self) -> &mut GitParity {
        self.git_clone.as_mut().expect("git_clone to exist")
    }
}

pub static DOWNSTREAM_REPO_ALPHA_STEPPER: LazyLock<StepperBuilder<DownstreamRepoStepperOptions, RepoDownstreamStepState, GitParity>> = LazyLock::new(|| {
    Stepper::<DownstreamRepoStepperOptions, RepoDownstreamStepState, GitParity>::builder("downstream_repo_alpha")
        .init(|_test, o| {
            let downstream_repo_dir = o.temp_dir
                .join(format!("downstream-repo-alpha-{}", o.git_kind));

            let git_env = build_git_env(&USER_TESTER_1, *TIME_INIT);
            let git = GitParity::clone_with(
                o.git_kind,
                o.upstream_repo_dir.to_str().unwrap(),
                downstream_repo_dir.clone(),
                gitparity::CloneOptions { env: Some(git_env) },
            ).unwrap();
            
            assert_eq!(downstream_repo_dir, git.top_dir());
            assert_eq!(gitparity::MAIN, git.branch_current().unwrap());
            assert!(git.status().unwrap().is_unmodified());
            
            let state = RepoDownstreamStepState {
                git_kind: o.git_kind,
                downstream_repo_dir,
                upstream_repo_dir: o.upstream_repo_dir,
                git_clone: None,
            };
            
            
            StepState(state, git)
        })
        .step("create_file_txt", |_test, state, git| {
            let file_txt_path = state.downstream_repo_dir.join("file.txt");
            stdx::fs::touch_file(&file_txt_path, Some(*TIME_INIT), Some("hello, git")).unwrap();
            
            // check the status, expect it to be untracked
            let expected_changes = HashMap::from([
                (Arc::new(PathBuf::from("file.txt")), gitparity::PathStatus::new(
                    PathBuf::from("file.txt"),
                    Some(gitparity::StatusCode::Untracked),
                    Some(gitparity::StatusCode::Untracked),
                    None
                )),
            ]);
            
            let actual_changes = git.status().unwrap().into_changes();
            assert_eq!(expected_changes, actual_changes);
            
            git.add("file.txt").unwrap();
            
            StepState(state, git)
        })
        .step("add_file_txt", |_test, state, git| {
            git.add("file.txt").unwrap();
            
            // check the status, expect it to be added to the index
            let expected_changes = HashMap::from([
                (Arc::new(PathBuf::from("file.txt")), gitparity::PathStatus::new(
                    PathBuf::from("file.txt"),
                    Some(gitparity::StatusCode::Added),
                    None,
                    None
                )),
            ]);
            let actual_changes = git.status().unwrap().into_changes();
            
            assert_eq!(expected_changes, actual_changes);
            
            StepState(state, git)
        })
        .step("commit_file_txt", |_test, state, git| {
            git.commit(COMMIT_INIT.message.as_ref().expect("exists")).unwrap();
            
            // check the log
            let expected_commits = Vec::from([COMMIT_INIT.clone()]);
            
            let actual_log = git.log_with(gitparity::LogOptions {
                show_message: true,
                show_signature_fingerprint: false, 
            }).unwrap();
            
            assert_eq!(&expected_commits, actual_log.commits());
            
            StepState(state, git)
        })
        .step("push_file_txt_commit", |_test, state, git| {
            git.push().unwrap();
            StepState(state, git)
        })
        .step("pull_main_after_init", |_test, state, git| {
            git.pull().unwrap();
            StepState(state, git)
        })
        .step("clone", |_test, mut state, git| {
            let clone_repo_dir = git.top_dir().parent().expect("topdir parent")
                .join(format!("downstream-repo-alpha-clone-{}", state.git_kind));
            
            let git_clone_env = build_git_env(&USER_TESTER_2, *TIME_STEP_2);
            let git_clone = GitParity::clone_with(
                state.git_kind,
                state.upstream_repo_dir.to_str().unwrap(),
                clone_repo_dir.clone(),
                gitparity::CloneOptions { env: Some(git_clone_env) },
            ).unwrap();
            
            assert_eq!(clone_repo_dir, git_clone.top_dir());
            assert_eq!(gitparity::MAIN, git_clone.branch_current().unwrap());
            assert!(git.status().unwrap().is_unmodified());
            
            state.git_clone = Some(git_clone);
            
            StepState(state, git)
        })
        .step("modify_file_txt", |_test, state, git| {
            let file_txt_path = state.downstream_repo_dir.join("file.txt");
            fs::write(&file_txt_path, "hello, GIT").unwrap();
            stdx::fs::touch_file(&file_txt_path, Some(*TIME_STEP_1), None).unwrap();
          
            // check the status, expect file.txt to be modified in the working tree
            let expected_changes = HashMap::from([
                (Arc::new(PathBuf::from("file.txt")), gitparity::PathStatus::new(
                    PathBuf::from("file.txt"),
                    None,
                    Some(gitparity::StatusCode::Modified),
                    None
                ))
            ]);
        
            let actual_changes = git.status().unwrap().into_changes();
            assert_eq!(expected_changes, actual_changes);
            
            StepState(state, git)
        })
        .step("add_modified_file_txt", |_test, state, git| {
            // add the file.txt to the index
            git.add(PATHSPEC_ALL).unwrap();
            
            // check the status, expect file.txt to be modified in the index
            let expected_changes = HashMap::from([
                (Arc::new(PathBuf::from("file.txt")), gitparity::PathStatus::new(
                    PathBuf::from("file.txt"),
                    Some(gitparity::StatusCode::Modified),
                    None,
                    None
                )),
            ]);
        
            let actual_changes = git.status().unwrap().into_changes();
            assert_eq!(expected_changes, actual_changes);
            
            StepState(state, git)
        })
        .step("commit_push_modified_file_txt", |_test, state, mut git| {
            let git_env = git.git_env_mut();
            git_env.set_author_date(*TIME_STEP_1);
            git_env.set_committer_date(*TIME_STEP_1);
            
            git.commit(COMMIT_MAIN_MODIFY_FILETXT.message.as_ref().expect("exists")).unwrap();
            git.push().unwrap();
        
            // check diff status between 
            let expected_changes = HashMap::from([
                (Arc::new(PathBuf::from("file.txt")), gitparity::PathDiffStatus::new(
                    PathBuf::from("file.txt"),
                    gitparity::StatusCode::Modified,
                    None
                )),
            ]);
        
            let actual_changes = git.diff_revision_statuses(HEAD, "HEAD~1")
                .unwrap()
                .into_changes();
            assert_eq!(expected_changes, actual_changes);
            
            let expected_commits = Vec::from([
                COMMIT_MAIN_MODIFY_FILETXT.clone(),
                COMMIT_INIT.clone(),
            ]);
            
            let actual_log = git.log_with(gitparity::LogOptions {
                show_message: true,
                show_signature_fingerprint: false, 
            }).unwrap();
            assert_eq!(&expected_commits, actual_log.commits());
            
            StepState(state, git)
        })
        .step("cloned_pull_modified_filetxt", |_test, state, git| {
            state.git_cloned().pull_forward().unwrap();
            
            // commits should match 
            let expected_commits = Vec::from([
                COMMIT_MAIN_MODIFY_FILETXT.clone(),
                COMMIT_INIT.clone()
            ]);
            
            let actual_log = state.git_cloned().log_with(gitparity::LogOptions {
                show_message: true,
                show_signature_fingerprint: false, 
            }).unwrap();
            
            assert_eq!(&expected_commits, actual_log.commits());
            
            StepState(state, git)
        })
        .step("branch_to_next", |_test, state, git| {
            git.branch_create(NEXT).unwrap();
            StepState(state, git)
        })
        .step("switch_to_next", |_test, state, git| {
            git.switch_branch(NEXT).unwrap();
            assert_eq!(git.branch_current().unwrap(), NEXT);
            
            // check the status, expect it to be clean
            let expected_changes = HashMap::new();
            let actual_changes = git.status().unwrap().into_changes();
            assert_eq!(expected_changes, actual_changes);
            
            StepState(state, git)
        })
        .step("next_add_nexttxt", |_test, state, git| {
            let file_next_txt_path = state.downstream_repo_dir.join("file-next.txt");
            stdx::fs::touch_file(&file_next_txt_path, Some(*TIME_INIT), Some("hello, next")).unwrap();
            
            git.add("file-next.txt").unwrap();
            StepState(state, git)
        })
        .step("next_commit_init_nexttxt", |_test, state, git| {
            git.commit("init_next").unwrap();
            
            let expected_commits = Vec::from([
                COMMIT_NEXT_INIT.clone(),
                COMMIT_MAIN_MODIFY_FILETXT.clone(),
                COMMIT_INIT.clone()
            ]);
            
            let log = git.log_with(LogOptions { show_message: true, ..Default::default() }).unwrap();
            assert_eq!(&expected_commits, log.commits());
            
            StepState(state, git)
        })
        .step("next_push_nexttxt", |_test, state, git| {
            git.push_new().unwrap();
            StepState(state, git)
        })
        .step("next_cloned_pull_next_init", |_test, state, git| {
            state.git_cloned().fetch_all().unwrap();
            state.git_cloned().switch_branch(NEXT).unwrap();
            state.git_cloned().pull_forward().unwrap();
            
            let expected_commits = Vec::from([
                COMMIT_NEXT_INIT.clone(),
                COMMIT_MAIN_MODIFY_FILETXT.clone(),
                COMMIT_INIT.clone()
            ]);
            
            let log = git.log_with(LogOptions { show_message: true, ..Default::default() }).unwrap();
            assert_eq!(&expected_commits, log.commits());
            let cloned_log = state.git_cloned().log_with(LogOptions { show_message: true, ..Default::default() }).unwrap();
            assert_eq!(&expected_commits, cloned_log.commits());
            
            StepState(state, git)
        })
        .step("next_cloned_modify_nexttxt", |_test, state, git| {
            stdx::fs::touch_file(&state.git_cloned().top_dir().join("next.txt"),
                Some(*TIME_STEP_1), Some("hello, MODIFY 1".into())).unwrap();
            state.git_cloned().add_all().unwrap();
            state.git_cloned().commit("next_cloned_modify_nexttxt_1").unwrap();
            
            stdx::fs::touch_file(&state.git_cloned().top_dir().join("next.txt"),
                Some(*TIME_STEP_2), Some("hello, MODIFY 2".into())).unwrap();
            state.git_cloned().add_all().unwrap();
            state.git_cloned().commit("next_cloned_modify_nexttxt_2").unwrap();
            
            assert_expected_commits!(state.git_cloned(), [
                &COMMIT_CLONED_MODIFY_NEXTTXT_2,
                &COMMIT_CLONED_MODIFY_NEXTTXT_1,
                &COMMIT_NEXT_INIT,
                &COMMIT_MAIN_MODIFY_FILETXT,
                &COMMIT_INIT
            ]);
            
            state.git_cloned().push().unwrap();
            
            StepState(state, git)
        })
        .step("next_modify_filenext", |_test, state, git| {
            stdx::fs::touch_file(&git.top_dir().join("file-next.txt"),
                Some(*TIME_STEP_1), Some("hello, MODIFY 1".into())).unwrap();
            git.add_all().unwrap();
            git.commit("next_modify_filenext_1").unwrap();
            
            stdx::fs::touch_file(&git.top_dir().join("file-next.txt"),
                Some(*TIME_STEP_2), Some("hello, MODIFY 2".into())).unwrap();
            git.add_all().unwrap();
            git.commit("next_modify_filenext_2").unwrap();
            
            assert_expected_commits!(git, [
                &COMMIT_NEXT_PREBASE_MODIFY_FILENEXT_2,
                &COMMIT_NEXT_PREBASE_MODIFY_FILENEXT_1,
                &COMMIT_NEXT_INIT,
                &COMMIT_MAIN_MODIFY_FILETXT,
                &COMMIT_INIT
            ]);
            
            // now attempt to rebase the cloned's two commits under ours
            git.pull_rebase().unwrap();
            
            assert_expected_commits!(git, [
                &COMMIT_NEXT_MODIFY_FILENEXT_2,
                &COMMIT_NEXT_MODIFY_FILENEXT_1,
                &COMMIT_CLONED_MODIFY_NEXTTXT_2,
                &COMMIT_CLONED_MODIFY_NEXTTXT_1,
                &COMMIT_NEXT_INIT,
                &COMMIT_MAIN_MODIFY_FILETXT,
                &COMMIT_INIT
            ]);
            
            git.push().unwrap();
            
            StepState(state, git)
        })
        .step("draft1_init", |_test, state, git| {
            git.branch_create("draft1").unwrap();
            git.switch_branch("draft1").unwrap();
            git.push_new().unwrap();            
            
            stdx::fs::touch_file(&git.top_dir().join("draft1.txt"),
                Some(*TIME_STEP_1), Some("draft1: init")
            ).unwrap();
            
            git.add_all().unwrap();
            git.commit("draft1_init").unwrap();
            git.push().unwrap();
            
            StepState(state, git)
        })
        .step("draft2_init", |_test, state, git| {
            let git_cloned = state.git_cloned();
            git_cloned.switch_branch("next").unwrap();
            git_cloned.pull_forward().unwrap();
            git_cloned.branch_create("draft2").unwrap();
            git_cloned.switch_branch("draft2").unwrap();
            git_cloned.push_new().unwrap();            
            
            stdx::fs::touch_file(&git_cloned.top_dir().join("draft2.txt"),
                Some(*TIME_STEP_1), Some("draft2: init")
            ).unwrap();
            
            git_cloned.add_all().unwrap();
            git_cloned.commit("draft2_init").unwrap();
            git_cloned.push().unwrap();
            
            StepState(state, git)
        })
        .step("next_merge_drafts", |_test, state, git| {
            git.fetch_all().unwrap();
            
            git.switch_branch("draft2").unwrap();
            git.pull_forward().unwrap();
            
            git.switch_branch("draft1").unwrap();
            git.pull_forward().unwrap();
            
            git.switch_branch(NEXT).unwrap();
            git.pull_forward().unwrap();
            
            let actual = git.merge_forward("draft1").unwrap();
            assert_eq!(Resolution::FastForwarded, actual);
            
            assert!(git.merge_forward("draft2").is_err());
            let actual = git.merge_auto("draft2").unwrap();
            assert_eq!(Resolution::AutoResolved, actual);
            git.commit("next_merge_draft2").unwrap();
            
            StepState(state, git)
        })
        .step("next_merge_manual_drafts", |_test, state, git| {
            git.switch_branch("draft1").unwrap();
            let actual = git.merge_forward(NEXT).unwrap();
            assert_eq!(Resolution::FastForwarded, actual);
            
            stdx::fs::touch_file(&git.top_dir().join("draft2.txt"),
                Some(*TIME_STEP_2), Some("this is will conflict")
            ).unwrap();
            
            git.add_all().unwrap();
            git.commit("draft1_conflict").unwrap();
            
            git.switch_branch("draft2").unwrap();
            let actual = git.merge_forward(NEXT).unwrap();
            assert_eq!(Resolution::FastForwarded, actual);

            stdx::fs::touch_file(&git.top_dir().join("draft2.txt"),
                Some(*TIME_STEP_2), Some("THIS WILL CONFLICT")
            ).unwrap();
            
            git.add_all().unwrap();
            git.commit("draft2_conflict").unwrap();
            
            git.switch_branch(NEXT).unwrap();
            let actual = git.merge_forward("draft1").unwrap();
            assert_eq!(Resolution::FastForwarded, actual);
            
            assert!(git.merge_forward("draft2").is_err());
            assert!(git.merge_auto("draft2").is_err());
            
            let actual = git.merge("draft2").unwrap();
            assert!(matches!(actual, Resolution::Unresolved(_)));
            
            stdx::fs::touch_file(&git.top_dir().join("draft2.txt"),
                Some(*TIME_STEP_3), Some("conflict resolved")).unwrap();
            
            git.add("draft2.txt").unwrap();
            git.commit("draft2_resolved").unwrap();
            git.push().unwrap();
            
            StepState(state, git)
        })
        .step("next_rebase", |_test, state, git| {
            StepState(state, git)
        })
        .finalize()
});

static COMMIT_INIT: LazyLock<gitparity::Commit> = LazyLock::new(|| gitparity::Commit {
    commit_oid: gitparity::GitOID::from_str("84cb5e344c42ae0c2288c8468eb1f565e60850e8").unwrap(),
    tree_oid: gitparity::GitOID::from_str("5e8cf560ce7c0a620a4abd9c0b16a6a7fe6d2ef7").unwrap(),
    parent_oid: vec![],
    author: Arc::clone(&USER_TESTER_1),
    author_time: *TIME_INIT,
    committer: Arc::clone(&USER_TESTER_1),
    committer_time: *TIME_INIT,
    signature_fingerprint: None,
    message: Some("init".to_string()),
});

static COMMIT_MAIN_MODIFY_FILETXT: LazyLock<gitparity::Commit> = LazyLock::new(|| gitparity::Commit {
    commit_oid: gitparity::GitOID::from_str("e37f83afe3b9dc91c038538c1ce5b33b94c8d52e").unwrap(),
    tree_oid: gitparity::GitOID::from_str("12908bc394fd55ae1a116340016f3cde9d675840").unwrap(),
    parent_oid: vec![
        COMMIT_INIT.commit_oid.clone(),
    ],
    author: Arc::clone(&USER_TESTER_1),
    author_time: *TIME_STEP_1,
    committer: Arc::clone(&USER_TESTER_1),
    committer_time: *TIME_STEP_1,
    signature_fingerprint: None,
    message: Some("main_modify_filetxt".to_string()),
});

static COMMIT_NEXT_INIT: LazyLock<gitparity::Commit> = LazyLock::new(|| Commit {
    commit_oid: GitOID::from_str("e08ddbf42b0d0ca361c9c712d3003b703e86d027").unwrap(),
    tree_oid: GitOID::from_str("c7f9c2c2828ee1e40ba53adac63dc7e7d830e260").unwrap(),
    parent_oid: vec![
        COMMIT_MAIN_MODIFY_FILETXT.commit_oid.clone(),
    ],
    author: Arc::clone(&USER_TESTER_1),
    author_time: *TIME_STEP_1,
    committer: Arc::clone(&USER_TESTER_1),
    committer_time: *TIME_STEP_1,
    signature_fingerprint: None,
    message: Some("init_next".into()),
});

static COMMIT_CLONED_MODIFY_NEXTTXT_1: LazyLock<gitparity::Commit> = LazyLock::new(|| gitparity::Commit {
    commit_oid: gitparity::GitOID::from_str("5eda8f0c09ed3cb7086ed2a75897c8e87088b880").unwrap(),
    tree_oid: gitparity::GitOID::from_str("334e8934a6bfa444240136a050edd9c98fa7cdb3").unwrap(),
    parent_oid: vec![
        COMMIT_NEXT_INIT.commit_oid().clone(),
    ],
    author: Arc::clone(&USER_TESTER_2),
    author_time: *TIME_STEP_2,
    committer: Arc::clone(&USER_TESTER_2),
    committer_time: *TIME_STEP_2,
    signature_fingerprint: None,
    message: Some("next_cloned_modify_nexttxt_1".into()),
});

static COMMIT_CLONED_MODIFY_NEXTTXT_2: LazyLock<gitparity::Commit> = LazyLock::new(|| gitparity::Commit {
    commit_oid: gitparity::GitOID::from_str("ffae42a1ba323fc05dd95619648d0535c9096977").unwrap(),
    tree_oid: gitparity::GitOID::from_str("11699ceda6ca8b9923d074f99cea0aec96800cdb").unwrap(),
    parent_oid: vec![
        COMMIT_CLONED_MODIFY_NEXTTXT_1.commit_oid().clone(),
    ],
    author: Arc::clone(&USER_TESTER_2),
    author_time: *TIME_STEP_2,
    committer: Arc::clone(&USER_TESTER_2),
    committer_time: *TIME_STEP_2,
    signature_fingerprint: None,
    message: Some("next_cloned_modify_nexttxt_2".into()),
});

static COMMIT_NEXT_PREBASE_MODIFY_FILENEXT_1: LazyLock<gitparity::Commit> = LazyLock::new(|| gitparity::Commit {
    commit_oid: gitparity::GitOID::from_str("03d3273254c437f3fd688d6ba26a5b80458c1267").unwrap(),
    tree_oid: gitparity::GitOID::from_str("9b93f16cff46cc2063a527f576efff8dfb125feb").unwrap(),
    parent_oid: vec![
        COMMIT_NEXT_INIT.commit_oid().clone(),
    ],
    author: Arc::clone(&USER_TESTER_1),
    author_time: *TIME_STEP_1,
    committer: Arc::clone(&USER_TESTER_1),
    committer_time: *TIME_STEP_1,
    signature_fingerprint: None,
    message: Some("next_modify_filenext_1".into()),
});

static COMMIT_NEXT_PREBASE_MODIFY_FILENEXT_2: LazyLock<gitparity::Commit> = LazyLock::new(|| gitparity::Commit {
    commit_oid: gitparity::GitOID::from_str("3ed7c1397840d1f60c052625a9bc6e7a17e27841").unwrap(),
    tree_oid: gitparity::GitOID::from_str("1b9988a0e854f2c34b43cdf0acb11812b62d8d87").unwrap(),
    parent_oid: vec![
        COMMIT_NEXT_PREBASE_MODIFY_FILENEXT_1.commit_oid().clone(),
    ],
    author: Arc::clone(&USER_TESTER_1),
    author_time: *TIME_STEP_1,
    committer: Arc::clone(&USER_TESTER_1),
    committer_time: *TIME_STEP_1,
    signature_fingerprint: None,
    message: Some("next_modify_filenext_2".into()),
});

static COMMIT_NEXT_MODIFY_FILENEXT_1: LazyLock<gitparity::Commit> = LazyLock::new(|| gitparity::Commit {
    commit_oid: gitparity::GitOID::from_str("fd6aa64f2b3d809fb6e9700d4faaf9c08e8dc210").unwrap(),
    tree_oid: gitparity::GitOID::from_str("5aa3c06cbb8ef562def0979dc63c60f857031c3e").unwrap(),
    parent_oid: vec![
        COMMIT_CLONED_MODIFY_NEXTTXT_2.commit_oid().clone(),
    ],
    author: Arc::clone(&USER_TESTER_1),
    author_time: *TIME_STEP_1,
    committer: Arc::clone(&USER_TESTER_1),
    committer_time: *TIME_STEP_1,
    signature_fingerprint: None,
    message: Some("next_modify_filenext_1".into()),
});

static COMMIT_NEXT_MODIFY_FILENEXT_2: LazyLock<gitparity::Commit> = LazyLock::new(|| gitparity::Commit {
    commit_oid: gitparity::GitOID::from_str("38b1d629287cab8dca127e124d27568c9588891c").unwrap(),
    tree_oid: gitparity::GitOID::from_str("ff0515b546c5e604074f1123f0d6a27ea9793848").unwrap(),
    parent_oid: vec![
        COMMIT_NEXT_MODIFY_FILENEXT_1.commit_oid().clone(),
    ],
    author: Arc::clone(&USER_TESTER_1),
    author_time: *TIME_STEP_1,
    committer: Arc::clone(&USER_TESTER_1),
    committer_time: *TIME_STEP_1,
    signature_fingerprint: None,
    message: Some("next_modify_filenext_2".into()),
});