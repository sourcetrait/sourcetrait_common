use std::collections::HashSet;
use crate::*;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, strum::Display, strum::AsRefStr, strum::EnumCount, strum::EnumIter)]
#[strum(serialize_all = "kebab-case")]
pub enum DesignatedKind {
    Standard,
    Tenant
}

impl DesignatorKind for DesignatedKind {}

#[derive(Debug, Clone, PartialEq, Eq, Hash, strum::Display, strum::AsRefStr)]
#[strum(serialize_all = "kebab-case")]
pub enum Designated<D: Designator> {
    #[strum(transparent)]
    Standard(StandardDesignator),
    #[strum(transparent)]
    Tenant(D)
}

impl<D: Designator> Designator for Designated<D> {
    fn identifier(&self) -> Option<&str> {
        match self {
            Self::Standard(designator) => designator.identifier(),
            Self::Tenant(designator) => designator.identifier(),
        }
    }
}

impl<D: Designator> AsRef<Designated<D>> for Designated<D> {
    fn as_ref(&self) -> &Designated<D> {
        &self
    }
}

#[derive(Debug)]
pub struct DesignatedTuple<DK: DesignatorKind>(pub DesignatedKind, pub Option<StandardDesignatorKind>, pub Option<DK>, pub Option<String>);

/*impl<DK: DesignatorKind, D: Designator + TryFrom<DesignatorTuple<DK>>> TryFrom<DesignatedTuple<DK>> for Designated<D>
where
    <D as TryFrom<designator::DesignatorTuple<DK>>>::Error: std::fmt::Debug
{
    type Error = crate::RepoError;

    fn try_from(value: DesignatedTuple<DK>) -> std::result::Result<Self, Self::Error> {
        let DesignatedTuple(tuple_kind, std_kind, tenant_kind, ident) = value;
        let designator = match (tuple_kind, std_kind, tenant_kind) {
            (DesignatedKind::Standard, Some(std_kind), None) => Self::Standard(
                StandardDesignator::try_from_tuple(DesignatorTuple(std_kind, ident)).expect("try_from")
            ),
            (DesignatedKind::Tenant, None, Some(tenant_kind)) => Self::Tenant(
                D::try_from(DesignatorTuple(tenant_kind, ident)).expect("try_from")
            ),
            _ => panic!("Invalid designated tuple"),
        };
        
        Ok(designator)
    }
}*/

#[derive(Debug, Clone, PartialEq, derive_builder::Builder)]
#[builder(default)]
pub struct DesignatorMatches<R: 'static + DotRepoType> {
    standard_kind: HashSet<StandardDesignatorKind>,
    standard_designator: HashSet<StandardDesignator>,
    tenant_kind: HashSet<R::DesignatorKind>,
    tenant_designator: HashSet<R::Designator>
}

impl<R: 'static + DotRepoType> Default for DesignatorMatches<R> {
    fn default() -> Self {
        Self {
            standard_kind: HashSet::new(),
            standard_designator: HashSet::new(),
            tenant_kind: HashSet::new(),
            tenant_designator: HashSet::new(),
        }
    }
}

impl<R: 'static + DotRepoType> DesignatorMatches<R> {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn builder() -> DesignatorMatchesBuilder<R> {
        DesignatorMatchesBuilder::default()
    }
    
    pub fn new_from(
        standard_kind: HashSet<StandardDesignatorKind>,
        standard_designator: HashSet<StandardDesignator>,
        tenant_kind: HashSet<R::DesignatorKind>,
        tenant_designator: HashSet<R::Designator>,
    ) -> Self {
        Self {
            standard_kind,
            standard_designator,
            tenant_kind,
            tenant_designator,
        }
    }
    
    pub fn standard_kind(&self) -> &HashSet<StandardDesignatorKind> {
        &self.standard_kind
    }
    
    pub fn standard_designator(&self) -> &HashSet<StandardDesignator> {
        &self.standard_designator
    }
    
    pub fn tenant_kind(&self) -> &HashSet<R::DesignatorKind> {
        &self.tenant_kind
    }
    
    pub fn tenant_designator(&self) -> &HashSet<R::Designator> {
        &self.tenant_designator
    }
}

impl<R: 'static + DotRepoType> DesignatorMatchesBuilder<R> {
    pub fn insert_standard_kind(&mut self, kind: StandardDesignatorKind) -> &mut Self {
        if self.standard_kind.is_none() {
            self.standard_kind = Some(HashSet::new());
        }
        
        self.standard_kind.as_mut().expect("exist").insert(kind);
        self
    }
    
    pub fn insert_tenant_kind(&mut self, kind: R::DesignatorKind) -> &mut Self {
        if self.tenant_kind.is_none() {
            self.tenant_kind = Some(HashSet::new());
        }
        
        self.tenant_kind.as_mut().expect("exist").insert(kind);
        self
    }
    
    pub fn insert_standard_designator(&mut self, designator: StandardDesignator) -> &mut Self {
        if self.standard_designator.is_none() {
            self.standard_designator = Some(HashSet::new());
        }
        
        self.standard_designator.as_mut().expect("exist").insert(designator);
        self
    }
    
    pub fn insert_tenant_designator(&mut self, designator: R::Designator) -> &mut Self {
        if self.tenant_designator.is_none() {
            self.tenant_designator = Some(HashSet::new());
        }
        
        self.tenant_designator.as_mut().expect("exist").insert(designator);
        self
    }
}