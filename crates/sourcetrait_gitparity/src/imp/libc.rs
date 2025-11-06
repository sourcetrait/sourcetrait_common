use std::{collections::{HashMap, HashSet}, fs, path::{Path, PathBuf}, sync::Arc};
use chrono::DateTime;
use git2;
use hex::ToHex;
use crate::*;

const ALL_REFSPECS: [&'static str; 0] = [];
const FETCH_HEAD: &'static str = "FETCH_HEAD";
const REFLOG_FAST_FORWARD: &'static str = "Fast-forward";

pub struct GitLibC {
    top_dir: PathBuf,
    working_dir: PathBuf,
    env: GitEnv,
    repo: git2::Repository,
}

impl GitLibC {
    fn author_signature(&self) -> Result<git2::Signature<'_>> {
        let env = &self.env;
        let sig = if env.author_name().is_some() && env.author_email().is_some() {
            if let Some(date) = &env.author_date() {
                let time = git2::Time::new(date.timestamp(), 0);
                git2::Signature::new(
                    env.author_name().as_ref().expect("some"),
                    env.author_email().as_ref().expect("some"),
                    &time
                )
            } else {
                git2::Signature::now(
                    env.author_name().as_ref().expect("some"),
                    env.author_email().as_ref().expect("some"),
                )
            }
        } else {
            let sig = self.repo.signature()
                .map_err(|e| Error::Config("user".into(), ErrSrc::LibC(e)))?;

            let name = env.author_name();
            let name = name.as_deref()
                .unwrap_or(sig.name().expect("some"));
            let email = env.author_email();
            let email = email.as_deref()
                .unwrap_or(sig.email().expect("some"));

            if let Some(date) = &env.author_date() {
                let time = git2::Time::new(date.timestamp(), 0);
                git2::Signature::new(name, email, &time)
            } else {
                git2::Signature::now(name, email)
            }
        };

        sig.map_err(|e| Error::Config("user".into(), ErrSrc::LibC(e)))
    }

    fn committer_signature(&self) -> Result<git2::Signature<'_>> {
        let env = &self.env;
        let sig = if env.committer_name().is_some() && env.committer_email().is_some() {
            if let Some(date) = &env.committer_date() {
                let time = git2::Time::new(date.timestamp(), 0);
                git2::Signature::new(
                    env.committer_name().as_ref().expect("some"),
                    env.committer_email().as_ref().expect("some"),
                    &time
                )
            } else {
                git2::Signature::now(
                    env.committer_name().as_ref().expect("some"),
                    env.committer_email().as_ref().expect("some"),
                )
            }
        } else {
            let sig = self.repo.signature()
                .map_err(|e| Error::Config("user".into(), ErrSrc::LibC(e)))?;

            let name = env.committer_name();
            let name = name.as_deref()
                .unwrap_or(sig.name().expect("some"));
            let email = env.committer_email();
            let email = email.as_deref()
                .unwrap_or(sig.email().expect("some"));

            if let Some(date) = &env.committer_date() {
                let time = git2::Time::new(date.timestamp(), 0);
                git2::Signature::new(name, email, &time)
            } else {
                git2::Signature::now(name, email)
            }
        };

        sig.map_err(|e| Error::Config("user".into(), ErrSrc::LibC(e)))
    }
    
    fn enter_merge_mode(&self, merge_meta: MergeModeMeta) -> Result<()> {
        merge_meta.write(&self.top_dir())
    }
    
    #[allow(dead_code)]
    fn exit_merge_mode(&self) -> Result<()> {
        MergeModeMeta::delete(self.top_dir())?;
        
        self.repo.cleanup_state()
            .map_err(|e| Error::State(StateErr::UnexpectedStatus, ErrSrc::LibC(e)))?;
        
        Ok(())
    }

    fn head(&self) -> Result<git2::Reference<'_>> {
        self.repo.head()
            .map_err(|e| Error::State(StateErr::HeadNotFound, ErrSrc::LibC(e)))
    }

    fn head_name<'s:'a,'a>(&'s self, head: &'a git2::Reference<'a>) -> Result<&'a str> {
        head.shorthand()
            .ok_or_else(|| Error::State(StateErr::HeadNameNotFound, ErrSrc::None))
    }

    fn head_commit<'s:'a,'r:'a,'a>(&'s self, head: &'a git2::Reference<'r>) -> Result<git2::Commit<'r>> {
        head.peel_to_commit()
            .map_err(|e| Error::State(StateErr::HeadCommitNotFound, ErrSrc::LibC(e)))
    }

    fn index(&self) -> Result<git2::Index> {
        self.repo.index()
            .map_err(|e| Error::State(StateErr::IndexNotFound, ErrSrc::LibC(e)))
    }
    
    fn upstream<'s:'r,'r:'a,'a>(&'s self, head: &'a git2::Reference<'r>) -> Result<Option<git2::Branch<'r>>> {
        let branch_name = self.head_name(head)?;
        
        self.repo.find_branch(branch_name, git2::BranchType::Local)
            .map_err(|e| Error::BranchNotFound(branch_name.into(), ErrSrc::LibC(e)))?
            .upstream()
            .map_or_else(
                |e| {
                    if e.code() == git2::ErrorCode::NotFound {
                        Ok(None)
                    } else {
                        Err(Error::State(StateErr::UpstreamNotFound, ErrSrc::LibC(e)))
                    }
                },
                |u| Ok(Some(u))
            )
    }
    
    /// Throws an [Error::UpstreamNotFound] if not configured.
    fn try_upstream<'s:'r,'r:'a,'a>(&'s self, head: &'a git2::Reference<'r>) -> Result<git2::Branch<'r>> {
        self.upstream(head)?
            .ok_or_else(|| Error::UpstreamNotFound(
                self.head_name(head).map_or_else(|_| "HEAD", |n| n).into(),
                ErrSrc::None
            ))
    }
    
    fn upstream_name<'s:'r,'r:'a,'a>(&'s self, upstream: &'a git2::Branch<'r>) -> Result<&'a str> {
        upstream.get().name()
            .ok_or_else(|| Error::State(StateErr::UnsupportedBranchName, ErrSrc::None))
    }
    
    fn upstream_ref_name<'s:'r,'r:'a,'a>(&'s self, upstream_ref: &'a git2::Reference<'r>) -> Result<(Upstream<'a>, &'a str)> {
        let ref_name = upstream_ref.name()
            .ok_or_else(|| Error::State(StateErr::UnsupportedBranchName, ErrSrc::None))?;
        let upstream_name = Upstream::from_ref_name(ref_name)?;
        Ok((upstream_name, ref_name))
    }
    
    fn remote_for_branch<'s:'r,'r:'a,'a>(&'s self, branch_name: &'a str) -> Result<Option<git2::Remote<'r>>> {
        let upstream = self.repo.find_branch(branch_name, git2::BranchType::Local)
            .map_err(|e| Error::BranchNotFound(branch_name.into(), ErrSrc::LibC(e)))?
            .upstream();

        let upstream = if let Ok(upstream) = upstream {
            upstream
        } else {
            return Ok(None);
        };

        let upstream_ref_name = upstream.get().name()
            .ok_or_else(|| Error::State(StateErr::UnsupportedBranchName, ErrSrc::None))?;

        let upstream_name = Upstream::from_ref_name(upstream_ref_name)?;

        self.repo.find_remote(upstream_name.remote_name())
            .map(|r| Some(r))
            .map_err(|e| Error::RemoteNotFound(upstream_name.remote_name().into(), ErrSrc::LibC(e)))
    }
    
    fn get_rev_file_blob<'s:'r,'r:'a,'a>(&'s self, rev: &'a str, filepath: &'a Path) -> Result<git2::Blob<'r>> {
        let entry = self.repo.revparse_single(rev)
            .map_err(|e| Error::RevNotFound(rev.into(), ErrSrc::LibC(e), None))?
            .peel_to_commit()
            .map_err(|e| Error::RevNotFound(rev.into(), ErrSrc::LibC(e), None))?
            .tree()
            .map_err(|e| Error::RevNotFound(rev.into(), ErrSrc::LibC(e), None))?
            .get_path(filepath)
            .map_err(|e| Error::PathNotFound(rev.into(), filepath.to_string_lossy().into(), ErrSrc::LibC(e), None))?;
        
        self.repo.find_blob(entry.id())
            .map_err(|e| Error::PathNotFound(rev.into(), filepath.to_string_lossy().into(), ErrSrc::LibC(e), None))
    }
    
    fn remote_callbacks<'a>() -> git2::RemoteCallbacks<'a> {
        let mut callbacks = git2::RemoteCallbacks::new();
        callbacks.credentials(|_, username, _| {
            git2::Cred::ssh_key_from_agent(username.unwrap_or(GIT))
        });
        
        callbacks
    }
    
    fn upstream_default_branch(repository: &str) -> Result<String> {
        match url::Url::parse(repository) {
            Ok(url) => Self::upstream_url_default_branch(url),
            Err(_) => Self::upstream_path_default_branch(Path::new(repository)),
        }
    }
    
    fn upstream_path_default_branch(path: &Path) -> Result<String> {
        let repo = git2::Repository::open_bare(path)
            .map_err(|e| Error::Open(path.into(), ErrSrc::LibC(e), None))?;
        
        
        repo.find_reference(HEAD)
            .map_err(|e| Error::State(StateErr::HeadNotFound, ErrSrc::LibC(e)))?
            .symbolic_target()
            .ok_or_else(|| Error::Open(path.into(), ErrSrc::None,
                Some("HEAD does not have a symbolic target".into())
            ))?
            .strip_prefix("refs/heads/")
            .ok_or_else(|| Error::Open(path.into(), ErrSrc::None,
                Some("Invalid HEAD name on repo".into())
            ))
            .map(|s| s.into())
    }
    
    fn upstream_url_default_branch(url: url::Url) -> Result<String> {
        let repository = url.as_str();
        let mut remote = git2::Remote::create_detached(url.as_str())
            .map_err(|e| Error::Connect(repository.into(), ErrSrc::LibC(e), None))?;
    
        remote.connect_auth(git2::Direction::Fetch, Some(Self::remote_callbacks()), None)
            .map_err(|e| Error::Connect(repository.into(), ErrSrc::LibC(e), None))?;
    
        // refs/heads/...
        remote.default_branch()
            .map_err(|e| Error::Connect(repository.into(), ErrSrc::LibC(e), None))?
            .as_str()
            .ok_or_else(|| Error::Connect(repository.into(), ErrSrc::None,
                Some("Invalid default branch name".into())
            ))?
            .strip_prefix("refs/heads/")
            .ok_or_else(|| Error::Connect(repository.into(), ErrSrc::None,
                Some("Invalid default branch name".into())
            ))
            .map(|s| s.into())
    }
    
    fn config(&self) -> Result<git2::Config> {
        self.repo.config()
            .map_err(|e| Error::Config(self.top_dir.to_string_lossy().into(), ErrSrc::LibC(e)))
    }
    
    fn config_entry<'s, 'a, 'b>(&'s self, config: &'a git2::Config, key: &'b str) -> Result<Option<git2::ConfigEntry<'a>>> {
        config
            .get_entry(key)
            .map_or_else(|_| Ok(None), |entry| if entry.has_value() {
                Ok(Some(entry))
            } else {
                Ok(None)
            }) 
    }
    
    /// Expects an entry validated through [Self::config_entry]
    fn config_entry_str<'s, 'a>(&'s self, entry: &'a git2::ConfigEntry) -> Result<&'a str> {
        entry.value()
            .ok_or_else(|| Error::Config(self.top_dir.to_string_lossy().into(), ErrSrc::None))
    }
}

