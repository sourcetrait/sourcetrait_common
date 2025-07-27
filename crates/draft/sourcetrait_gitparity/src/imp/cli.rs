use crate::*;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::Arc;
use const_format::concatcp;

#[derive(Debug)]
pub struct GitCli {
    top_dir: PathBuf,
    working_dir: PathBuf,
    env: GitEnv,
}

impl GitCli {
    const ENV_GIT_AUTHOR_NAME: &'static str = "GIT_AUTHOR_NAME";
    const ENV_GIT_AUTHOR_EMAIL: &'static str = "GIT_AUTHOR_EMAIL";
    const ENV_GIT_AUTHOR_DATE: &'static str = "GIT_AUTHOR_DATE";
    const ENV_GIT_COMMITTER_NAME: &'static str = "GIT_COMMITTER_NAME";
    const ENV_GIT_COMMITTER_EMAIL: &'static str = "GIT_COMMITTER_EMAIL";
    const ENV_GIT_COMMITTER_DATE: &'static str = "GIT_COMMITTER_DATE";
    const CMD_ADD: &'static str = "add";
    const CMD_BRANCH: &'static str = "branch";
    const CMD_CLONE: &'static str = "clone";
    const CMD_COMMIT: &'static str = "commit";
    const CMD_DIFF: &'static str = "diff";
    const CMD_INIT: &'static str = "init";
    const CMD_FETCH: &'static str = "fetch";
    const CMD_LOG: &'static str = "log";
    const CMD_MERGE: &'static str = "merge";
    const CMD_MV: &'static str = "mv";
    const CMD_PUSH: &'static str = "push";
    const CMD_PULL: &'static str = "pull";
    const CMD_REBASE: &'static str = "rebase";
    const CMD_RESET: &'static str = "reset";
    const CMD_SHOW: &'static str = "show";
    const CMD_STATUS: &'static str = "status";
    const CMD_SWITCH: &'static str = "switch";

    pub(crate) fn open_unchecked(top_dir: PathBuf, working_dir: PathBuf, env: GitEnv) -> Self {
        // SAFETY: This should only be called when the repository path has been verified internally
        Self {
            top_dir,
            working_dir,
            env,
        }
    }

    fn build_command(&self) -> Command {
        let mut cmd = Command::new(GIT);
        cmd
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        if self.working_dir.is_dir() {
            cmd.current_dir(&self.working_dir());
        } else if self.top_dir != self.working_dir && self.top_dir.is_dir() {
            cmd.current_dir(&self.top_dir());
        }

        self.build_command_env(&mut cmd);
        cmd
    }

    fn build_command_env(&self, cmd: &mut Command) {
        let gitenv = self.git_env();

        if let Some(name) = gitenv.author_name() {
            cmd.env(Self::ENV_GIT_AUTHOR_NAME, name);
        }
        if let Some(email) = gitenv.author_email() {
            cmd.env(Self::ENV_GIT_AUTHOR_EMAIL, email);
        }
        if let Some(datestamp) = gitenv.author_datestamp() {
            cmd.env(Self::ENV_GIT_AUTHOR_DATE, datestamp);
        }
        if let Some(name) = gitenv.committer_name() {
            cmd.env(Self::ENV_GIT_COMMITTER_NAME, name);
        }
        if let Some(email) = gitenv.author_email() {
            cmd.env(Self::ENV_GIT_COMMITTER_EMAIL, email);
        }
        if let Some(datestamp) = gitenv.committer_datestamp() {
            cmd.env(Self::ENV_GIT_COMMITTER_DATE, datestamp);
        }
    }

    fn run_command(mut cmd: Command) -> Result<String> {
        let output = cmd.output().map_err(|_| Error::GitCmdRun)?;

        if !output.status.success() {
            let code = output.status.code().unwrap_or(0);
            let msg = String::from_utf8_lossy(&output.stderr).trim().into();
            Err(Error::GitCmd(code, Some(msg)))
        } else {
            let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
            Ok(stdout)
        }
    }

