use crate::*;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, strum::Display, strum::AsRefStr, strum::EnumCount, strum::EnumIter)]
#[strum(serialize_all = "kebab-case")]
pub enum StandardDesignatorKind {
    Top,
    Module,
}

impl DesignatorKind for StandardDesignatorKind {}

impl DesignatorKindTraits<StandardDesignator> for StandardDesignatorKind {
    fn from_designator(value: &StandardDesignator) -> Self {
        match value {
            StandardDesignator::Top => StandardDesignatorKind::Top,
            StandardDesignator::Module(_) => StandardDesignatorKind::Module,
        }
    }
}

/// Top is required. Module is optional.
#[derive(Debug, Clone, PartialEq, Eq, Hash, strum::Display, strum::AsRefStr)]
#[strum(serialize_all = "kebab-case")]
pub enum StandardDesignator {
    /// Required.
    /// No ancestor on the filesystem should exist with the same designator.
    /// No children on the filesystem should exist with the same designator. 
    Top,
    /// Exactly one [Designator::Top] should exist as an ancestor.
    /// May have multiple ancestors and children, in a heirarchy, with the same
    /// designator.
    Module(Option<String>),
}

impl Designator for StandardDesignator {
    fn identifier(&self) -> Option<&str> {
        match self {
            Self::Top => None,
            Self::Module(identifier) => identifier.as_deref(),
        }
    }
}

impl DesignatorTraits<StandardDesignatorKind> for StandardDesignator {
    fn try_from_tuple<R: 'static + DotRepoType>(tuple: DesignatorTuple<StandardDesignatorKind>) -> RepoResult<R, Self> {
        Ok(match tuple.0 {
            StandardDesignatorKind::Top => Self::Top,
            StandardDesignatorKind::Module => Self::Module(tuple.1),
        })
    }
}

