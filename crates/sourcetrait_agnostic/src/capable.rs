use crate::*;

pub trait CapableExt: Sized {
    fn ensure_capable(&self) -> CrossResult<&Self>;
}

impl CapableExt for AccessKeyRef<'_> {
    fn ensure_capable(&self) -> CrossResult<&Self> {
        match self {
            Self::UnixID(_) => match crate::PLATFORM.capable(Capability::UnixIDs)? {
                true => Ok(self),
                false => CrossError::err_incapable(Capability::UnixIDs)
            },
            _ => Ok(self),
        }
    }
}