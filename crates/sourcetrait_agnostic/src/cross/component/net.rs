use crate::*;

pub trait NetComponentTrait {
    fn hostname(&self) -> CrossResult<Arc<String>>;
    fn primary_domain(&self) -> CrossResult<Capable<DomainsCapable, Option<Arc<String>>>>;
    fn domain_authorities(&self) -> CrossResult<Capable<DomainsCapable, Vec<Arc<DomainAuthority>>>>;
    fn qualifying_authority(&self, domain: &str) -> CrossResult<Option<QualifyingDomainAuthority>>;
}

#[allow(private_bounds)]
pub struct StandardNetComponent<LOOKUP: NetComponentLookup>(pub(crate) LOOKUP);

#[allow(private_bounds)]
impl<LOOKUP: NetComponentLookup> StandardNetComponent<LOOKUP> {
    pub(crate) fn lookup(&self) -> &LOOKUP { &self.0 }
}

impl<LOOKUP: NetComponentLookup> NetComponentTrait for StandardNetComponent<LOOKUP> {
    fn hostname(&self) -> CrossResult<Arc<String>> {
        let result = {
            let mut net_cache_lock = net_cache_lock()?;
            cache_locked_value_mut(&mut net_cache_lock)?
                .hostname
                .determine(|| {
                    self.lookup().lookup_hostname()
                        .map(Arc::new)
                        .map_err(CrossError::from)
                })
                .map(Arc::clone)
        };
        
        result
    }

    /// The primary domain name that this machine is joined to, if capable.
    /// 
    /// Currently supported domain controllers:
    /// - Windows Active Directory
    /// - RedHat FreeIPA
    /// 
    /// Currently supported client configurations:
    /// - Linux: configuration parsing of optional NSS + (SSSD or Samba WinBind)
    /// - Windows: out-of-the-box via windows-sys API
    /// - MacOS: out-of-the-box via objc2-open-directory API
    fn primary_domain(&self) -> CrossResult<Capable<DomainsCapable, Option<Arc<String>>>> {
        let result = {
            let mut net_cache_lock = net_cache_lock()?;
            cache_locked_value_mut(&mut net_cache_lock)?
                .primary_domain
                .determine(|| {
                    self.lookup().lookup_domain()
                        .map(|cap| cap.map_into(|opt| opt.map(Arc::new)))
                        .map_err(CrossError::from)
                })
                .map(|cap| {
                    cap.as_ref()
                        .map_into(|opt| opt.as_ref().map(Arc::clone))
                })
        };
        
        result
    }
    
    
    fn domain_authorities(&self) -> CrossResult<Capable<DomainsCapable, Vec<Arc<DomainAuthority>>>> {
        let result = {
            let mut net_cache_lock = net_cache_lock()?;
            cache_locked_value_mut(&mut net_cache_lock)?
                .domain_authorities
                .determine(|| {
                    let authorities = self.lookup().lookup_domain_authorities()?
                        .map_into(|vec| vec 
                            .into_iter()
                            .map(Arc::new)
                            .collect()
                        );
                    
                    Ok(authorities)
                })
                .map(|vec| vec.map(|vec| vec.iter().map(Arc::clone).collect()))
        };
        
        result
    }
    
    fn qualifying_authority(&self, domain: &str) -> CrossResult<Option<QualifyingDomainAuthority>> {
        let found = {
            let mut net_cache_lock = net_cache_lock()?;
            cache_locked_value_mut(&mut net_cache_lock)?
                .domain_authorities
                .determine(|| {
                    let authorities = self.lookup().lookup_domain_authorities()?
                        .map_into(|vec| vec
                            .into_iter()
                            .map(Arc::new)
                            .collect()
                        );
                    
                    Ok(authorities)
                })?
                .ok()?
                .iter()
                .find_map(|authority| authority.qualifies(domain))
        };
        
        Ok(found)
    }
}

pub(crate) struct NetCache {
    hostname: CacheDetermined<Arc<String>>,
    primary_domain: CacheDetermined<Capable<DomainsCapable, Option<Arc<String>>>>,
    domain_authorities: CacheDetermined<Capable<DomainsCapable, Vec<Arc<DomainAuthority>>>>,
}

impl NetCache {
    pub(crate) const fn default_const() -> Self {
        Self {
            hostname: None,
            primary_domain: None,
            domain_authorities: None,
        }
    }
}

fn net_cache() -> &'static StaticCache<NetCache> {
    static CACHE: LazyLock<StaticCache<NetCache>> = LazyLock::new(|| { 
        new_static_cache_value(NetCache::default_const())
    });
    
    &CACHE
}

pub(crate) fn net_cache_lock<'lock>() -> CrossResult<StaticCacheLock<'lock, NetCache>> {
    net_cache().lock().map_err(|_| CrossError::lock(CrossErr::NetCache))
}