impl WorkingRepo for GitLibC {
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

impl GitLibC {
    fn clone_impl(repository: &str, top_dir: PathBuf, options: Option<CloneOptions>) -> Result<Self> {
        if top_dir.exists() {
            return Err(Error::Open(top_dir.clone(), ErrSrc::None,
                Some("Directory already exists".into()),
            ));
        }

        let default_branch_name = Self::upstream_default_branch(repository)?;
        let env = options.unwrap_or_default().env.unwrap_or_default();
        let working_dir = top_dir.clone();
        
        let mut fetch_options = git2::FetchOptions::new();
        fetch_options.remote_callbacks(Self::remote_callbacks());
        let repo = git2::build::RepoBuilder::new()
            .fetch_options(fetch_options)
            .clone(repository, &top_dir)
            .map_err(|e| Error::Clone(top_dir.clone(), ErrSrc::LibC(e), None))?;
        
        // lib2 doesn't respect default branch names as paramters properly,
        // including in the config file. so ...
        if default_branch_name != MASTER {
            // lib2 has no means of deleting an config entire section,
            // so we have to edit the config file directly regardless
            let config_filepath = top_dir.join(DOT_GIT).join("config");
            let config_txt = fs::read_to_string(&config_filepath)
                .map_err(|e| Error::Config(top_dir.to_string_lossy().into(), ErrSrc::Io(e)))?
                .replace("[branch \"master\"]", &format!("[branch \"{default_branch_name}\"]"))
                .replace("merge = refs/heads/master", &["merge = refs/heads/", &default_branch_name].concat());
            
            fs::write(&config_filepath, config_txt)
                .map_err(|e| Error::Config(top_dir.to_string_lossy().into(), ErrSrc::Io(e)))?;
            
            // force refresh
            repo.config()
                .map_err(|e| Error::Config(top_dir.to_string_lossy().into(), ErrSrc::LibC(e)))?;
        }
        
        repo.find_remote(ORIGIN).unwrap().fetch(&ALL_REFSPECS, None, None)
            .map_err(|e| Error::Clone(repository.into(), ErrSrc::LibC(e), None))?;
        
        repo.set_head(&["refs/heads/", &default_branch_name].concat())
            .map_err(|e| Error::Clone(repository.into(), ErrSrc::LibC(e), None))?;
        
        Ok(Self {
            top_dir,
            working_dir,
            env,
            repo
        })
    }

