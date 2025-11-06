use std::hash::{Hash, Hasher};

use crate::*;

pub type CapableFlags = u16;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Capabilities(pub CapableFlags);

impl Capabilities {
    pub const NONE: Self = Self(0);
    
    pub fn capable(&self, capabilities: impl Into<Capabilities>) -> bool {
        *self & capabilities.into() != Self::NONE
    }
}

impl BitAnd<Capabilities> for Capabilities {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl From<Capability> for Capabilities {
    fn from(value: Capability) -> Self {
        Self(value as CapableFlags)
    }
}

impl BitOr<Capability> for Capabilities {
    type Output = Self;

    fn bitor(self, rhs: Capability) -> Self::Output {
        Self(self.0 | rhs as CapableFlags)
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum Capability {
    PrimaryUserGroups       = 0x01,
    QualifiedAccessNames    = 0x02,
    UnixIDs                 = 0x04,
    WindowsSIDs             = 0x08,
    NestedUserGroups        = 0x10,
    Domains                 = 0x20,
}

impl BitOr<Capability> for Capability {
    type Output = Capabilities;

    fn bitor(self, rhs: Capability) -> Self::Output {
        Capabilities(self as CapableFlags | rhs as CapableFlags)
    }
}

pub enum Capable<C: CapableType, T> {
    Unknown,
    Incapable(C),
    Capable(T),
}

impl<C: CapableType> Capable<C, Option<TwoString>> {
    pub fn as_deref(&self) -> Capable<C, Option<TwoStr<'_>>> {
        match self {
            Capable::Unknown => Capable::Unknown,
            Capable::Incapable(c) => Capable::Incapable(*c),
            Capable::Capable(opt) => Capable::Capable(opt.as_ref().map(|s| s.as_two_str())),
        }
    }
}

impl<C: CapableType> Capable<C, StringSID> {
    pub const fn as_deref(&self) -> Capable<C, &StrSID> {
        match self {
            Capable::Unknown => Capable::Unknown,
            Capable::Incapable(c) => Capable::Incapable(*c),
            Capable::Capable(str) => Capable::Capable(str.as_str()),
        }
    }
}

impl<C: CapableType, T> Capable<C, T> {
    pub fn default_capable<C1: Into<Capabilities>, C2: Into<Capabilities>>(capability: C1, capabilities: C2) -> Self
    where
        T: Default
    {
        if capabilities.into().capable(capability) {
            Self::Capable(T::default())
        } else {
            Self::Incapable(C::SELF)
        }
    }
    
    pub const fn as_ref(&self) -> Capable<C, &T> {
        match *self {
            Capable::Unknown => Capable::Unknown,
            Capable::Incapable(c) => Capable::Incapable(c),
            Capable::Capable(ref x) => Capable::Capable(x),
        }
    }
    
    pub const fn is_capable(&self) -> bool {
        matches!(self, Capable::Capable(_))
    }
    
    #[track_caller]
    pub fn expect(self, s: &'static str) -> T {
        match self {
            Capable::Capable(x) => x,
            Capable::Unknown => std::panic::panic_any(s),
            Capable::Incapable(_) => std::panic::panic_any(s),
        }
    }
    
    pub fn is_capable_and<F: FnOnce(&T) -> bool>(&self, func: F) -> bool {
        match self {
            Self::Capable(x) => (func)(x),
            _ => false,
        }
    }
    
    pub fn capable_and_then<F, UC, UT>(&self, func: F) -> Capable<UC, UT>
    where
        UC: CapableType,
        F: FnOnce(&T) -> Capable<UC, UT>,
    {
        match self {
            Self::Capable(x) => (func)(x),
            Self::Incapable(_) => Capable::Incapable(UC::SELF),
            Self::Unknown => Capable::Unknown,
        }
    }
    
    pub fn some(&self) -> Option<&T> {
        match self {
            Self::Capable(t) => Some(t),
            Self::Unknown => None,
            Self::Incapable(_) => None,
        }
    }
    
