use crate::*;

pub trait NetComponentLookup {
    fn lookup_hostname(&self) -> BridgeResult<String>;
    fn lookup_domain(&self) -> BridgeResult<Capable<DomainsCapable, Option<String>>>;
    fn lookup_domain_authorities(&self) -> BridgeResult<Capable<DomainsCapable, Vec<DomainAuthority>>>;
}

