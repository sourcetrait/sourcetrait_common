use crate::*;

pub trait AccessComponentTrait: Sized {
    fn current_user(&self) -> CrossResult<User>;
    
    fn user<'a, A: Into<AccessKeyRef<'a>>>(&self, query: A) -> CrossResult<Option<User>>;
    
    fn user_groups(&self, user: &User) -> CrossResult<Vec<UserGroup>>;
    
    //fn fetch_user<'a, A: Into<Access<'a>>>(&self, query: A) -> CrossResult<Option<MaybeShared<User, Arc<User>>>>;
    
    //fn fetch_user_groups(&self, user: &User) -> CrossResult<Vec<MaybeShared<Group, Arc<Group>>>>;
    
    fn group<'a, A: Into<AccessKeyRef<'a>>>(&self, query: A) -> CrossResult<Option<UserGroup>>;
    
    //fn fetch_group<'a, A: Into<Access<'a>>>(&self, query: A) -> CrossResult<Option<MaybeShared<Group, Arc<Group>>>>;
    
    fn group_users(&self, group: &UserGroup) -> CrossResult<Vec<User>>;
    
    //fn fetch_group_users(&self, group: &Group) -> CrossResult<Vec<MaybeShared<User, Arc<User>>>>;
    
    fn user_primary_group(&self, user: &User) -> CrossResult<Capable<PrimaryUserGroupsCapable, UserGroup>>;
}

pub(crate) struct StandardAccessComponent<L: AccessComponentLookup>(pub(crate) L);
impl<L: AccessComponentLookup> StandardAccessComponent<L> {
    fn _lookup(&self) -> &L { &self.0 }
}

impl<L: AccessComponentLookup> AccessComponentTrait for StandardAccessComponent<L> {
    fn current_user(&self) -> CrossResult<User> {
        let result = {
            let mut access_cache_lock = access_cache_lock::<L>()?;
            cache_locked_value_mut(&mut access_cache_lock)?
                .current_user()
                .map(|u| u.clone())
        };
        
        result
    }
    
    fn user<'a, A: Into<AccessKeyRef<'a>>>(&self, query: A) -> CrossResult<Option<User>> {
        let result = {
            let mut access_cache_lock = access_cache_lock::<L>()?;
            cache_locked_value_mut(&mut access_cache_lock)?
                .user(query.into())
                .map(|opt| opt.cloned())
        };
        
        result
    }
    
    fn user_groups(&self, user: &User) -> CrossResult<Vec<UserGroup>> {
        let list = {
            let mut access_cache_lock = access_cache_lock::<L>()?;
            cache_locked_value_mut(&mut access_cache_lock)?
                .user_groups(&user)?
                .into_iter()
                .map(|g| g.clone())
                .collect()
        };
        
        Ok(list)
    }
    
    fn user_primary_group(&self, user: &User) -> CrossResult<Capable<PrimaryUserGroupsCapable, UserGroup>> {
        let result = {
            let mut access_cache_lock = access_cache_lock::<L>()?;
            cache_locked_value_mut(&mut access_cache_lock)?
                .user_primary_group(user)
                .map(|cap| cap.map_into(UserGroup::clone))
        };
        
        result
    }
    
    fn group<'a, A: Into<AccessKeyRef<'a>>>(&self, query: A) -> CrossResult<Option<UserGroup>> {
        let result = {
            let mut access_cache_lock = access_cache_lock::<L>()?;
            cache_locked_value_mut(&mut access_cache_lock)?
                .group(query.into())
                .map(|opt| opt.cloned())
        };
        
        result
    }
    
    fn group_users(&self, group: &UserGroup) -> CrossResult<Vec<User>> {
        let list = {
            let mut access_cache_lock = access_cache_lock::<L>()?;
            cache_locked_value_mut(&mut access_cache_lock)?
                .group_users(&group)?
                .into_iter()
                .map(|g| g.clone())
                .collect()
        };
        
        Ok(list)
    }
}

fn access_cache<L: AccessComponentLookup>() -> &'static StaticCache<AccessCache> {
    static CACHE: OnceLock<StaticCache<AccessCache>> = OnceLock::new();
    CACHE.get_or_init(|| { 
        new_static_cache_value(AccessCache::new(L::LOOKUP))
    })
}

pub(crate) fn access_cache_lock<'lock, L: AccessComponentLookup>() -> CrossResult<StaticCacheLock<'lock, AccessCache>> {
    access_cache::<L>().lock().map_err(|_| CrossError::lock(CrossErr::AccessCache))
}
