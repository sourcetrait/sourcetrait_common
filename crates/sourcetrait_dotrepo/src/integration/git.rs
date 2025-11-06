use crate::*;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, strum::Display, strum::AsRefStr, strum::EnumCount, strum::EnumIter)]
#[strum(serialize_all = "kebab-case")]
pub enum GitDesignatorKind {
    Top,
    Submodule,
}

impl GitDesignatorKind {
    pub fn to_integrated<R: 'static + GitIntegratedType>(&self) -> R::DesignatorKind {
        match self {
            Self::Top => R::DESIGNATOR_KIND_TOP,
            Self::Submodule => R::DESIGNATOR_KIND_SUBMODULE,
        }
    }
    
    pub fn try_from_integrated<R: 'static + GitIntegratedType>(&self) -> RepoResult<R, R::DesignatorKind> {
        match self {
            Self::Top => Ok(R::DESIGNATOR_KIND_TOP),
            Self::Submodule => Ok(R::DESIGNATOR_KIND_SUBMODULE),
        }
    }
}

pub trait GitIntegratedType: DotRepoType {
    /// GitTop
    const DESIGNATOR_KIND_TOP: Self::DesignatorKind;
    /// GitSubmodule
    const DESIGNATOR_KIND_SUBMODULE: Self::DesignatorKind;
}

#[derive(Debug)]
pub struct GitIntegration<'a, R: 'static + GitIntegratedType> {
    dot_repo: &'a DotRepo<R>,
}

impl<'a, R: 'static + GitIntegratedType> GitIntegration<'a, R> {
    pub fn new(dot_repo: &'a DotRepo<R>) -> RepoResult<R, Self> {
        Ok(Self {
            dot_repo
        })
    }
}

