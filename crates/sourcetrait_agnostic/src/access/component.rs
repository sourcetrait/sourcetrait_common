use crate::*;

pub(crate) type AccessCacheLockFn<'a> = fn() -> MutexGuard<'a, AccessCache>;

pub struct AccessComponent<'lock>(pub(crate) AccessCacheLockFn<'lock>);
impl<'lock> AccessComponent<'lock> {
    pub(crate) fn cache_lock(&self) -> MutexGuard<'lock, AccessCache> {
        (self.0)()
    }
    
    pub fn current_user(&self) -> CrossResult<User> {
        self.cache_lock().current_user()
            .map(|u| u.clone())
    }
    
    pub fn user<'a, A: Into<AccessKeyRef<'a>>>(&self, query: A) -> CrossResult<Option<User>> {
        self.cache_lock().user(query.into())
            .map(|opt| opt.cloned())
    }
    
    pub fn user_groups(&self, user: &User) -> CrossResult<Vec<UserGroup>> {
        let groups = self.cache_lock().user_groups(&user)?
            .into_iter()
            .map(|g| g.clone())
            .collect();
        
        Ok(groups)
    }
    
    pub fn group<'a, A: Into<AccessKeyRef<'a>>>(&self, query: A) -> CrossResult<Option<UserGroup>> {
        self.cache_lock().group(query.into())
            .map(|opt| opt.cloned())
    }
    
    pub fn group_users(&self, group: &UserGroup) -> CrossResult<Vec<User>> {
        let users = self.cache_lock().group_users(group)?
            .into_iter()
            .map(|u| u.clone())
            .collect();
        
        Ok(users)
    }
}