use crate::*;

/// Enumerates between a UTF8 [String] and a non-UTF8 [std::ffi::OsString].
/// 
/// Conversion to UTF8 is implicit upon initialization.
/// 
/// Serialization is available via [serde].
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum TwoString {
    String(String),
    OsString(OsString),
}

impl TwoString {
    #[inline]
    pub fn new_utf8(utf8: String) -> Self {
        TwoString::String(utf8)
    }
    
    pub fn try_into_ut8(self) -> TwoStrResult<String> {
        match self {
            Self::String(utf8) => Ok(utf8),
            Self::OsString(_) => Err(TwoStrError::Utf8),
        }
    }
    
    #[inline]
    pub fn new_ffi_unchecked(ffi: OsString) -> Self {
        TwoString::OsString(ffi)
    }
    
    pub fn from_ffi(ffi: OsString) -> Self {
        match ffi.into_string() {
            Ok(utf8) => Self::String(utf8),
            Err(ffi) => Self::OsString(ffi),
        }
    }
    
    pub fn from_utf8_str(utf8_str: &str) -> Self {
        Self::String(utf8_str.to_string())
    }
    
    pub fn from_ffi_str(ffi_str: &OsStr) -> Self {
        match ffi_str.to_str() {
            Some(utf8) => Self::String(utf8.to_string()),
            None => Self::OsString(ffi_str.to_os_string()),
        }
    }
    
    pub fn from_cow(cow: Cow<'_, str>) -> Self {
        match cow {
            Cow::Borrowed(utf8_str) => Self::String(utf8_str.to_string()),
            Cow::Owned(utf8) => Self::String(utf8),
        }
    }
    
    pub fn as_ffi_str(&self) -> &OsStr {
        match self {
            Self::String(utf8_ref) => OsStr::new(utf8_ref),
            Self::OsString(ffi_ref) => ffi_ref.as_os_str(),
        }
    }
    
    pub fn as_two_str<'s>(&'s self) -> TwoStr<'s> {
        match self {
            Self::String(utf8_ref) => TwoStr::Str(utf8_ref.as_str()),
            Self::OsString(ffi_ref) => TwoStr::OsStr(ffi_ref.as_os_str()),
        }
    }
}

impl From<OsString> for TwoString {
    fn from(ffi: OsString) -> Self {
        Self::from_ffi(ffi)
    }
}

impl From<&OsStr> for TwoString {
    fn from(ffi_str: &OsStr) -> Self {
        Self::from_ffi_str(ffi_str)
    }
}

impl AsRef<OsStr> for TwoString {
    fn as_ref(&self) -> &OsStr {
        self.as_ffi_str()
    }
}

impl PartialEq<str> for TwoString {
    fn eq(&self, other_utf8_str: &str) -> bool {
        match self {
            TwoString::String(utf8_ref) => utf8_ref == other_utf8_str,
            TwoString::OsString(ffi_ref) => match ffi_ref.to_str() {
                Some(utf8_str) => utf8_str == other_utf8_str,
                None => false,
            },
        }
    }
}

impl PartialEq<&str> for TwoString {
    fn eq(&self, other_utf8_str_ref: &&str) -> bool {
        self.eq(*other_utf8_str_ref)
    }
}

impl PartialEq<TwoString> for &str {
    fn eq(&self, two_string_ref: &TwoString) -> bool {
        two_string_ref.eq(self)
    }
}

impl From<TwoStr<'_>> for TwoString {
    fn from(two_str: TwoStr<'_>) -> Self {
        match two_str {
            TwoStr::Str(utf8_str) => TwoString::String(utf8_str.to_string()),
            TwoStr::OsStr(ffi_str) => TwoString::OsString(ffi_str.to_os_string()),
        }
    }
}

impl From<&TwoStr<'_>> for TwoString {
    fn from(two_str_ref: &TwoStr<'_>) -> Self {
        match two_str_ref {
            TwoStr::Str(utf8_str) => TwoString::String(utf8_str.to_string()),
            TwoStr::OsStr(ffi_str) => TwoString::OsString(ffi_str.to_os_string()),
        }
    }
}

impl AsTwoStr for TwoString {
    fn as_two_str<'a>(&'a self) -> TwoStr<'a> {
        TwoString::as_two_str(self)
    }
}

#[cfg(test)]
mod tests {
    use crate::*;
    
    const ALPHA_STR: &'static str = "alpha";
    //const BRAVO_STR: &'static str = "bravo";
    
    #[test]
    fn test_from_utf8_str() {
        let actual = TwoString::from_utf8_str(ALPHA_STR);
        assert_eq!(ALPHA_STR, actual);
    }
    
    #[test]
    fn test_as_ffi_str() {
        let actual = TwoString::from_utf8_str(ALPHA_STR);
        assert_eq!(ALPHA_STR, actual.as_ffi_str());
    }
}