    fn run_command_output(mut cmd: Command) -> Result<Output> {
        let output = cmd.output().map_err(|_| Error::GitCmdRun)?;

        if !output.status.success() {
            let code = output.status.code().unwrap_or(0);
            let msg = String::from_utf8_lossy(&output.stderr).trim().into();
            Err(Error::GitCmd(code, Some(msg)))
        } else {
            Ok(output)
        }
    }

    fn run_command_quiet(mut cmd: Command) -> Result<()> {
        let output = cmd.output().map_err(|_| Error::GitCmdRun)?;

        if !output.status.success() {
            let code = output.status.code().unwrap_or(0);
            let msg = String::from_utf8_lossy(&output.stderr).trim().into();
            Err(Error::GitCmd(code, Some(msg)))
        } else {
            Ok(())
        }
    }

    fn run_command_status(mut cmd: Command) -> Result<()> {
        let exit_status = cmd.status().map_err(|_| Error::GitCmdRun)?;

        if exit_status.success() {
            Ok(())
        } else {
            let code = exit_status.code().unwrap_or(1);
            Err(Error::GitCmd(code, None))
        }
    }
}

impl WorkingRepo for GitCli {
    fn git_env(&self) -> &GitEnv {
        &self.env
    }
    
    fn git_env_mut(&mut self) -> &mut GitEnv {
        &mut self.env
    }
    
    fn set_git_env(&mut self, env: GitEnv) {
        self.env = env;
    }

    fn top_dir(&self) -> &Path {
        &self.top_dir
    }

    fn working_dir(&self) -> &Path {
        &self.working_dir
    }
}

impl GitCli {
    fn add_impl(&self, pathspec: &str, _options: Option<AddOptions>) -> Result<()> {
        let mut cmd = self.build_command();
        cmd.arg(Self::CMD_ADD).arg(pathspec);
        Self::run_command_quiet(cmd)
    }

    fn branch_create_impl(
        &self,
        branch_name: &str,
        options: Option<BranchCreateOptions>,
    ) -> Result<()> {
        let mut cmd = self.build_command();
        cmd.arg(Self::CMD_BRANCH);

        if let Some(options) = options {
            if options.orphan {
                cmd.arg("--orphan");
            }
            if let Some(start_point) = options.start_point {
                cmd.arg(start_point.as_ref());
            }
        }

        cmd.arg(branch_name);

        Self::run_command_quiet(cmd)
    }

    fn branch_set_upstream_impl(&self, upstream: &Upstream) -> Result<()> {
        let mut cmd = self.build_command();
        cmd.arg(Self::CMD_BRANCH)
            .arg("--set-upstream-to")
            .arg([upstream.remote_name(), "/", upstream.remote_branch_name()].concat());

        Self::run_command_quiet(cmd)
    }

    fn commit_impl(&self, message: &str, _options: Option<CommitOptions>) -> Result<()> {
        let mut cmd = self.build_command();
        cmd.arg(Self::CMD_COMMIT)
            .arg("-m").arg(message.trim()); // trim ensures parity between implementations

        Self::run_command_status(cmd)
    }

    fn diff_revision_statuses_impl(
        &self,
        rev_lhs: &str,
        rev_rhs: &str,
        _options: Option<DiffStatusOptions>,
    ) -> Result<DiffStatus> {
        let mut cmd = self.build_command();
        cmd.arg(Self::CMD_DIFF)
            .arg("--name-status")
            .arg(rev_lhs)
            .arg(rev_rhs);

        let stdout = Self::run_command(cmd)?;
        diff_status_from_cli(&stdout)
    }

    fn fetch_impl(&self, options: Option<FetchOptions>) -> Result<()> {
        let mut cmd = self.build_command();
        cmd.arg(Self::CMD_FETCH);

        if let Some(opts) = options {
            if opts.all {
                cmd.arg("--all");
            }
            if opts.prune {
                cmd.arg("--prune");
            }
        }

        Self::run_command_quiet(cmd)
    }

