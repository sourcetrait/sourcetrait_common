use crate::*;

pub struct WindowsCrossPlatform;
impl CrossPlatform for WindowsCrossPlatform {
    const OS: Os = Os::Windows;
    const OS_FAMILY: OsFamily::Windows;
    
    fn has_command_line(&self) -> CrossResult<bool> {
        todo!("Windows is awaiting support")
    }

    fn has_graphical(&self) -> CrossResult<bool> {
        todo!("Windows is awaiting support")
    }

    fn copy_file_preserved<P1, P2>(&self, _source: P1, _dest: P2) -> CrossResult<()>
    where
        P1: AsRef<Path> + Into<std::path::PathBuf>,
        P2: AsRef<Path> + Into<std::path::PathBuf>
    {
        todo!("Windows is awaiting support")
    }

    fn access(&self) -> AccessComponent<'_> {
        AccessComponent(access_cache_lock)
    }
    
    fn run_best_editor<P>(&self, _file: P, _child_process: bool) -> CrossResult<CommandReturn>
    where
        P: AsRef<Path> + Into<PathBuf>,
    {
        todo!("Windows is awaiting support")
    }

    fn home_dir(&self) -> CrossResult<PathBuf> {
        home_dir()
    }

    fn init_dir_for<P>(&self, base: CrossDir, subdir: P) -> CrossResult<PathBuf>
    where
        P: AsRef<Path> + Into<PathBuf>
    {
        init_dir_for(base, subdir)
    }

    fn dir_for<P>(&self, base: CrossDir, subdir: P) -> CrossResult<PathBuf>
    where
        P: AsRef<Path> + Into<PathBuf>
    {
        dir_for(base, subdir)
    }

    fn init_xdg_dir_for<P>(&self, base: XdgDir, subdir: P) -> CrossResult<PathBuf>
    where
        P: AsRef<Path> + Into<PathBuf>
    {
        init_xdg_dir_for(base, subdir)
    }

    fn xdg_dir_for<P>(&self, base: XdgDir, subdir: P) -> CrossResult<PathBuf>
    where
        P: AsRef<Path> + Into<PathBuf>
    {
        xdg_dir_for(base, subdir)
    }

    fn sanitize_path<P>(&self, path: P) -> CrossResult<PathBuf>
    where
        P: Into<PathBuf>,
    {
        let path = path.into();
        Ok(path.to_string_lossy().trim_start_matches("\\\\?\\").into())
    }
    
    fn capabilities(&self) -> Capabilities {
        Capability::QualifiedAccessNames | Capability::WindowsSIDs
    }
}

fn access_cache() -> &'static Arc<Mutex<AccessCache>> {
    static CACHE: LazyLock<Arc<Mutex<AccessCache>>> = LazyLock::new(|| {
        let access_cache = AccessCache::new(
            crate::PLATFORM.capabilities(),
            lookup_user,
            lookup_group,
            lookup_process_user,
            lookup_user_groups,
            lookup_group_users,
        );
        
        Arc::new(Mutex::new(access_cache))
    });
    
    &CACHE
}

fn access_cache_lock<'lock>() -> MutexGuard<'lock, AccessCache> {
    access_cache().lock().expect("access cache")
}

fn lookup_user(query: Access<'_>) -> CrossResult<Option<User>> {
    let info = match query {
        Access::Name(s) => win_lookup_user_info(s),
        Access::QualifiedName(d, s) => win_lookup_qualified_user_info(s, d),
        Access::ID(_) => CrossError::err_incapable(Capability::AccessIDs),
        Access::SID(s) => todo!(),//win_lookup_user_sid(s),
    }?;

    Ok(info.map(UserInfo::into))
}

fn lookup_group(query: Access<'_>) -> CrossResult<Option<Group>> {
    let info = match query {
        Access::Name(s) => win_lookup_group_info(s),
        Access::QualifiedName(d, s) => win_lookup_qualified_group_info(s, d),
        Access::ID(_) => CrossError::err_incapable(Capability::AccessIDs),
        Access::SID(s) => todo!(),//win_lookup_group_sid(s),
    }?;

    Ok(info.map(GroupInfo::into))
}

fn lookup_user_groups(user: &User) -> CrossResult<(Vec<Group>, Capable<AccessKey>)> {
    let domain = user.ident.domain
        .ok_or_incapable(Capability::QualifiedAccessNames)?
        .as_ref()
        .map(PlatString::as_plat_str);

    let groups_info = match domain {
        Some(domain) => win_lookup_qualified_user_groups_info(domain, user.username())?,
        None => win_lookup_user_groups_info(user.username())?,
    };
    
    let groups_info = groups_info.into_iter().map(GroupInfo::into).collect();
    Ok((groups_info, Capable::Incapable))
}

fn lookup_group_users(group: &Group) -> CrossResult<Vec<User>> {
    let domain = group.ident.domain
        .ok_or_incapable(Capability::QualifiedAccessNames)?
        .as_ref()
        .map(PlatString::as_plat_str);

    let users_info = match domain {
        Some(domain) => win_lookup_qualified_group_users_info(domain, group.groupname())?,
        None => win_lookup_group_users_info(group.groupname())?,
    };

    let users_info = users_info.into_iter().map(UserInfo::into).collect();
    Ok(users_info)
}

fn lookup_env_username() -> CrossResult<PlatString> {
    const ENV_USERNAME: &'static str = "USERNAME";
    env::var_os(ENV_USERNAME)
        .ok_or_else(|| CrossError::env_var_none(ENV_USERNAME))
        .map(PlatString::from)
}

fn lookup_process_user() -> CrossResult<User> {
    let username = lookup_env_username()?;
    win_lookup_user_info(username.as_plat_str())?
        .ok_or_else(|| CrossError::not_found(ErrNoun::User))
        .map(UserInfo::into)
}