    fn init_impl(top_dir: PathBuf, initial_branch_name: &str, options: Option<InitOptions>) -> Result<Self> {
        if !top_dir.is_dir() {
            fs::create_dir_all(&top_dir).map_err(|e| {
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
        
        let mut init_options = git2::RepositoryInitOptions::new();
        init_options.initial_head(initial_branch_name);
        
        let repo = git2::Repository::init_opts(&top_dir, &init_options)
            .map_err(|e| Error::Init(top_dir.clone(), ErrSrc::LibC(e), None))?;
        
        Ok(Self {
            top_dir,
            working_dir,
            env,
            repo
        })
    }

    fn init_bare_impl(top_dir: PathBuf, initial_branch_name: &str, options: Option<InitBareOptions>) -> Result<Self> {
        if !top_dir.is_dir() {
            fs::create_dir_all(&top_dir).map_err(|e|
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
        let repo = git2::Repository::init_opts(&top_dir, &git2::RepositoryInitOptions::new()
                .bare(true)
                .initial_head(initial_branch_name)
            )
            .map_err(|e| Error::Init(top_dir.clone(), ErrSrc::LibC(e), None))?;

        Ok(Self {
            top_dir,
            working_dir,
            env,
            repo
        })
    }

    fn open_impl(working_dir: PathBuf, options: Option<OpenOptions>) -> Result<Self> {
        let top_dir = stdx::fs::find_parent_dir(&working_dir, DOT_GIT)
            .ok_or_else(|| Error::Open(working_dir.clone(), ErrSrc::None, Some("Not a git repository".into())))?;

        let env = options.unwrap_or_default().env.unwrap_or_default();

        let repo = git2::Repository::open(&top_dir)
            .map_err(|e| Error::Open(top_dir.clone(), ErrSrc::LibC(e), None))?;

        Ok(Self {
            top_dir,
            working_dir,
            env,
            repo,
        })
    }
    
    fn rebase_impl(&self, options: Option<RebaseOptions>) -> Result<()> {
        let _options = match options {
            Some(o) => o.validate()?,
            None => RebaseOptions::DEFAULT
        };
        
        todo!()
    }
    
    fn reset_impl(&self, options: Option<ResetOptions>) -> Result<()> {
        let options = match options {
            Some(o) => o.validate()?,
            None => ResetOptions::DEFAULT
        };
        
        let kind = options.kind.into();        
        
        let target = if let Some(to_rev) = options.to_rev {
            self.repo.revparse_single(to_rev)
                .map_err(|e| Error::RevNotFound(to_rev.into(), ErrSrc::LibC(e), None))?
        } else {
            self.head()?.peel(git2::ObjectType::Any)
                .map_err(|e| Error::State(StateErr::HeadNotFound, ErrSrc::LibC(e)))?
        };
        
        let mut checkout = if options.kind == ResetKind::Hard {
            Some(git2::build::CheckoutBuilder::new())
        } else {
            None
        };
        
        self.repo.reset(&target, kind, checkout.as_mut())
            .map_err(|e| Error::Reset(None, ErrSrc::LibC(e), None))?;

        Ok(())
    }
}

impl GitInterfaceConstruct for GitLibC {
    fn clone(repository: &str, top_dir: PathBuf) -> Result<Self> {
        Self::clone_impl(repository, top_dir, None)
    }

    fn clone_with(repository: &str, top_dir: PathBuf, options: CloneOptions) -> Result<Self> {
        Self::clone_impl(repository, top_dir, Some(options))
    }

    fn init(top_dir: PathBuf, initial_branch_name: &str) -> Result<Self> {
        Self::init_impl(top_dir, initial_branch_name, None)
    }

    fn init_with(top_dir: PathBuf, initial_branch_name: &str, options: InitOptions) -> Result<Self> {
        Self::init_impl(top_dir, initial_branch_name, Some(options))
    }

    fn init_bare(top_dir: PathBuf, initial_branch_name: &str) -> Result<Self> {
        Self::init_bare_impl(top_dir, initial_branch_name, None)
    }

    fn init_bare_with(top_dir: PathBuf, initial_branch_name: &str, options: InitBareOptions) -> Result<Self> {
        Self::init_bare_impl(top_dir, initial_branch_name, Some(options))
    }

    fn open(working_dir: PathBuf) -> Result<Self> {
        Self::open_impl(working_dir, None)
    }

    fn open_with(working_dir: PathBuf, options: OpenOptions) -> Result<Self> {
        Self::open_impl(working_dir, Some(options))
    }
}

impl GitLibC {
    fn add_impl(&self, pathspec: &str, _options: Option<AddOptions>) -> Result<()> {
        let mut index = self.index()?;
        
        if let Some(path) = pathspec_as_path(pathspec) && path.as_os_str() != "." {
            index.add_path(path)
                .map_err(|e| Error::Add(pathspec.into(), ErrSrc::LibC(e), None))?;
        } else {
            index.add_all([pathspec].iter(), git2::IndexAddOption::DEFAULT, None)
                .map_err(|e| Error::Add(pathspec.into(), ErrSrc::LibC(e), None))?;
        }

        index.write()
            .map_err(|e| Error::Add(pathspec.into(), ErrSrc::LibC(e),
                Some("Failed to write to index".into())
            ))?;

        index.write_tree()
            .map(|_| ())
            .map_err(|e| Error::Add(pathspec.into(), ErrSrc::LibC(e),
                Some("Failed to write to index tree".into())
            ))
    }

    fn branch_create_impl(
        &self,
        branch_name: &str,
        _options: Option<BranchCreateOptions>
    ) -> Result<()> {
        let head = self.head()?;
        let commit = self.head_commit(&head)?;
       
        self.repo.branch(branch_name, &commit, false)
            .map(|_| ())
            .map_err(|e| Error::BranchCreate(HEAD.into(), branch_name.into(), ErrSrc::LibC(e)))
    }

    fn branch_current_impl(&self) -> Result<String> {
        match self.head() {
            Ok(head) => Ok(self.head_name(&head)?.into()),
            Err(_) => {
                self.repo.find_reference(HEAD)
                    .map_err(|e| Error::State(StateErr::HeadNotFound, ErrSrc::LibC(e)))?
                    .symbolic_target()
                    .ok_or_else(|| Error::State(StateErr::HeadNameNotFound, ErrSrc::None))
                    .and_then(|name| {
                        name.strip_prefix("refs/heads/")
                            .ok_or_else(|| Error::State(StateErr::UnsupportedBranchName, ErrSrc::None))
                            .map(|s| s.into())
                    })
            }
        }
    }

    fn branch_delete_impl(&self, branch_name: &str, _force: bool, _delete_remote: bool) -> Result<()> {
        let mut branch = self.repo.find_branch(branch_name, git2::BranchType::Local)
            .map_err(|e| Error::BranchNotFound(branch_name.into(), ErrSrc::LibC(e)))?;

        branch.delete()
            .map_err(|e| Error::BranchDelete(branch_name.into(), ErrSrc::LibC(e)))
    }

    fn branch_list_impl(&self, filter: Option<git2::BranchType>) -> Result<Vec<String>> {
        let branches = self.repo.branches(filter)
            .map_err(|e| Error::State(StateErr::BranchesNotFound, ErrSrc::LibC(e)))?;
        
        let mut names = vec![];
        for branch in branches {
            let branch = branch
                .map_err(|e| Error::State(StateErr::BranchesNotFound, ErrSrc::LibC(e)))?;

            let name = branch.0.name()
                .map_err(|e| Error::State(StateErr::BranchesNotFound, ErrSrc::LibC(e)))?
                .ok_or_else(|| Error::State(StateErr::BranchesNotFound, ErrSrc::None))?;

            names.push(name.into());
        }

        Ok(names)
    }

    fn branch_set_upstream_impl(&self, upstream: &Upstream<'_>) -> Result<()> {
        let head = self.head()?;
        let branch_name = self.head_name(&head)?;
        let remote_name = upstream.remote_name();
        let remote_branch_name = upstream.remote_branch_name();

        let mut config = self.repo.config()
            .map_err(|e| Error::BranchConfigure(branch_name.into(), ErrSrc::LibC(e),
                Some("Cannot access configuration".into())
            ))?;

        config.set_str(&format!("branch.{branch_name}.remote"), remote_name)
            .map_err(|e| Error::BranchConfigure(branch_name.into(), ErrSrc::LibC(e),
                Some("Cannot access configuration".into())
            ))?;

        config.set_str(&format!("branch.{branch_name}.merge"),
            &["refs/heads/", remote_branch_name].concat())
            .map_err(|e| Error::BranchConfigure(branch_name.into(), ErrSrc::LibC(e),
                Some("Cannot access configuration".into())
            ))?;
        
        Ok(())
    }

    fn commit_impl(&self, message: &str, _options: Option<CommitOptions>) -> Result<()> {
        let mut index = self.index()?;
        
        let tree_oid = index.write_tree()
            .map_err(|e| Error::Commit(ErrSrc::LibC(e),
                Some("Failed to write to index tree".into())
            ))?;
        let tree = self.repo.find_tree(tree_oid)
            .map_err(|e| Error::Commit(ErrSrc::LibC(e),
                Some("Tree OID not found after write".into())
            ))?;
        
        let is_unborn = self.head().is_err();
        let is_merging;
        let commits = if is_unborn {
            is_merging = false;
            Vec::new()
        } else {
            let mut commits = vec![self.head_commit(&self.head()?)?];
            
            is_merging = MergeModeMeta::exists(&self.top_dir())?;
            if is_merging {
                let merge_meta = MergeModeMeta::read(&self.top_dir())?;
                let oid = git2::Oid::from_str(&merge_meta.head)
                    .map_err(|e| Error::State(StateErr::InternalsFileIO, ErrSrc::LibC(e)))?;
                let commit = self.repo.find_commit(oid)
                    .map_err(|e| Error::State(StateErr::ReferenceCommitNotFound, ErrSrc::LibC(e)))?;
                
                commits.push(commit);
            }
            
            commits
        };

        let parents: Vec<&git2::Commit<'_>> = commits.iter()
            .map(|c| c)
            .collect();
        
        // CLI parity requires a trailing newline in the commit message
        let message = [message.trim(), "\n"].concat();

        self.repo.commit(
            Some(HEAD),
            &self.author_signature()?,
            &self.committer_signature()?,
            &message,
            &tree,
            &parents
        )
        .map_err(|e| Error::Commit(ErrSrc::LibC(e),
            Some("Failed to access index".into())
        ))?;
        
        if is_merging {
            self.exit_merge_mode()?;
        }
        
        Ok(())
    }

    fn diff_revision_statuses_impl(&self, rev_lhs: &str, rev_rhs: &str, _options: Option<DiffStatusOptions>) -> Result<DiffStatus> {
        let tree_lhs = self.repo.revparse_single(rev_lhs)
            .map_err(|e| Error::RevNotFound(rev_lhs.into(), ErrSrc::LibC(e), None))
            .map(|r|
                r.peel_to_tree()
                    .map_err(|e| Error::RevNotFound(rev_lhs.into(), ErrSrc::LibC(e), None))
            )??;
        let tree_rhs = self.repo.revparse_single(rev_rhs)
            .map_err(|e| Error::RevNotFound(rev_rhs.into(), ErrSrc::LibC(e), None))
            .map(|r|
                r.peel_to_tree()
                    .map_err(|e| Error::RevNotFound(rev_lhs.into(), ErrSrc::LibC(e), None))
            )??;
        
        let mut diff_options = git2::DiffOptions::new();
        diff_options.include_typechange(true);
        
        let diff = self.repo.diff_tree_to_tree(Some(&tree_lhs), Some(&tree_rhs), Some(&mut diff_options))
            .map_err(|e| Error::Diff(rev_lhs.into(), rev_rhs.into(), ErrSrc::LibC(e), None))?;
        
        let mut changes: HashMap<Arc<PathBuf>, PathDiffStatus> = HashMap::new();
        for delta in diff.deltas() {
            let status_code = match delta.status() {
                git2::Delta::Unmodified => continue,
                git2::Delta::Added => StatusCode::Added,
                git2::Delta::Deleted => StatusCode::Deleted,
                git2::Delta::Modified => StatusCode::Modified,
                git2::Delta::Renamed => StatusCode::Renamed,
                git2::Delta::Copied => StatusCode::Added,
                git2::Delta::Ignored => StatusCode::Ignored,
                git2::Delta::Untracked => StatusCode::Untracked,
                git2::Delta::Typechange => StatusCode::TypeChanged,
                git2::Delta::Conflicted => StatusCode::Unmerged,
                git2::Delta::Unreadable => return Err(Error::GitStatusParse),
            };
            
            let path = delta.new_file().path()
                .ok_or_else(|| Error::Diff(rev_lhs.into(), rev_rhs.into(), ErrSrc::None,
                    Some("Diff path unavailable".into())
                ))?
                .to_path_buf();
            
            let orig_path = if status_code == StatusCode::Renamed {
                Some(delta.old_file().path()
                    .ok_or_else(|| Error::Diff(rev_lhs.into(), rev_rhs.into(), ErrSrc::None,
                        Some("Diff original path unavailable".into())
                    ))?
                    .to_path_buf())
            } else {
                None
            };
            
            let path_diff = PathDiffStatus::new(path, status_code, orig_path);
            changes.insert(path_diff.path.clone(), path_diff);
        }
        
        Ok(DiffStatus {
            changes,
        })
    }

    fn fetch_impl(&self, _options: Option<FetchOptions>) -> Result<()> {
        let head = self.head()?;
        let branch_name = self.head_name(&head)?;
        let mut remote = if let Some(remote) = self.remote_for_branch(branch_name)? {
            remote
        } else {
            match self.repo.find_remote(ORIGIN) {
                Ok(r) => r,
                Err(e) => return Err(Error::Fetch(ErrSrc::LibC(e),
                    Some(format!("Upstream not configured for branch: {branch_name}"))
                ))
            }
        };

        let mut callbacks = git2::RemoteCallbacks::new();
        callbacks.credentials(|_, username, _| {
            git2::Cred::ssh_key_from_agent(username.unwrap_or(GIT))
        });

        let mut fetch_options = git2::FetchOptions::new();
        fetch_options.remote_callbacks(callbacks);

        // todo: differentiate between options.all
        remote.fetch(&ALL_REFSPECS, Some(&mut fetch_options), None)
            .map_err(|e| Error::Fetch(ErrSrc::LibC(e), None))
    }

    fn log_impl(&self, options: Option<LogOptions>) -> Result<Log> {
        let options = match options {
            Some(o) => o.validate()?,
            None => LogOptions::DEFAULT
        };
        
        let mut revwalk = self.repo.revwalk()
            .map_err(|e| Error::Log(ErrSrc::LibC(e), Some("Failed to walk history".into())))?;
        revwalk.push_head()
            .map_err(|e| Error::Log(ErrSrc::LibC(e), Some("Failed to walk history head".into())))?;
        //revwalk.simplify_first_parent()
        //    .map_err(|e| Error::Log(ErrSrc::LibC(e), Some("Failed to simplify first parent".into())))?;
        
        let mut commits = Vec::new();
        let mut users: HashSet<Arc<GitUser>> = HashSet::new();
        let mut signature_fingerprints = HashSet::new();
        
        for commit_oid_result in revwalk {
            let commit_oid = commit_oid_result
                .map_err(|e| Error::Log(ErrSrc::LibC(e), Some("Failed to get OID".into())))?;
            let commit = self.repo.find_commit(commit_oid)
                .map_err(|e| Error::Log(ErrSrc::LibC(e), Some(format!("Failed to find logged commit: {commit_oid}"))))?;
            
            let author = commit.author();
            let author_time = DateTime::from_timestamp(author.when().seconds(), 0)
                .expect("valid time");
            let author = GitUser::new(
                author.name().unwrap_or_default().into(),
                author.email().unwrap_or_default().into()
            );
            
            let author = if let Some(author) = users.get(&author) {
                Arc::clone(author)
            } else {
                let author = Arc::new(author);
                users.insert(Arc::clone(&author));
                author
            };
            
            let committer = commit.author();
            let committer_time = DateTime::from_timestamp(committer.when().seconds(), 0)
                .expect("valid time");
            let committer = GitUser::new(
                committer.name().unwrap_or_default().into(),
                committer.email().unwrap_or_default().into()
            );
            
            let committer = if let Some(committer) = users.get(&committer) {
                Arc::clone(committer)
            } else {
                let committer = Arc::new(committer);
                users.insert(Arc::clone(&committer));
                committer
            };
            
            let mut signature_fingerprint = None;
            let mut message = None;
            
            if options.show_signature_fingerprint {
                signature_fingerprint = match self.repo.extract_signature(&commit_oid, None) {
                    Ok((_sig, _)) => {
                        signature_fingerprints.insert(Arc::new("TODO".into()));
                        todo!()
                    },
                    Err(e) if e.code() == git2::ErrorCode::NotFound && e.class() == git2::ErrorClass::Object => {
                        None
                    },
                    Err(e) => return Err(Error::Log(ErrSrc::LibC(e),
                                Some(format!("Failed to extract signature for commit: {commit_oid}"))))
                };
            }
                
            if options.show_message {
                message = commit.message().map(|s| s.trim().into());
            }
      
            let parent_hashes = commit.parent_ids()
                .map(|oid| GitOID::try_from(oid.as_bytes()))
                .collect::<Result<_>>()?;
            
            let commit = Commit {
                commit_oid: GitOID::try_from(commit_oid.as_bytes())?,
                tree_oid: GitOID::try_from(commit.tree_id().as_bytes())?,
                parent_oid: parent_hashes,
                author,
                author_time,
                committer,
                committer_time,
                signature_fingerprint,
                message,
            };
            
            commits.push(commit);
        }
        
        Ok(Log {
            commits,
            users,
            signature_fingerprints,
        })
    }
    
    fn merge_impl(&self, source_rev: &str, options: Option<MergeOptions>) -> Result<Resolution> {
        let options = match options {
            Some(o) => o.validate()?,
            None => MergeOptions::DEFAULT
        };
        
        let mut head = self.head()?;
        let head_name = self.head_name(&head)?;
        let head_commit = head.peel_to_commit()
            .map_err(|e| Error::State(StateErr::HeadCommitNotFound, ErrSrc::LibC(e)))?;
        let source_branch = self.repo.find_branch(source_rev, git2::BranchType::Local)
            .map_err(|e| Error::BranchNotFound(source_rev.into(), ErrSrc::LibC(e)))?;
        let source_commit_oid = source_branch.get().target()
            .unwrap();
        let source_annotated = self.repo.find_annotated_commit(source_commit_oid)
            .map_err(|e| Error::RevNotFound(source_rev.into(), ErrSrc::LibC(e),
                Some("Failed to find annotated commit for revision".into())
            ))?;
        
        let (analysis, _) = self.repo.merge_analysis(&[&source_annotated])
            .unwrap();
        
        if analysis.is_up_to_date() {
            return Ok(Resolution::Unmodified);
        } else if analysis.is_fast_forward() {
            head.set_target(source_annotated.id(), REFLOG_FAST_FORWARD)
                .unwrap();
            self.repo.checkout_head(Some(git2::build::CheckoutBuilder::new().force()))
                .unwrap();
            
            return Ok(Resolution::FastForwarded);
        } else if options.fast_forward_only {
            return Err(Error::MergeAborted(source_rev.into(), ErrSrc::None,
                Some("Merge is not fast-forwardable".into())
            ));
        }
        
        let source_commit = self.repo.find_commit(source_annotated.id())
            .unwrap();
        
        let mut merge_opts = git2::MergeOptions::new();
        if options.auto_resolve_only {
            merge_opts.fail_on_conflict(true);
        }
        
        let mut index = self.repo.merge_commits(&head_commit, &source_commit, Some(&merge_opts))
            .map_err(|e| Error::MergeAborted(source_rev.into(), ErrSrc::LibC(e), None))?;
        
        self.repo.checkout_index(
            Some(&mut index),
            Some(git2::build::CheckoutBuilder::new().safe()),
        ).unwrap();
        
        let status = self.status_impl(None)
            .map_err(|e| {
                let _ = self.merge_abort_impl();
                Error::MergeAborted(source_rev.into(), ErrSrc::Lib(Box::new(e)), None)
            })?;
        
        self.enter_merge_mode(MergeModeMeta {
            mode: "".into(),
            head: source_commit.id().encode_hex(),
            msg: MergeModeMeta::format_msg(head_name, source_rev, &status), 
        })?;
        
        if !index.has_conflicts() {
            return Ok(Resolution::AutoResolved);
        }
        
        
        if let Some(conflicts) = status.into_conflicts() {
            Ok(Resolution::Unresolved(conflicts))
        } else {
            let _ = self.merge_abort_impl();
            Err(Error::State(StateErr::UnexpectedStatus, ErrSrc::None))
        }
    }
    
    fn merge_abort_impl(&self) -> Result<()> {
        let head = self.head()?;
        let head_commit = head.peel_to_commit()
            .map_err(|e| Error::State(StateErr::HeadCommitNotFound, ErrSrc::LibC(e)))?;
        
        self.repo.reset(&head_commit.as_object(), git2::ResetType::Hard,
                Some(git2::build::CheckoutBuilder::new().force())
            )
            .map_err(|e| Error::State(StateErr::AbortFailed, ErrSrc::LibC(e)))?;
        
        self.exit_merge_mode()?;
        Ok(())
    }

    fn move_rename_impl(&self, from: &Path, to: &Path) -> Result<()> {
        fs::rename(from, to)
            .map_err(|e| Error::Move(from.to_string_lossy().into_owned(),
                to.to_string_lossy().into_owned(),
                ErrSrc::Io(e),
                Some("Failed to rename path".into())
            ))?;
        
        let mut index = self.index()?;
        
        index.remove_path(from)
            .map_err(|e| Error::Move(from.to_string_lossy().into_owned(),
                to.to_string_lossy().into_owned(),
                ErrSrc::LibC(e),
                Some("Failed to remove path from index".into())
            ))?;
        
        index.add_path(to)
            .map_err(|e| Error::Move(from.to_string_lossy().into_owned(),
                to.to_string_lossy().into_owned(),
                ErrSrc::LibC(e),
                Some("Failed to add path to index".into())
            ))?;
        
        index.write()
            .map_err(|e| Error::Move(from.to_string_lossy().into_owned(),
                to.to_string_lossy().into_owned(),
                ErrSrc::LibC(e),
                None
            ))?;
        
        index.write_tree()
            .map(|_| ())
            .map_err(|e| Error::Move(from.to_string_lossy().into_owned(),
                to.to_string_lossy().into_owned(),
                ErrSrc::LibC(e),
                None
            ))
        
    }

    fn pull_impl(&self, options: Option<PullOptions>) -> Result<()> {
        let options = match options {
            Some(o) => o.validate()?,
            None => PullOptions::DEFAULT
        };
        
        let head = self.head()?;
        let head_name = self.head_name(&head)?;
        let upstream = self.try_upstream(&head)?;
        let upstream_ref = upstream.get();
        let (upstream_name, _upstream_ref_name) = self.upstream_ref_name(&upstream_ref)?;
        let mut remote = self.repo.find_remote(upstream_name.remote_name())
            .map_err(|e| Error::RemoteNotFound(upstream_name.remote_name().into(), ErrSrc::LibC(e)))?;
        
        let mut callbacks = git2::RemoteCallbacks::new();
        callbacks.credentials(|_, username, _| {
            git2::Cred::ssh_key_from_agent(username.unwrap_or(GIT))
        });
        
        let mut fetch_options = git2::FetchOptions::new();
        fetch_options.remote_callbacks(callbacks);
        
        remote.fetch(&[upstream_name.remote_branch_name()], Some(&mut fetch_options), None)
            .map_err(|e| Error::Pull(ErrSrc::LibC(e), Some("Failed to fetch".into())))?;
        
        let fetch_head = self.repo.find_reference(FETCH_HEAD)
            .unwrap();
        let fetch_commit = self.repo.reference_to_annotated_commit(&fetch_head)
            .unwrap();
        
        let (analysis, _) = self.repo.merge_analysis(&[&fetch_commit])
            .map_err(|e| Error::Pull(ErrSrc::LibC(e), Some("Failed to analyze merge".into())))?;
        
        if analysis.is_up_to_date() {
            return Ok(());
        } else if analysis.is_fast_forward() { // perform a fast-forward
            let head_ref_name = ["refs/heads/", head_name].concat();
            let mut head_reference = self.repo.find_reference(&head_ref_name)
                .unwrap();
            head_reference.set_target(fetch_commit.id(), REFLOG_FAST_FORWARD)
                .unwrap();
            self.repo.set_head(&head_ref_name)
                .unwrap();
            self.repo.checkout_head(Some(git2::build::CheckoutBuilder::new().force()))
                .unwrap();
            
            return Ok(());
        } else if options.fast_forward_only {
            return Err(Error::FastForward(PULL, head_name.into()));
        }
        
        let head_commit = &self.repo.reference_to_annotated_commit(&head)
            .unwrap();
        let committer_sig = self.committer_signature()?;
        
        if options.rebase { // perform a rebase
            let mut rebase = self.repo.rebase(
                Some(&head_commit),
                Some(&fetch_commit),
                None,
                None,
            )
            .unwrap();
        
            while let Some(op) = rebase.next() {
                let _ = op.unwrap();
                let mut index = self.index()?;
                
                if index.has_conflicts() {
                    rebase.abort()
                        .map_err(|e| Error::State(StateErr::AbortFailed, ErrSrc::LibC(e)))?;
                    
                    return Err(Error::Conflict(PULL, head_name.into()));
                }
                
                index.write_tree()
                    .unwrap();
                
                rebase.commit(None, &committer_sig, None)
                    .unwrap();
            }
        
            rebase.finish(Some(&committer_sig))
                .unwrap();
            
            self.repo.checkout_head(Some(git2::build::CheckoutBuilder::new().force()))
                .unwrap();
        } else { // perform a simple merge commit
            let ours = self.repo.find_commit(head_commit.id())
                .unwrap();
            let theirs = self.repo.find_commit(fetch_commit.id())
                .unwrap();
            
            let mut index = self.repo.merge_commits(&ours, &theirs, None)
                .unwrap();
            
            if index.has_conflicts() {
                self.repo.cleanup_state()
                    .unwrap();
                self.repo.checkout_head(Some(git2::build::CheckoutBuilder::new().force()))
                    .unwrap();
                
                return Err(Error::Conflict(PULL, head_name.into()));
            }
            
            let tree_oid = index.write_tree_to(&self.repo)
                .unwrap();
            let tree = self.repo.find_tree(tree_oid)
                .unwrap();
            
            self.repo.commit(
                Some(HEAD),
                &committer_sig,
                &committer_sig,
                "Merge commit",
                &tree,
                &[&ours, &theirs]
            )
            .unwrap();
            
            self.repo.checkout_head(Some(git2::build::CheckoutBuilder::new().force()))
                .unwrap();
        }
        
        Ok(())
    }

    fn push_impl(&self, options: Option<PushOptions>) -> Result<()> {
        let options = match options {
            Some(o) => o.validate()?,
            None => PushOptions::DEFAULT
        };
        
        let head = self.head()?;
        let branch_name = self.head_name(&head)?;
        let head_upstream = self.upstream(&head)?;

        if options.auto_set_upstream {
            let upstream = Upstream::new_borrowed(ORIGIN, branch_name);
            self.branch_set_upstream_impl(&upstream)
                .map_err(|e| Error::Push(ErrSrc::Lib(Box::new(e)),
                    Some("Cannot auto-set upstream".into())
                ))?;
        } else if let Some(upstream) = &options.set_upstream {
            self.branch_set_upstream_impl(upstream)
                .map_err(|e| Error::Push(ErrSrc::Lib(Box::new(e)),
                    Some("Cannot set upstream".into())
                ))?;
        }
        
        let (mut remote, refspec)  = match &options.upstream {
            Some(upstream) =>  {
                let refspec = ["refs/heads/", branch_name, ":refs/heads/", upstream.remote_branch_name()].concat();
                let remote = self.repo.find_remote(upstream.remote_name())
                    .map_err(|e| Error::Push(ErrSrc::LibC(e),
                        Some("Cannot find remote for ORIGIN".into())
                    ))?;

                (remote, refspec)
            },
            None => {
                if let Some(upstream) = head_upstream.as_ref() {
                    let upstream_ref_name = self.upstream_name(upstream)?;
                    let upstream_name = Upstream::from_ref_name(upstream_ref_name)?;
                    
                    let remote = self.repo.find_remote(upstream_name.remote_name())
                        .map_err(|e| Error::RemoteNotFound(upstream_name.remote_name().into(), ErrSrc::LibC(e)))?;
                    let refspec = ["refs/heads/", branch_name, ":refs/heads/", upstream_name.remote_branch_name()].concat();
                    (remote, refspec)
                } else {
                    let config = self.config()?;
                    let cfg_remote_name = self.config_entry(&config, &["branch.", branch_name, ".remote"].concat())?;
                    let cfg_remote_branch = self.config_entry(&config, &["branch.", branch_name, ".merge"].concat())?;
                    
                    if let (Some(remote_name_cfg), Some(remote_branch_ref_cfg)) = (&cfg_remote_name, &cfg_remote_branch) {
                        let remote_name = self.config_entry_str(remote_name_cfg)?;
                        let remote_branch_ref = self.config_entry_str(remote_branch_ref_cfg)?;
                        let remote = self.repo.find_remote(remote_name)
                            .map_err(|e| Error::RemoteNotFound(remote_name.into(), ErrSrc::LibC(e)))?;
                        let refspec = ["refs/heads/", branch_name, ":", remote_branch_ref].concat();
                        (remote, refspec)
                    } else {
                        return Err(Error::Push(ErrSrc::None,
                            Some("No upstream configured for current branch".into())
                        ));
                    }
                }
                
            }
        };

        let mut callbacks = git2::RemoteCallbacks::new();
        callbacks.credentials(|_, username, _| {
            let username = username.unwrap_or(GIT);
            git2::Cred::ssh_key_from_agent(username)
        });

        let mut libc_options = git2::PushOptions::new();
        libc_options.remote_callbacks(callbacks);
        
        remote.push(&[&refspec], Some(&mut libc_options))
            .map_err(|e| Error::Push(ErrSrc::LibC(e), None))?;
        
        Ok(())
    }
    
    fn show_rev_file_impl(&self, rev: &str, filepath: &Path) -> Result<String> {
        self.get_rev_file_blob(rev, filepath)
            .map(|b| String::from_utf8_lossy(b.content()).into_owned())
    }

    fn show_rev_file_bytes_impl(&self, rev: &str, filepath: &Path) -> Result<Vec<u8>> {
        self.get_rev_file_blob(rev, filepath)
            .map(|b| Vec::from(b.content()))
    }

    fn status_impl(&self, _options: Option<StatusOptions>) -> Result<Status> {
        let mut status_options = git2::StatusOptions::new();
        status_options.renames_head_to_index(true);
        status_options.include_untracked(true);
        status_options.show(git2::StatusShow::IndexAndWorkdir);
        let status = self.repo.statuses(Some(&mut status_options))
            .unwrap();
        
        let is_conflict = status.iter()
            .any(|entry| entry.status().is_conflicted());
        
        let path_statuses = status.iter()
            .map(|entry| {
                let status = entry.status();
                let path = entry.path().map(|p| Arc::new(PathBuf::from(p)))
                    .unwrap();
                
                let code_working_tree = if status.is_wt_new() {
                    Some(StatusCode::Untracked)
                } else if status.is_wt_deleted() {
                    Some(StatusCode::Deleted)
                } else if status.is_ignored() {
                    Some(StatusCode::Ignored)
                } else if status.is_wt_modified() {
                    Some(StatusCode::Modified)
                } else if status.is_wt_renamed() {
                    Some(StatusCode::Renamed)
                } else if status.is_wt_typechange() {
                    Some(StatusCode::TypeChanged)
                } else if status.is_conflicted() {
                    Some(StatusCode::Unmerged)
                } else {
                    None
                };
                
                let code_index = if status.is_index_new() {
                    Some(StatusCode::Added)
                } else if status.is_index_deleted() {
                    Some(StatusCode::Deleted)
                } else if status.is_ignored() {
                    Some(StatusCode::Ignored)
                } else if status.is_index_modified() {
                    Some(StatusCode::Modified)
                } else if status.is_index_renamed() {
                    Some(StatusCode::Renamed)
                } else if status.is_index_typechange() {
                    Some(StatusCode::TypeChanged)
                } else if status.is_conflicted() {
                    Some(StatusCode::Unmerged)
                } else if code_working_tree == Some(StatusCode::Untracked) {
                    Some(StatusCode::Untracked)
                } else {
                    None
                };
                
                let original_path = if code_working_tree == Some(StatusCode::Renamed) {
                    entry.head_to_index()
                        .and_then(|h2i| h2i.old_file().path())
                        .map(|p| PathBuf::from(p))
                } else {
                    None
                };
                
                let path_status = PathStatus::new_arc(
                    Arc::clone(&path),
                    code_index,
                    code_working_tree,
                    original_path
                );
                
                Ok((path, path_status))
            })
            .collect::<Result<HashMap<_,_>>>()?;
        
        if is_conflict {
            let map = path_statuses.into_iter()
                .map(|(path, status)| (path, PathStatus::from(status)))
                .collect::<HashMap<_,_>>();
            
            Ok(Status::new(map))
        } else {
            let map = path_statuses.into_iter()
                .map(|(path, status)| (path, PathStatus::from(status)))
                .collect::<HashMap<_,_>>();
            
            Ok(Status::new(map))
        }
    }

    fn switch_branch_impl(&self, branch_name: &str, _options: Option<SwitchBranchOptions>) -> Result<()> {
        let head = self.head()?;
        let head_name = self.head_name(&head)?;
        
        let branch = match self.repo.find_branch(branch_name, git2::BranchType::Local) {
            Ok(branch) => branch,
            Err(e) if e.code() == git2::ErrorCode::NotFound && e.class() == git2::ErrorClass::Reference => {
                let upstream = [ORIGIN, "/", branch_name].concat();
                let remote_branch = self.repo.find_branch(&upstream, git2::BranchType::Remote)
                    .map_err(|e| Error::BranchNotFound(branch_name.into(), ErrSrc::LibC(e)))?;
                let commit = remote_branch.into_reference().peel_to_commit()
                    .map_err(|e| Error::State(StateErr::ReferenceCommitNotFound, ErrSrc::LibC(e)))?;
                
                let mut branch = self.repo.branch(branch_name, &commit, false)
                    .map_err(|e| Error::BranchSwitch(branch_name.into(), ErrSrc::LibC(e), None))?;
                
                branch.set_upstream(Some(&upstream))
                    .map_err(|e| Error::BranchSwitch(branch_name.into(), ErrSrc::LibC(e), None))?;
                
                branch
            },
            Err(e) => return Err(Error::BranchNotFound(branch_name.into(), ErrSrc::LibC(e))),
        };
            
        let commit = branch.into_reference()
            .peel_to_commit()
            .map_err(|e| Error::BranchSwitch(branch_name.into(), ErrSrc::LibC(e),
                Some("Failed to peel to commit".into())
            ))?
            .into_object();
        
        self.repo.set_head(&["refs/heads/", branch_name].concat())
            .map_err(|e| Error::BranchSwitch(branch_name.into(), ErrSrc::LibC(e),
                Some("Failed to set HEAD".into())
            ))?;
        
        self.repo.checkout_tree(&commit, Some(git2::build::CheckoutBuilder::new().force()))
            .map(|_| ())
            .map_err(|e| {
                let msg = match self.repo.set_head(&head_name) {
                    Ok(_) => None,
                    Err(_) => Some("Also failed to abort".into()),
                };
                
                Error::BranchSwitch(branch_name.into(), ErrSrc::LibC(e), msg)
            })
    }
}

impl GitInterface for GitLibC {
    fn add(&self, pathspec: &str) -> Result<()> {
        self.add_impl(pathspec, None)
    }

    fn add_with(&self, pathspec: &str, options: AddOptions) -> Result<()> {
        self.add_impl(pathspec, Some(options))
    }

    fn branch_create(&self, branch_name: &str) -> Result<()> {
        self.branch_create_impl(branch_name, None)
    }

    fn branch_create_with(
        &self,
        branch_name: &str,
        options: BranchCreateOptions
    ) -> Result<()> {
        self.branch_create_impl(branch_name, Some(options))
    }

    fn branch_current(&self) -> Result<String> {
        self.branch_current_impl()
    }

    fn branch_delete(&self, branch_name: &str, force: bool, delete_remote: bool) -> Result<()> {
        self.branch_delete_impl(branch_name, force, delete_remote)
    }

    fn branch_list_local(&self) -> Result<Vec<String>> {
        self.branch_list_impl(Some(git2::BranchType::Local))
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
        self.move_rename_impl(from, to)
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
        self.show_rev_file_impl(rev, filepath)
    }

    fn show_rev_file_bytes(&self, rev: &str, filepath: &Path) -> Result<Vec<u8>> {
        self.show_rev_file_bytes_impl(rev, filepath)
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

impl From<GitOID> for git2::Oid {
    fn from(value: GitOID) -> Self {
        git2::Oid::from_bytes(value.as_bytes())
            .expect("oid")
    }
}

impl From<git2::Oid> for GitOID {
    fn from(value: git2::Oid) -> Self {
        GitOID::try_from(value.as_bytes()).expect("hash")
    }
}

impl From<ResetKind> for git2::ResetType {
    fn from(value: ResetKind) -> Self {
        match value {
            ResetKind::Default => git2::ResetType::Mixed,
            ResetKind::Soft => git2::ResetType::Soft,
            ResetKind::Hard => git2::ResetType::Hard,
        }
    }
}

impl From<git2::ResetType> for ResetKind {
    fn from(value: git2::ResetType) -> Self {
        match value {
            git2::ResetType::Soft => ResetKind::Soft,
            git2::ResetType::Mixed => ResetKind::Default,
            git2::ResetType::Hard => ResetKind::Hard,
        }
    }
}