pub mod standard;
pub mod composite;

use std::fmt::Display;
use std::fmt::Debug;
use std::hash::Hash;
use crate::*;

pub trait DesignatorKind: 'static + Debug + Display + Hash + Copy + Eq + AsRef<str> + strum::EnumCount + strum::IntoEnumIterator {
    fn name(&self) -> &str {
        self.as_ref()
    }
    
    fn filename(&self) -> &str {
        self.as_ref()
    }
}

pub trait DesignatorKindTraits<D: DesignatorTraits<Self>>: DesignatorKind {
    fn from_designator(designator: &D) -> Self;
}

pub trait Designator: 'static + Debug + Display + Hash + Clone + PartialEq + Eq + AsRef<str> {
    fn name(&self) -> &str {
        self.as_ref()
    }
    
    fn filename(&self) -> &str {
        self.as_ref()
    }
    
    fn identifier(&self) -> Option<&str>;
}

pub trait DesignatorTraits<DK: DesignatorKind>: Designator {
    fn try_from_tuple<R: 'static + DotRepoType>(tuple: DesignatorTuple<DK>) -> RepoResult<R, Self>;
}

#[derive(Debug)]
pub struct DesignatorTuple<DK: DesignatorKind>(pub DK, pub Option<String>);

