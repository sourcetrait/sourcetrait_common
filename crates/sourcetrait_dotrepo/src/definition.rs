use std::{marker::PhantomData, path::Path};
use semver::Version;

use crate::*;

/// Defines how a tenant behaves
#[derive(Debug)]
pub struct Definition<DK: DesignatorKind + DesignatorKindTraits<D>, D: Designator + DesignatorTraits<DK>> {
    pub(crate) subdir: &'static str,
    pub(crate) name: &'static str,
    pub(crate) semver: (u64,u64,u64),
    pub(crate) designated_top: DesignatedDefinition,
    pub(crate) designated_module: Option<DesignatedDefinition>,
    pub(crate) designated: &'static [(DK, DesignatedDefinition)],
    pub(crate) marker: PhantomData<D>,
}

impl<DK: DesignatorKindTraits<D>, D: DesignatorTraits<DK>> Definition<DK, D> {
    pub const fn builder() -> DefinitionBuilder<DK,D> {
        DefinitionBuilder::new()
    }
    
    pub fn designated(&'static self, designated: &Designated<D>) -> &'static DesignatedDefinition {
        match designated {
            Designated::Standard(StandardDesignator::Top) => return &self.designated_top,
            Designated::Standard(StandardDesignator::Module(_)) => {
                return self.designated_module.as_ref()
                    .unwrap_or_else(|| panic!("Attempted to use designator that does not have a definition: Module"))
            },
            Designated::Tenant(designator) => {
                let designator_kind = DK::from_designator(designator);
                for (kind, def) in &*self.designated {
                    if *kind == designator_kind {
                        return def;
                    }
                }
                
                panic!("Expected tenant definition for designator kind: {designator_kind}");
            }
        }
    }
    
    pub fn tenant_path(&'static self) -> &'static Path {
        Path::new(self.subdir)
    }
    
    pub const fn tenant_path_str(&'static self) -> &'static str {
        self.subdir
    }
    
    pub const fn name(&'static self) -> &'static str {
        &self.name
    }
    
    pub const fn version(&'static self) -> Version {
        Version::new(self.semver.0, self.semver.1, self.semver.2)
    }
    
    pub const fn top_definition(&'static self) -> &'static DesignatedDefinition {
        &self.designated_top
    }
    
    pub const fn module_definition(&'static self) -> Option<&'static DesignatedDefinition> {
        self.designated_module.as_ref()
    }
    
    pub const fn tenant_definition_tuples(&'static self) -> &'static [(DK, DesignatedDefinition)] {
        &self.designated
    }
}

#[derive(Debug)]
pub struct DesignatedDefinition {
    pub(crate) excludes: bool,
    pub(crate) state: bool,
    pub(crate) local: bool,
    pub(crate) default_excludes: Option<&'static str>,
}

impl DesignatedDefinition {
    pub const fn builder() -> DesignatedDefinitionBuilder {
        DesignatedDefinitionBuilder::new()
    }
}

#[derive(Debug)]
pub struct DefinitionBuilder<DK: DesignatorKind + DesignatorKindTraits<D>, D: Designator + DesignatorTraits<DK>> {
    subdir: Option<&'static str>,
    name: Option<&'static str>,
    semver: Option<(u64,u64,u64)>,
    designated_top: Option<DesignatedDefinition>,
    designated_module: Option<DesignatedDefinition>,
    designated: Option<&'static [(DK, DesignatedDefinition)]>,
    marker: PhantomData<D>
}

impl<DK: DesignatorKind + DesignatorKindTraits<D>, D: Designator + DesignatorTraits<DK>> DefinitionBuilder<DK,D> {
    pub const fn new()-> Self {
        Self {
            subdir: None,
            name: None,
            semver: None,
            designated_top: None,
            designated_module: None,
            designated: None,
            marker: PhantomData,
        }
    }
    
    pub const fn relative_path(mut self, path: &'static str) -> Self {
        self.subdir = Some(path);
        self
    }
    
    pub const fn name(mut self, name: &'static str) -> Self {
        self.name = Some(name);
        self
    }
    
    pub const fn version(mut self, major: u64, minor: u64, patch: u64) -> Self {
        self.semver = Some((major,minor,patch));
        self
    }
    
    pub const fn top_definition(mut self, builder: DesignatedDefinitionBuilder) -> Self {
        self.designated_top = Some(builder.build());
        self
    }
    
    pub const fn module_definition(mut self, builder: DesignatedDefinitionBuilder) -> Self {
        self.designated_module = Some(builder.build());
        self
    }
    
    pub const fn tenant_definitions(mut self, tuples: &'static [(DK, DesignatedDefinition)]) -> Self {
        if tuples.len() != DK::COUNT {
            panic!("Not all designators have a definition");
        }
        
        self.designated = Some(tuples);
        self
    }
    
    pub const fn build(self) -> Definition<DK,D> {
        let designated = match self.designated {
            Some(designated) => designated,
            None => &[]
        };
        
        Definition {
            subdir: self.subdir.expect("DotRepo tenant subdir is required"),
            name: self.name.expect("DotRepo tenant name is required"),
            semver: self.semver.expect("DotRepo tenant semver is required"),
            designated_top: self.designated_top.expect("DotRepo tenant definition for Top is required"),
            designated_module: self.designated_module,
            designated,
            marker: PhantomData,
        }
    }
}

#[derive(Debug)]
pub struct DesignatedDefinitionBuilder {
    excludes: bool,
    state: bool,
    local: bool,
    default_excludes: Option<&'static str>,
}

impl DesignatedDefinitionBuilder {
    pub const fn new() -> Self {
        Self {
            excludes: false,
            state: false,
            local: false,
            default_excludes: None,
        }
    }
    
    pub const fn using_excludes(mut self) -> Self {
        self.excludes = true;
        self
    }
    
    pub const fn using_state(mut self) -> Self {
        self.state = true;
        self
    }
    
    pub const fn using_local(mut self) -> Self {
        self.local = true;
        self
    }
    
    pub const fn default_excludes(mut self, globs: &'static str) -> Self {
        self.excludes = true;
        self.default_excludes = Some(globs);
        self
    }
    
    pub const fn build(self) -> DesignatedDefinition {
        DesignatedDefinition {
            excludes: self.excludes,
            state: self.state,
            local: self.local,
            default_excludes: self.default_excludes,
        }
    }
}
