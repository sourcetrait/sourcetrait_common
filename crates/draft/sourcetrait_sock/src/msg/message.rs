use crate::*;

pub trait TlsMsg: Sized {
    const LANGUAGE: Language;
    
    #[inline]
    fn is_compatible(language: u64) -> bool {
        Self::LANGUAGE.hashcode() == language
    }
}