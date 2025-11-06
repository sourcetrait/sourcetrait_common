use crate::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TwoStr<'s> {
    Str(&'s str),
    OsStr(&'s OsStr),
}

impl<'s> TwoStr<'s> {
    #[inline]
    pub fn new_utf8(utf8: &'s str) -> Self {
        Self::Str(utf8)
    }
    
    #[inline]
    pub fn new_ffi(ffi: &'s OsStr) -> Self {
        Self::OsStr(ffi)
    }
    
    pub fn from_two_string_ref(two_string_ref: &'s TwoString) -> Self {
        match two_string_ref {
            TwoString::String(utf8_string_ref) => Self::Str(utf8_string_ref.as_str()),
            TwoString::OsString(ffi_string_ref) => Self::OsStr(ffi_string_ref.as_os_str()),
        }
    }
    
    pub fn as_ffi(&self) -> &OsStr {
        match self {
            Self::OsStr(ffi) => ffi,
            Self::Str(utf8) => OsStr::new(utf8),
        }
    }
    
    pub fn try_into_utf8(self) -> TwoStrResult<String> {
        match self {
            Self::Str(utf8) => Ok(utf8.to_string()),
            Self::OsStr(_) => Err(TwoStrError::Utf8),
        }
    }
}

impl<'s> From<&'s TwoString> for TwoStr<'s> {
    fn from(value: &'s TwoString) -> Self {
        Self::from_two_string_ref(value)
    }
}

impl<'a> From<&'a str> for TwoStr<'a> {
    fn from(utf8: &'a str) -> Self {
        Self::new_utf8(utf8)
    }
}

impl AsRef<OsStr> for TwoStr<'_> {
    fn as_ref(&self) -> &OsStr {
        self.as_ffi()
    }
}

impl PartialEq<str> for TwoStr<'_> {
    fn eq(&self, other_utf8: &str) -> bool {
        match *self {
            TwoStr::Str(utf8) => utf8 == other_utf8,
            TwoStr::OsStr(ffi) => ffi == other_utf8,
        }
    }
}

impl PartialEq<&str> for TwoStr<'_> {
    fn eq(&self, other_utf8_ref: &&str) -> bool {
        match self {
            TwoStr::Str(utf8_ref) => utf8_ref == other_utf8_ref,
            TwoStr::OsStr(ffi_ref) => ffi_ref == other_utf8_ref,
        }
    }
}

impl PartialEq<TwoStr<'_>> for &str {
    fn eq(&self, two_str_ref: &TwoStr<'_>) -> bool {
        TwoStr::eq(two_str_ref, self)
    }
}