    fn log_impl(&self, options: Option<LogOptions>) -> Result<Log> {
        let options = match options {
            Some(o) => o.validate()?,
            None => LogOptions::DEFAULT
        };
        
        let format = match (options.show_message, options.show_signature_fingerprint) {
            (false, false) => LOG_FORMAT,
            (true, false) => concatcp!(LOG_FORMAT, LOG_FORMAT_MESSAGE),
            (false, true) => concatcp!(LOG_FORMAT, LOG_FORMAT_SIGNATURE_FINGERPRINT),
            (true, true) => concatcp!(
                LOG_FORMAT,
                LOG_FORMAT_MESSAGE,
                LOG_FORMAT_SIGNATURE_FINGERPRINT
            ),
        };

        let mut cmd = self.build_command();
        cmd.arg(Self::CMD_LOG)
            .arg(format);

        let stdout = Self::run_command(cmd)?;
        Log::from_cli(&stdout, Some(options))
    }
    
    fn merge_impl(&self, source_rev: &str, options: Option<MergeOptions>) -> Result<Resolution> {
        let options = match options {
            Some(o) => o.validate()?,
            None => MergeOptions::DEFAULT
        };
        
        let mut cmd = self.build_command();
        cmd.arg(Self::CMD_MERGE)
            .arg("--quiet")
            .arg("--no-commit");
        
        if options.fast_forward_only {
            cmd.arg("--ff-only");
        }
        
        cmd.arg(source_rev);
        
        let check = ResolutionCheck::before(self)?;
        let resolution = check.after(self, Self::run_command_quiet(cmd))
            .map_err(|e| Error::MergeAborted(source_rev.into(), ErrSrc::Lib(Box::new(e)), None))?;
        
        if !resolution.is_unresolved() {
            return Ok(resolution);
        }
        
        // abort if necessary
        if options.auto_resolve_only {
            let mut cmd = Self::build_command(&self);
            cmd.arg(Self::CMD_MERGE)
                .arg("--abort");
            
            Self::run_command_quiet(cmd)
                .map_err(|e| Error::MergeUnaborted(source_rev.into(), ErrSrc::Lib(Box::new(e)),
                    Some("Failed to abort problematic merge".into())
                ))?;
            
            Err(Error::MergeAborted(source_rev.into(), ErrSrc::None, None))
        } else {
            Ok(resolution)
        }
    }
    
    fn merge_abort_impl(&self) -> Result<()> {
        let mut cmd = self.build_command();
        cmd.arg(Self::CMD_MERGE)
            .arg("--abort");
        
        Self::run_command_quiet(cmd)
    }

    fn pull_impl(&self, options: Option<PullOptions>) -> Result<()> {
        let options = match options {
            Some(o) => o.validate()?,
            None => PullOptions::DEFAULT
        };
        
        let mut cmd = self.build_command();
        cmd.arg(Self::CMD_PULL);

        if options.rebase {
            cmd.arg("--rebase");
        } else if options.fast_forward_only {
            cmd.arg("--ff-only");
        }

        Self::run_command_quiet(cmd)
    }

    fn push_impl(&self, options: Option<PushOptions>) -> Result<()> {
        let options = match options {
            Some(o) => o.validate()?,
            None => PushOptions::DEFAULT
        };
        
        let mut cmd = self.build_command();
        cmd.arg(Self::CMD_PUSH);

        if let Some(upstream) = options.upstream {
            cmd.arg(upstream.remote_name()).arg(upstream.remote_branch_name());
        } else if let Some(upstream) = options.set_upstream {
            cmd.arg("-u").arg(upstream.remote_name()).arg(upstream.remote_branch_name());
        } else if options.auto_set_upstream {
            let branch_name = self.branch_current()?;
            cmd.arg("-u").arg(ORIGIN).arg(branch_name);
        }

        Self::run_command_quiet(cmd)
    }

    fn rebase_impl(&self, options: Option<RebaseOptions>) -> Result<()> {
        let _options = match options {
            Some(o) => o.validate()?,
            None => RebaseOptions::DEFAULT
        };

        let mut cmd = self.build_command();
        cmd.arg(Self::CMD_REBASE);
        
        Self::run_command_quiet(cmd)
    }
    