    pub fn ok(&self) -> BridgeResult<&T> {
        match self {
            Self::Capable(t) => Ok(t),
            Self::Unknown => BridgeError::err_incapable(C::CAPABILITY),
            Self::Incapable(_) => BridgeError::err_incapable(C::CAPABILITY),
        }
    }
    
    pub fn ok_into(self) -> BridgeResult<T> {
        match self {
            Self::Capable(t) => Ok(t),
            Self::Unknown => BridgeError::err_incapable(C::CAPABILITY),
            Self::Incapable(_) => BridgeError::err_incapable(C::CAPABILITY),
        }
    }

    pub fn map_into<U, F>(self, f: F) -> Capable<C, U>
    where
        F: FnOnce(T) -> U,
    {
        match self {
            Self::Capable(t) => Capable::Capable(f(t)),
            Self::Incapable(c) => Capable::Incapable(c),
            Self::Unknown => Capable::Unknown,
        }
    }
    
    pub fn map<U, F>(&self, f: F) -> Capable<C, U>
    where
        F: FnOnce(&T) -> U,
    {
        match self {
            Self::Capable(t) => Capable::Capable(f(t)),
            Self::Incapable(c) => Capable::Incapable(*c),
            Self::Unknown => Capable::Unknown,
        }
    }

}

impl<C: CapableType, T, E> Capable<C, Result<T, E>> {
    pub fn transpose(self) -> Result<Capable<C, T>, E> {
        match self {
            Self::Capable(Ok(t)) => Ok(Capable::Capable(t)),
            Self::Capable(Err(e)) => Err(e),
            Self::Incapable(c) => Ok(Capable::Incapable(c)),
            Self::Unknown => Ok(Capable::Unknown),
        }
    }
}

impl<C: CapableType, T: Debug> Debug for Capable<C, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unknown => write!(f, "Unknown"),
            Self::Incapable(_) => f.debug_tuple("Incapable").field(&C::CAPABILITY).finish(),
            Self::Capable(t) => f.debug_tuple("Capable").field(t).finish(),
        }
    }
}

impl<C: CapableType, T: Clone> Clone for Capable<C, T> {
    fn clone(&self) -> Self {
        match self {
            Capable::Unknown => Capable::Unknown,
            Capable::Incapable(c) => Capable::Incapable(*c),
            Capable::Capable(x) => Capable::Capable(x.clone()),
        }
    }
}

impl<C: CapableType, T: Copy> Copy for Capable<C, T> {}

impl<C: CapableType, T: PartialEq> PartialEq for Capable<C, T> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Capable::Unknown, Capable::Unknown) => true,
            (Capable::Incapable(_), Capable::Incapable(_)) => true,
            (Capable::Capable(l), Capable::Capable(r)) => l == r,
            _ => false,
        }
    }
}

impl<C: CapableType, T: Eq> Eq for Capable<C, T> {}

impl<C: CapableType, T: Hash> Hash for Capable<C, T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Self::Unknown => 0.hash(state),
            Self::Incapable(_) => 1.hash(state),
            Self::Capable(x) => {
                2.hash(state);
                x.hash(state);
            },
        }
    }
}

pub trait CapableType: Sized + Clone + Copy + PartialEq + Eq {
    const SELF: Self;
    const CAPABILITY: Capability;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnixIDsCapable;
impl CapableType for UnixIDsCapable {
    const SELF: Self = Self;
    const CAPABILITY: Capability = Capability::UnixIDs;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PrimaryUserGroupsCapable;
impl CapableType for PrimaryUserGroupsCapable {
    const SELF: Self = Self;
    const CAPABILITY: Capability = Capability::PrimaryUserGroups;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DomainsCapable;
impl CapableType for DomainsCapable {
    const SELF: Self = Self;
    const CAPABILITY: Capability = Capability::Domains;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindowsSIDsCapable;
impl CapableType for WindowsSIDsCapable {
    const SELF: Self = Self;
    const CAPABILITY: Capability = Capability::WindowsSIDs;
}

impl<C: CapableType, T> AsRef<Capable<C, T>> for Capable<C, T> {
    fn as_ref(&self) -> &Capable<C, T> {
        self
    }
}