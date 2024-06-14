use asmov_common_traitenum::enumtrait;
pub use asmov_common_traitenum::EnumTrait;

/// Defines special (non-standard) treatment for a field.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Treatment {
    /// The field is treated normally.
    #[default]
    Normal,
    /// The field is represented differently in a database depending on context:
    /// - local: The DB lives on the client's system (sqlite, indexeddb, etc.)
    /// - authorative: The DB lives on a network server (postgres, mysql, etc.)
    ///
    /// On a local DB: The field is represented by two columns: 'local_id' (client) and 'id' (server / authorative).
    /// On an authorative DB: The field is represented by a single column: 'id'.
    ///
    /// Local systems will always use the 'local_id' as its primary key and use the authorative 'id' when communicating
    /// with remote systems.
    ///
    /// Authorative systems will only store and use the 'id' column. They are unaware of a client's 'local_id'.
    MutualID,
    /// The field is a primary key using a [Treatment::MutualID].
    PrimaryMutualID,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelingContext {
    Local,
    Authorative,
}

impl ModelingContext {
    pub fn is_local(&self) -> bool {
        match self { Self::Local => true, _ => false }
    }

    pub fn is_authorative(&self) -> bool {
        match self { Self::Authorative => true, _ => false }
    }
}

/// Composite pattern
#[enumtrait]
pub trait DatasetFieldEnum {
    #[enumtrait::Str(preset(Snake))]
    fn name(&self) -> &'static str;

    #[enumtrait::Num(preset(Ordinal))]
    fn ordinal(&self) -> usize;

    #[enumtrait::Enum(default(Treatment::Normal))]
    fn treatment(&self) -> Treatment;

    #[enumtrait::Str(default(""))]
    fn local_name(&self) -> &'static str;

    fn contextual_name(&self, context: ModelingContext) -> &'static str {
        match self.treatment() {
            Treatment::PrimaryMutualID | Treatment::MutualID => {
                match context {
                    ModelingContext::Local => {
                        #[cfg(debug_assertions)]
                        assert!(!self.local_name().is_empty(),
                            "Local name is empty for field name: {}", self.name());

                        self.local_name()
                    },
                    ModelingContext::Authorative => self.name(),
                }
            },
            _ => self.name(),
        }
    }

    //todo: bug: default empty OneToMany isn't working
    //#[enumtrait::Rel(nature(OneToMany), default(OneToMany::Empty))]
    //fn components(&self) -> Box<dyn Iterator<Item = Box<dyn DatasetFieldEnum>>>;
}

pub trait DatasetFieldEnumComposite: DatasetFieldEnum {
    type Meta: DatasetFieldEnum + EnumTrait;
}