    fn reset_impl(&self, options: Option<ResetOptions>) -> Result<()> {
        let options = match options {
            Some(o) => o.validate()?,
            None => ResetOptions::DEFAULT
        };
        
        let mut cmd = self.build_command();
        cmd.arg(Self::CMD_RESET);
        
        match options.kind {
            ResetKind::Hard => { cmd.arg("--hard"); },
            ResetKind::Soft => { cmd.arg("--soft"); },
            _ => {},
        }
        
        if let Some(to_rev) = options.to_rev {
            cmd.arg(to_rev);
        }
        
        Self::run_command_quiet(cmd)
    }
    
    fn status_impl(&self, _options: Option<StatusOptions>) -> Result<Status> {
        let mut cmd = self.build_command();
        cmd.arg(Self::CMD_STATUS).arg("--porcelain");

        let stdout = Self::run_command(cmd)?;
        let status = Status::from_cli(&stdout)?;
        Ok(status)
    }

    fn switch_branch_impl(
        &self,
        branch_name: &str,
        _options: Option<SwitchBranchOptions>,
    ) -> Result<()> {
        let mut cmd = self.build_command();
        cmd.arg(Self::CMD_SWITCH).arg(branch_name);

        Self::run_command_status(cmd)
    }
}

impl GitCli {
    fn clone_impl(
        repository: &str,
        top_dir: PathBuf,
        options: Option<CloneOptions>,
    ) -> Result<Self> {
        if top_dir.exists() {
            return Err(Error::Open(top_dir.clone(), ErrSrc::None,
                Some("Directory already exists".into()),
            ));
        }

        let env = options.unwrap_or_default().env.unwrap_or_default();

        let working_dir = top_dir.clone();
        let git = Self::open_unchecked(top_dir, working_dir, env);

        let mut cmd = git.build_command();
        cmd.arg(Self::CMD_CLONE).arg(repository).arg(git.top_dir());

        Self::run_command(cmd)?;
        Ok(git)
    }

    fn init_impl(
        top_dir: PathBuf,
        initial_branch_name: &str,
        options: Option<InitOptions>,
    ) -> Result<Self> {
        if !top_dir.is_dir() {
            std::fs::create_dir_all(&top_dir).map_err(|e| {
                Error::Init(top_dir.clone(), ErrSrc::Io(e),
                    Some("Unable to create directory".into()),
                )
            })?;
        } else {
            let num_items = top_dir
                .read_dir()
                .map_err(|e| {
                    Error::Init(top_dir.clone(), ErrSrc::Io(e),
                        Some("Unable to access directory".into()),
                    )
                })?
                .count();

            if num_items > 0 {
                return Err(Error::Init(top_dir.clone(), ErrSrc::None,
                    Some("Cannot create repository in a non-empty directory".into())
                ));
            }
        }

        let env = options.unwrap_or_default().env.unwrap_or_default();

        let working_dir = top_dir.clone();
        let git = Self::open_unchecked(top_dir, working_dir, env);

        let mut cmd = git.build_command();
        cmd.arg(Self::CMD_INIT)
            .arg("-b")
            .arg(initial_branch_name)
            .arg(git.top_dir());

        Self::run_command(cmd)?;
        Ok(git)
    }

    fn init_bare_impl(
        top_dir: PathBuf,
        initial_branch_name: &str,
        options: Option<InitBareOptions>,
    ) -> Result<Self> {
        if !top_dir.is_dir() {
            std::fs::create_dir_all(&top_dir).map_err(|e|
                Error::Init(top_dir.clone(), ErrSrc::Io(e),
                    Some("Failed to create directory".into()))
            )?;
        } else {
            let num_items = top_dir
                .read_dir()
                .map_err(|e| Error::Init(top_dir.clone(), ErrSrc::Io(e), None))?
                .count();

            if num_items > 0 {
                return Err(Error::Init(top_dir.into(), ErrSrc::None,
                    Some("Directory is not empty".into())
                ));
            }
        }

        let env = options.unwrap_or_default().env.unwrap_or_default();

        let working_dir = top_dir.clone();
        let git = Self::open_unchecked(top_dir, working_dir, env);

        let mut cmd = git.build_command();
        cmd.arg(Self::CMD_INIT)
            .arg("--bare")
            .arg("-b")
            .arg(initial_branch_name);

        cmd.arg(git.top_dir());

        Self::run_command(cmd)?;
        Ok(git)
    }

