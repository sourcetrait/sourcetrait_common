use std::{hash::{Hash, Hasher}, fmt::{Debug, Display}};

/// Represents a value that can be one of two types; A or B.
pub enum Either<A, B> {
    A(A),
    B(B),
}

impl<A,B> Either<A, B> {
    pub fn is_a(&self) -> bool {
        matches!(self, Self::A(_))
    }
    
    pub fn is_b(&self) -> bool {
        matches!(self, Self::B(_))
    }
}

impl<A: Display, B: Display> Display for Either<A, B> {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::A(a) => { write!(f, "Either::A(")?; a.fmt(f)?; write!(f, ")") },
            Self::B(b) => { write!(f, "Either::B(")?; b.fmt(f)?; write!(f, ")") },
        }
    }
}

impl<A: Debug, B: Debug> Debug for Either<A, B> {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::A(a) => f.debug_tuple("Either::A").field(a).finish(),
            Self::B(b) => f.debug_tuple("Either::B").field(b).finish(),
        }
    }
}

impl<A: Clone, B: Clone> Clone for Either<A, B> {
    #[inline]
    fn clone(&self) -> Self {
        match self {
            Self::A(a) => Self::A(a.clone()),
            Self::B(b) => Self::B(b.clone()),
        }
    }

    #[inline]
    fn clone_from(&mut self, source: &Self) {
        match (self, source) {
            (Self::A(to), Self::A(from)) => to.clone_from(from),
            (Self::B(to), Self::B(from)) => to.clone_from(from),
            (to, from) => *to = from.clone(),
        }
    }
}

impl<A: Copy, B: Copy> Copy for Either<A, B> {}

impl<A: PartialEq, B: PartialEq> PartialEq for Either<A, B> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::A(a), Self::A(b)) => a == b,
            (Self::B(a), Self::B(b)) => a == b,
            _ => false,
        }
    }
}

impl<A: Eq, B: Eq> Eq for Either<A, B> {}

impl <A: Hash, B: Hash> Hash for Either<A, B> {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Self::A(a) => { 0.hash(state) ; a.hash(state) },
            Self::B(b) => { 1.hash(state) ; b.hash(state) },
        }
    }
}