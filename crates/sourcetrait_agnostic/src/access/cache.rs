use crate::*;


pub(crate) struct AccessCache {
    users: HashMap<AccessKey, CacheValue<User>>,
    groups: HashMap<AccessKey, CacheValue<UserGroup>>,
    /// Self::groups[key] => Self::groups[key]
    user_groups: HashMap<AccessKey, Cached<Vec<AccessKey>>>,
    /// Self::users[key] => Self::groups[key]
    group_users: HashMap<AccessKey, Cached<Vec<AccessKey>>>,
    /// Self::users[key] => Self::groups[key]
    user_primary_groups: Capable<PrimaryUserGroupsCapable, HashMap<AccessKey, Cached<AccessKey>>>,
    /// Self::users[index] => Self::groups[value]
    current_user: Cached<AccessKey>,
    user_queries: HashMap<AccessKey, Cached<AccessKey>>,
    group_queries: HashMap<AccessKey, Cached<AccessKey>>,
    //_qualifying_authorities: Capable<HashMap<TwoString, Cached<DomainAuthority>>>,
    lookup: AccessLookup,
}

impl AccessCache {
    pub(crate) fn new(lookup: AccessLookup) -> Self {
        let capabilities = crate::PLATFORM.capabilities();
        Self {
            users: HashMap::new(),
            groups: HashMap::new(),
            group_users: HashMap::new(),
            user_groups: HashMap::new(),
            user_primary_groups: Capable::default_capable(Capability::PrimaryUserGroups, capabilities),
            current_user: Cached::None,
            user_queries: HashMap::new(),
            group_queries: HashMap::new(),
            //_qualifying_authorities: Capable::default_capable(Capability::QualifiedAccessNames, capabilities),
            lookup,
        }
    }

    /*fn qualifying_authority(&mut self, domain: PlatStr<'_>) -> CrossResult<Option<QualifyingAccessAuthority<'_>> {
        match self.qualifying_authorities.ensure_capable()?.get(&domain) {
            Some(Cached::Hit(q)) => Some(q.value().qualifying(domain).expect("domain")),
            Some(Cached::Miss(_)) => None,
            None | Some(Cached::None) => {
                self.lookup_authority_and_cache(domain)
                    .map(|opt| opt.
                ,
            },
        }
        
    }*/

    fn lookup_user_and_cache(&mut self, query: AccessKeyRef<'_>) -> CrossResult<Option<(AccessKey, &User)>> {
        /*let qualifying = match query {
            Access::QualifiedName(domain, _) => Some(self.qualifying_authority(domain)),
            _ => None,
        };*/
        
        match (self.lookup.lookup_user_fn)(query)? {
            Some(user) => {
                let key = user.to_id_key();
                let identifiers = user.to_identifiers();
                let cache_value = CacheValue::new(user);
                
                for identifier in identifiers {
                    let cached_key = Cached::hit(key.clone());
                    self.user_queries.insert(identifier, cached_key);
                }
                
                self.users.insert(key.clone(), cache_value);
                let user = self.users.get(&key)
                    .map(|cache_value| cache_value.value())
                    .expect("key_value");
                
                Ok(Some((key, user)))
            }
            None => {
                self.user_queries.insert(query.into(), Cached::miss());
                Ok(None)
            }
        }
    }

    fn lookup_group_and_cache(&mut self, query: AccessKeyRef<'_>) -> CrossResult<Option<(AccessKey, &UserGroup)>> {
        match (self.lookup.lookup_group_fn)(query)? {
            Some(group) => {
                let key = group.to_id_key();
                let identifiers = group.to_identifiers();
                let cache_value = CacheValue::new(group);
                
                for identifier in identifiers {
                    let cached_key = Cached::hit(key.clone());
                    self.group_queries.insert(identifier, cached_key);
                }
                
                self.groups.insert(key.clone(), cache_value);
                let group = self.groups.get(&key)
                    .map(|cache_value| cache_value.value())
                    .expect("key_value");
                
                Ok(Some((key, group)))
            }
            None => {
                self.group_queries.insert(query.into(), Cached::miss());
                Ok(None)
            }
        }
    }
    
    fn lookup_user_groups_and_cache(&mut self, user: &User) -> CrossResult<(AccessKey, Vec<&UserGroup>)> {
        let (user_groups, user_primary_group_key) = (self.lookup.lookup_user_groups_fn)(user)?;
        let user_key = user.to_id_key();
        
        let mut group_keys = vec![];
        for group in user_groups {
            let key = group.to_id_key();
            let identifiers = group.to_identifiers();
            let cache_value = CacheValue::new(group);
            
            for identifier in identifiers {
                let cached_key = Cached::hit(key.clone());
                self.group_queries.insert(identifier, cached_key);
            }
            
            self.groups.insert(key.clone(), cache_value);
            group_keys.push(key.clone());
        }
        
        if let Capable::Capable(primary_group_key) = user_primary_group_key && let Capable::Capable(primary_groups) = &mut self.user_primary_groups {
            primary_groups.insert(user_key.clone(), Cached::hit(primary_group_key));
        }
        
        let user_groups = group_keys.into_iter()
            .map(|group_key| self.groups.get(&group_key).expect("group").value())
            .collect();
        
        Ok((user_key, user_groups))
    }
    