    fn open_impl(working_dir: PathBuf, options: Option<OpenOptions>) -> Result<Self> {
        let top_dir = stdx::fs::find_parent_dir(&working_dir, DOT_GIT)
            .ok_or_else(|| {
                Error::Open(working_dir.clone(), ErrSrc::None,
                    Some("Not a git repository".into())
                )
            })?;

        let env = options.unwrap_or_default().env.unwrap_or_default();

        Ok(Self {
            top_dir,
            working_dir,
            env,
        })
    }
}

impl GitInterfaceConstruct for GitCli {
    fn clone(repository: &str, top_dir: PathBuf) -> Result<Self> {
        Self::clone_impl(repository, top_dir, None)
    }

    fn clone_with(repository: &str, top_dir: PathBuf, options: CloneOptions) -> Result<Self> {
        Self::clone_impl(repository, top_dir, Some(options))
    }

    fn init(top_dir: PathBuf, initial_branch_name: &str) -> Result<Self> {
        Self::init_impl(top_dir, initial_branch_name, None)
    }

    fn init_with(
        top_dir: PathBuf,
        initial_branch_name: &str,
        options: InitOptions,
    ) -> Result<Self> {
        Self::init_impl(top_dir, initial_branch_name, Some(options))
    }

    fn init_bare(top_dir: PathBuf, initial_branch_name: &str) -> Result<Self> {
        Self::init_bare_impl(top_dir, initial_branch_name, None)
    }

    fn init_bare_with(
        top_dir: PathBuf,
        initial_branch_name: &str,
        options: InitBareOptions,
    ) -> Result<Self> {
        Self::init_bare_impl(top_dir, initial_branch_name, Some(options))
    }

    fn open(working_dir: PathBuf) -> Result<Self> {
        Self::open_impl(working_dir, None)
    }

    fn open_with(working_dir: PathBuf, options: OpenOptions) -> Result<Self> {
        Self::open_impl(working_dir, Some(options))
    }
}

impl GitInterface for GitCli {
    fn add(&self, pathspec: &str) -> Result<()> {
        self.add_impl(pathspec, None)
    }

    fn add_with(&self, pathspec: &str, options: AddOptions) -> Result<()> {
        self.add_impl(pathspec, Some(options))
    }

    fn branch_create(&self, branch_name: &str) -> Result<()> {
        self.branch_create_impl(branch_name, None)
    }

    fn branch_create_with(&self, branch_name: &str, options: BranchCreateOptions) -> Result<()> {
        self.branch_create_impl(branch_name, Some(options))
    }

    fn branch_current(&self) -> Result<String> {
        let mut cmd = self.build_command();
        cmd.arg(Self::CMD_BRANCH).arg("--show-current");

        let stdout = Self::run_command(cmd)?.trim().into();
        Ok(stdout)
    }

    fn branch_delete(&self, branch_name: &str, force: bool, delete_remote: bool) -> Result<()> {
        let mut cmd = self.build_command();
        cmd.arg(Self::CMD_BRANCH)
            .arg(if force { "-D" } else { "-d" })
            .arg(branch_name);

        if delete_remote {
            cmd.arg("-r");
        }

        Self::run_command_status(cmd)
    }

    fn branch_list_local(&self) -> Result<Vec<String>> {
        let mut cmd = self.build_command();
        cmd.arg(Self::CMD_BRANCH).arg("--no-color");

        let branches = Self::run_command(cmd)?
            .lines()
            .map(|line| line.trim().trim_start_matches("* ").into())
            .collect::<Vec<_>>();

        Ok(branches)
    }

    fn branch_set_upstream(&self, upstream: &Upstream) -> Result<()> {
        self.branch_set_upstream_impl(upstream)
    }

    fn commit(&self, message: &str) -> Result<()> {
        self.commit_impl(message, None)
    }

    fn commit_with(&self, message: &str, options: CommitOptions) -> Result<()> {
        self.commit_impl(message, Some(options))
    }

    fn diff_revision_statuses(&self, rev_lhs: &str, rev_rhs: &str) -> Result<DiffStatus> {
        self.diff_revision_statuses_impl(rev_lhs, rev_rhs, None)
    }

