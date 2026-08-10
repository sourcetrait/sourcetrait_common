pub mod log;
pub mod options;
pub mod types;
pub mod status;

use std::path::{Path, PathBuf};
use crate::*;

/// Loosely based on Git's command-line interface.
/// 
/// We prefer using the sub-command as the method name prefix, followed by
/// an appropriate term for the common usage "form" referred to by the git man
/// page. Different forms of the same command are typically broken out into
/// their own methods.
/// 
/// We prefer options over parameters for anything non-essential to the purpose
/// of each function. Not all options are supported, while others are to-do.
pub trait GitInterface {
    /// See [add_with].
    fn add(&self, pathspec: &str) -> Result<()>;
    
    /// ```bash
    /// git add [--verbose | -v] [--dry-run | -n] [--force | -f] [--interactive | -i] [--patch | -p]
    ///        [--edit | -e] [--[no-]all | --[no-]ignore-removal | [--update | -u]] [--sparse]
    ///        [--intent-to-add | -N] [--refresh] [--ignore-errors] [--ignore-missing] [--renormalize]
    ///        [--chmod=(+|-)x] [--pathspec-from-file=<file> [--pathspec-file-nul]]
    ///        [--] [<pathspec>...]
    /// ```
    /// 
    /// See [AddOptions] for details on support.
    fn add_with(&self, pathspec: &str, options: AddOptions) -> Result<()>;
    
    fn add_all(&self) -> Result<()> {
        self.add(PATHSPEC_ALL)
    }
    
    /// See [branch_create_with].
    fn branch_create(
        &self,
        branch_name: &str,
    ) -> Result<()>;
    
    /// ```bash
    /// git branch [--track[=(direct|inherit)] | --no-track] [-f]
    ///      [--recurse-submodules] <branchname> [<start-point>]
    /// ```
    fn branch_create_with(
        &self,
        branch_name: &str,
        options: BranchCreateOptions
    ) -> Result<()>;
    
    /// `git branch --show-current`
    fn branch_current(&self) -> Result<String>;
    
    /// `git branch (-d | -D) [-r] <branchname>...`
    fn branch_delete(&self, branch_name: &str, force: bool, delete_remote: bool) -> Result<()>;
    
    /// `git branch --list`
    fn branch_list_local(&self) -> Result<Vec<String>>;

    /// `git branch --set-upstream-to=<remote_name>/<remote_branch_name>`
    fn branch_set_upstream(&self, upstream: &Upstream) -> Result<()>;
    
    /// The message will be trimmed and a single newline appended.
    fn commit(&self, message: &str) -> Result<()>;
    
    /// The message will be trimmed and a single newline appended.
    fn commit_with(&self, message: &str, options: CommitOptions) -> Result<()>;
    
    /// See [diff_status_revs_with].
    fn diff_revision_statuses(&self, rev_lhs: &str, rev_rhs: &str) -> Result<DiffStatus>;
    
    /// ```bash
    /// git diff [<options>] [--merge-base] <commit> <commit> [--] [<path>...]
    /// ```
    /// 
    /// View differences between two revisions.
    /// The option `--name-status` is applied, returning a result similar to `git status`.
    fn diff_revision_statuses_with(&self, rev_lhs: &str, rev_rhs: &str, options: DiffStatusOptions) -> Result<DiffStatus>;
    
    fn fetch(&self) -> Result<()>;
    
    fn fetch_with(&self, options: FetchOptions) -> Result<()>;
    
    fn fetch_all(&self) -> Result<()> {
        self.fetch_with(FetchOptions { all: true, ..Default::default() })
    }
    
    fn log(&self) -> Result<Log>;

    fn log_with(&self, options: LogOptions) -> Result<Log>;
    
    fn merge(&self, source_rev: &str) -> Result<Resolution>;
    
    fn merge_with(&self, source_rev: &str, options: MergeOptions) -> Result<Resolution>;
    
    fn merge_auto(&self, source_rev: &str) -> Result<Resolution> {
        self.merge_with(source_rev, MergeOptions {
            auto_resolve_only: true,
            ..Default::default()
        })
    }
    
    fn merge_forward(&self, source_rev: &str) -> Result<Resolution> {
        self.merge_with(source_rev, MergeOptions {
            fast_forward_only: true,
            ..Default::default()
        })
    }
    
    fn merge_abort(&self) -> Result<()>;
    
    //fn merge_continue(&self) -> Result<()>;
    
    fn move_rename(&self, from: &Path, to: &Path) -> Result<()>;
    
    fn pull(&self) -> Result<()>;
    
    fn pull_with(&self, options: PullOptions) -> Result<()>;
    
    fn pull_forward(&self) -> Result<()> {
        self.pull_with(PullOptions { fast_forward_only: true, ..Default::default() })
    }
    
    fn pull_forward_with(&self, mut options: PullOptions) -> Result<()> {
        options.fast_forward_only = true;
        self.pull_with(options)
    }
    