    //todo: handle primary_groups for these users on lookup
    fn lookup_groups_users_and_cache(&mut self, group: &UserGroup) -> CrossResult<(AccessKey, Vec<&User>)> {
        let group_users = (self.lookup.lookup_group_users_fn)(group)?;
        let group_key = group.to_id_key();
        
        let mut user_keys = vec![];
        for user in group_users {
            let key = user.to_id_key();
            let identifiers = user.to_identifiers();
            let cache_value = CacheValue::new(user);
            
            for identifier in identifiers {
                let cached_key = Cached::hit(key.clone());
                self.user_queries.insert(identifier, cached_key);
            }
            
            self.users.insert(key.clone(), cache_value);
            user_keys.push(key.clone());
        }
        
        let group_users = user_keys.into_iter()
            .map(|user_key| self.users.get(&user_key).expect("user").value())
            .collect();
        
        Ok((group_key, group_users))
    }

    fn lookup_current_user_and_cache(&mut self) -> CrossResult<(AccessKey, &User)> {
        let user = (self.lookup.lookup_effective_process_user_fn)()?;
        let key = user.to_id_key();
        let identifiers = user.to_identifiers();
        let cache_value = CacheValue::new(user);
        
        for identifier in identifiers {
            let cached_key = Cached::hit(key.clone());
            self.user_queries.insert(identifier, cached_key);
        }
        
        self.users.insert(key.clone(), cache_value);
        let user = self.users.get(&key)
            .map(|cache_value| cache_value.value())
            .expect("key_value");
        
        Ok((key, user))
    }

    pub(crate) fn user(&mut self, query: AccessKeyRef<'_>) -> CrossResult<Option<&User>> {
        match self.user_queries.get(query.ensure_capable()?) {
            Some(Cached::Miss(_)) => Ok(None),
            Some(Cached::Hit(hit)) => Ok(Some(
                self.users.get(hit.value())
                    .expect("hit value")
                    .value()
            )),
            None | Some(Cached::None) => { 
                self.lookup_user_and_cache(query)
                    .map(|opt| opt.map(|kv| kv.1))
            },
        }
    }

    pub(crate) fn group(&mut self, query: AccessKeyRef<'_>) -> CrossResult<Option<&UserGroup>> {
        match self.group_queries.get(query.ensure_capable()?) {
            Some(Cached::Miss(_)) => Ok(None),
            Some(Cached::Hit(hit)) => Ok(Some(
                self.groups.get(hit.value())
                    .expect("hit value")
                    .value()
            )),
            None | Some(Cached::None) => { 
                self.lookup_group_and_cache(query)
                    .map(|opt| opt.map(|kv| kv.1))
            },
        }
    }
    
    pub(crate) fn user_groups(&mut self, user: &User) -> CrossResult<Vec<&UserGroup>> {
        let key = user.to_id_key();
        match self.user_groups.get(&key) {
            Some(Cached::Hit(cached)) => Ok( 
                cached.value().iter()
                    .map(|group_key| self.groups.get(group_key).expect("group").value())
                    .collect()
            ),
            Some(Cached::Miss(_)) => CrossError::err_not_found(CrossErr::User),
            None | Some(Cached::None) => {
                self.lookup_user_groups_and_cache(user)
                    .map(|kv| kv.1)
            },
        }
    }
    
    pub(crate) fn group_users(&mut self, group: &UserGroup) -> CrossResult<Vec<&User>> {
        let key = group.to_id_key();
        match self.group_users.get(&key) {
            Some(Cached::Hit(cached)) => Ok( 
                cached.value().iter()
                    .map(|user_key| self.users.get(user_key).expect("user").value())
                    .collect()
            ),
            Some(Cached::Miss(_)) => CrossError::err_not_found(CrossErr::UserGroup),
            None | Some(Cached::None) => {
                self.lookup_groups_users_and_cache(group)
                    .map(|kv| kv.1)
            },
        }
    }
    
    /*pub(crate) fn cached_current_user(&self) -> CrossResult<&User> {
        match &self.current_user {
            Cached::Hit(cached) => Ok(
                self.users.get(cached.value())
                    .expect("cached").value()
            ),
            Cached::None | Cached::Miss(_) => CrossError::err_not_found(ErrNoun::User),
        }
    }*/

    pub(crate) fn current_user(&mut self) -> CrossResult<&User> {
        match &self.current_user {
            Cached::Hit(cached) => Ok(
                self.users.get(cached.value())
                    .expect("cached").value()
            ),
            Cached::Miss(_) => CrossError::err_not_found(CrossErr::User),
            Cached::None => {
                self.lookup_current_user_and_cache()
                    .map(|kv| kv.1)
            },
        }
    }

    fn user_key(&mut self, user: &User) -> CrossResult<Option<AccessKey>> {
        let key = user.to_id_key();
        if self.users.contains_key(&key) {
            return Ok(Some(key));
        }

        self.lookup_user_and_cache(key.as_key_ref())
            .map(|opt| opt.map(|kv| kv.0))
    }

    pub(crate) fn user_primary_group(&mut self, user: &User) -> CrossResult<Capable<PrimaryUserGroupsCapable, &UserGroup>> {
        let user_key = self.user_key(user)?
            .ok_or_else(|| CrossError::not_found(CrossErr::User))?;
        
        let Capable::Capable(primary_groups) = &self.user_primary_groups else {
            return Ok(Capable::Incapable(PrimaryUserGroupsCapable));
        };
        
        match primary_groups.get(&user_key) {
            Some(Cached::Hit(cached)) => Ok(Capable::Capable(
                self.groups.get(cached.value())
                    .expect("cached")
                    .value()
            )),
            Some(Cached::Miss(_)) => CrossError::err_not_found(CrossErr::UserGroup),
            None | Some(Cached::None) => {
                let group_id = user.primary_group_id().ok_into()?;
                let group_key = group_id.as_key_ref();
                self.lookup_group_and_cache(group_key)?
                    .ok_or_else(|| CrossError::not_found(CrossErr::UserGroup))
                    .map(|kv| Capable::Capable(kv.1))
            },
        }
    }
}