    fn diff_revision_statuses_with(&self, rev_lhs: &str, rev_rhs: &str, options: DiffStatusOptions) -> Result<DiffStatus> {
        self.diff_revision_statuses_impl(rev_lhs, rev_rhs, Some(options))
    }

    fn fetch(&self) -> Result<()> {
        self.fetch_impl(None)
    }

    fn fetch_with(&self, options: FetchOptions) -> Result<()> {
        self.fetch_impl(Some(options))
    }

    fn log(&self) -> Result<Log> {
        self.log_impl(None)
    }

    fn log_with(&self, options: LogOptions) -> Result<Log> {
        self.log_impl(Some(options))
    }
    
    fn merge(&self, source_rev: &str) -> Result<Resolution> {
        self.merge_impl(source_rev, None)
    }
    
    fn merge_with(&self, source_rev: &str, options: MergeOptions) -> Result<Resolution> {
        self.merge_impl(source_rev, Some(options))
    }
    
    fn merge_abort(&self) -> Result<()> {
        self.merge_abort_impl()
    }

    fn move_rename(&self, from: &Path, to: &Path) -> Result<()> {
        let mut cmd = self.build_command();
        cmd.arg(Self::CMD_MV).arg(from).arg(to);

        Self::run_command_quiet(cmd)
    }

    fn pull(&self) -> Result<()> {
        self.pull_impl(None)
    }

    fn pull_with(&self, options: PullOptions) -> Result<()> {
        self.pull_impl(Some(options))
    }

    fn push(&self) -> Result<()> {
        self.push_impl(None)
    }

    fn push_with(&self, options: PushOptions) -> Result<()> {
        self.push_impl(Some(options))
    }

    fn rebase(&self) -> Result<()> {
        self.rebase_impl(None)
    }

    fn rebase_with(&self, options: RebaseOptions) -> Result<()> {
        self.rebase_impl(Some(options))
    }
    
    fn reset(&self) -> Result<()> {
        self.reset_impl(None)
    }
    
    fn reset_with(&self, options: ResetOptions) -> Result<()> {
        self.reset_impl(Some(options))
    }
    
    fn show_rev_file(&self, rev: &str, filepath: &Path) -> Result<String> {
        let mut cmd = self.build_command();
        cmd.arg(Self::CMD_SHOW)
            .arg(&[rev.as_ref(), ":", filepath.to_string_lossy().as_ref()].concat());

        Self::run_command(cmd)
    }

    fn show_rev_file_bytes(&self, rev: &str, filepath: &Path) -> Result<Vec<u8>> {
        let mut cmd = self.build_command();
        cmd.arg(Self::CMD_SHOW)
            .arg(&[rev.as_ref(), ":", filepath.to_string_lossy().as_ref()].concat());

        let output = Self::run_command_output(cmd)?;
        Ok(output.stdout)
    }

    fn status(&self) -> Result<Status> {
        self.status_impl(None)
    }

    fn status_with(&self, options: StatusOptions) -> Result<Status> {
        self.status_impl(Some(options))
    }

    fn switch_branch(&self, branch_name: &str) -> Result<()> {
        self.switch_branch_impl(branch_name, None)
    }

    fn switch_branch_with(&self, branch_name: &str, options: SwitchBranchOptions) -> Result<()> {
        self.switch_branch_impl(branch_name, Some(options))
    }
}

pub struct ResolutionCheck {
    head_ref_filepath: PathBuf,
    head_ref_file_contents_before: String,
}

impl ResolutionCheck {
    pub fn before(git: &GitCli) -> Result<Self> {
        let head_ref_filepath = git.top_dir()
            .join(DOT_GIT)
            .join(Self::read_head_symbolic_ref_name(git)?);
        let head_ref_file_contents_before = Self::read_head_ref_file(&head_ref_filepath)?;
        
        Ok(Self {
            head_ref_filepath,
            head_ref_file_contents_before
        })
    }
    