    fn pull_rebase(&self) -> Result<()> {
        self.pull_with(PullOptions { rebase: true, ..Default::default() })
    }
    
    fn pull_rebase_with(&self, mut options: PullOptions) -> Result<()> {
        options.rebase = true;
        self.pull_with(options)
    }
    
    fn push(&self) -> Result<()>;
    
    fn push_with(&self, options: PushOptions) -> Result<()>;
    
    fn push_new(&self) -> Result<()> {
        self.push_with(PushOptions {
            auto_set_upstream: true,
            ..Default::default()
        })
    }
    
    fn rebase(&self) -> Result<()>;
    
    fn rebase_with(&self, options: RebaseOptions) -> Result<()>;
    
    fn reset(&self) -> Result<()>;
    
    fn reset_with(&self, options: ResetOptions) -> Result<()>;
    
    fn reset_soft(&self) -> Result<()> {
        self.reset_with(ResetOptions { kind: ResetKind::Soft, ..ResetOptions::DEFAULT })
    }
    
    fn reset_soft_to(&self, to_rev: &str) -> Result<()> {
        self.reset_with(ResetOptions { kind: ResetKind::Soft, to_rev: Some(to_rev), ..ResetOptions::DEFAULT })
    }
    
    fn show_rev_file(&self, rev: &str, filepath: &Path) -> Result<String>;
    
    fn show_rev_file_bytes(&self, rev: &str, filepath: &Path) -> Result<Vec<u8>>;
    
    fn status(&self) -> Result<Status>;
    
    fn status_with(&self, options: StatusOptions) -> Result<Status>;
    
    /// See [switch_branch_with].
    fn switch_branch(&self, branch_name: &str) -> Result<()>;
    
    /// `git switch [<options>] --no-guess <branch>`
    fn switch_branch_with(&self, branch_name: &str, options: SwitchBranchOptions) -> Result<()>;
    
    /* TODO */
    
    // `git branch (-m | -M) [<oldbranch>] <newbranch>`
    //fn branch_move(&self, branch_name: &str, new_branch_name: &str, force: bool) -> Result<()>;
    

    //fn move_into(&self, from: Vec<&Path>, to_dir: &Path) -> Result<()>;
}
    
pub trait GitInterfaceConstruct: Sized {
    fn clone(repository: &str, top_dir: PathBuf) -> Result<Self>;
    
    /// ```bash
    /// git clone [--template=<template-directory>]
    ///      [-l] [-s] [--no-hardlinks] [-q] [-n] [--bare] [--mirror]
    ///      [-o <name>] [-b <name>] [-u <upload-pack>] [--reference <repository>]
    ///      [--dissociate] [--separate-git-dir <git-dir>]
    ///      [--depth <depth>] [--[no-]single-branch] [--no-tags]
    ///      [--recurse-submodules[=<pathspec>]] [--[no-]shallow-submodules]
    ///      [--[no-]remote-submodules] [--jobs <n>] [--sparse] [--[no-]reject-shallow]
    ///      [--filter=<filter> [--also-filter-submodules]] [--] <repository>
    ///      [<directory>]
    /// ```
    fn clone_with(repository: &str, top_dir: PathBuf, options: CloneOptions) -> Result<Self>;
    
    /// See [init_with].
    fn init(top_dir: PathBuf, initial_branch_name: &str) -> Result<Self>;
    
    /// ```bash
    /// git init [-q | --quiet] [--bare] [--template=<template-directory>]
    ///      [--separate-git-dir <git-dir>] [--object-format=<format>]
    ///      [-b <branch-name> | --initial-branch=<branch-name>]
    ///      [--shared[=<permissions>]] [<directory>]
    /// ```
    fn init_with(top_dir: PathBuf, initial_branch_name: &str, options: InitOptions) -> Result<Self>;
    
    /// See [init_bare].
    fn init_bare(top_dir: PathBuf, initial_branch_name: &str) -> Result<Self>;
    
    /// ```bash
    /// git init [-q | --quiet] [--bare] [--template=<template-directory>]
    ///      [--separate-git-dir <git-dir>] [--object-format=<format>]
    ///      [-b <branch-name> | --initial-branch=<branch-name>]
    ///      [--shared[=<permissions>]] [<directory>]
    /// ```
    fn init_bare_with(top_dir: PathBuf, initial_branch_name: &str, options: InitBareOptions) -> Result<Self>;
    
    fn open(working_dir: PathBuf) -> Result<Self>;
    
    fn open_with(working_dir: PathBuf, options: OpenOptions) -> Result<Self>;
}

pub trait WorkingRepo: GitInterface {
    fn git_env(&self) -> &GitEnv;
    fn git_env_mut(&mut self) -> &mut GitEnv;
    fn set_git_env(&mut self, env: GitEnv);
    fn top_dir(&self) -> &Path;
    fn working_dir(&self) -> &Path;
    
    fn is_merging(&self) -> Result<bool> {
        MergeModeMeta::exists(self.top_dir())
    }
}

pub trait AnyGitType: WorkingRepo + GitInterfaceConstruct {}

