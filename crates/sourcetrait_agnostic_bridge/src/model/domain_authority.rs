use crate::*;

#[derive(Debug, Clone)]
pub struct DomainAuthority {
    server_name: String,
    domains: Vec<Arc<String>>,
}

impl DomainAuthority {
    pub fn new(server_name: String, domains: Vec<String>) -> Self {
        Self {
            server_name,
            domains: domains.into_iter().map(Arc::new).collect(),
        }
    }
    
    pub fn server_name(&self) -> &str {
        &self.server_name
    }

    pub fn domains(&self) -> &Vec<Arc<String>> {
        &self.domains
    }

    pub fn qualifies(self: &Arc<Self>, domain: &str) -> Option<QualifyingDomainAuthority> {
        self.domains.iter()
            .position(|d| d.as_ref() == domain)
            .map(|domain_index| QualifyingDomainAuthority::new(Arc::clone(self), domain_index))
    }
}

pub struct QualifyingDomainAuthority {
    authority: Arc<DomainAuthority>,
    domain_index: usize,
}

impl QualifyingDomainAuthority {
    pub(crate) fn new(authority: Arc<DomainAuthority>, domain_index: usize) -> Self {
        Self {
            authority,
            domain_index
        }
    }
    
    pub fn authority(&self) -> &DomainAuthority {
        self.authority.as_ref()
    }

    pub fn domain(&self) -> &str {
        &self.authority.domains[self.domain_index]
    }
}