    pub fn after(self, git: &GitCli, merge_result: Result<()>) -> Result<Resolution> {
        let exit_ok = merge_result.is_ok();
        let is_merging = Self::merge_mode_file_exists(git)?;
        
        if exit_ok && !is_merging {
            // it's either: up-to-date, fast-forwarded
            // only fast-forwarded can be known by a simple check  
            let head_ref_file_contents_after = Self::read_head_ref_file(&self.head_ref_filepath)?;
            if head_ref_file_contents_after != self.head_ref_file_contents_before {
                return Ok(Resolution::FastForwarded);
            }
        }
        
        let status = git.status()?;
        
        if is_merging {
            // either: unmerged or error
            if exit_ok {
                if !status.has_conflicts() && !status.is_unmodified() {
                    Ok(Resolution::AutoResolved)
                } else {
                    Err(Error::State(StateErr::UnexpectedStatus, ErrSrc::None))
                }
            } else {
                // unmerged
                if let Some(conflicts) = status.into_conflicts() {
                    Ok(Resolution::Unresolved(conflicts))
                } else {
                    Err(Error::State(StateErr::UnexpectedStatus, ErrSrc::None))
                }
            }
        } else {
            // either: up-to-date or err
            if exit_ok {
                if status.is_unmodified() {
                    Ok(Resolution::Unmodified)
                } else {
                    Err(Error::State(StateErr::UnexpectedStatus, ErrSrc::None))
                }
            } else {
                Err(merge_result.err().expect("err"))
            }
        }
    }
    
    fn read_head_symbolic_ref_name(git: &GitCli) -> Result<String> {
        let ref_name = fs::read_to_string(git.top_dir().join(DOT_GIT).join(HEAD))
            .map_err(|e| Error::State(StateErr::InternalsFileIO, ErrSrc::Io(e)))?
            .trim_start_matches("ref: ")
            .trim_end()
            .into();
        
        Ok(ref_name)
    }
    
    fn merge_mode_file_exists(git: &GitCli) -> Result<bool> {
        git.top_dir().join(DOT_GIT).join("MERGE_MODE").try_exists()
            .map_err(|e| Error::State(StateErr::InternalsFileIO, ErrSrc::Io(e)))
    }
    
    fn read_head_ref_file(path: &Path) -> Result<String> {
        let commit_hash = fs::read_to_string(path)
            .map_err(|e| Error::State(StateErr::InternalsFileIO, ErrSrc::Io(e)))?
            .trim()
            .into();
        
        Ok(commit_hash)
    }
}

fn diff_status_from_cli(s: &str) -> Result<DiffStatus> {
    let path_diffs = s.lines()
        .map(|line| path_diff_status_from_cli(line))
        .collect::<Result<Vec<_>>>()?;
    
    // panic on unexpected
    #[cfg(debug_assertions)] {
        let _ = path_diffs.iter()
            .find(|d| matches!(d.code, StatusCode::Unmerged | StatusCode::Untracked))
            .is_none_or(|_| panic!("unexpected status code")); 
    }
    
    let changes = path_diffs.into_iter()
        .map(|d| (d.path.clone(), d))
        .collect::<HashMap<_,_>>();

    Ok(DiffStatus::new(changes))
}

fn path_diff_status_from_cli(line: &str) -> Result<PathDiffStatus> {
    let items = line.split_whitespace()
        .map(|s| s.trim_start_matches(['"', '\'']).trim_end_matches(['"', '\'']))
        .collect::<Vec<_>>();

    if items.len() < 2 {
        return Err(Error::GitStatusParse);
    }

    // parse code, ignore diff score
    let mut char_indices = items[0].char_indices();
    let code = char_indices
        .next()
        .ok_or_else(|| Error::GitStatusParse)
        .map(|(_,c)| StatusCode::try_from_char(c))
        .map_err(|_| Error::GitStatusParse)??
        .ok_or_else(|| Error::GitStatusParse)?;
    
    let (path, original_path) = match items.len() {
        2 => (PathBuf::from(items[1]), None),
        3 => ( 
            PathBuf::from(items[2]),
            Some(PathBuf::from(items[1]))
        ),
        _ => {
            return Err(Error::GitStatusParse);
        }
    };

    let path = Arc::new(path);
    
    Ok(PathDiffStatus {
        path,
        code,
        original_path,
    })
}
