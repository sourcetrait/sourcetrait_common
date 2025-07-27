use std::fmt::Display;

/// Represents a value that can be one of two types; A or B.
#[derive(Eq, Debug, Hash)]
pub enum Either<A, B> {
    A(A),
    B(B),
}

impl<A: Display, B: Display> Display for Either<A, B> {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::A(a) => a.fmt(f),
            Self::B(b) => b.fmt(f),
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