use crate::*;
use crate::unix::*;

impl TwoStringC for TwoString {
    fn try_from_ffi_cstr(ffi_cstr: &CStr) -> TwoStrResult<Self> {
        Ok(match ffi_cstr.to_str() {
            Ok(utf8_str) => Self::String(utf8_str.to_string()),
            Err(_) => Self::OsString(
                OsStr::from_bytes(ffi_cstr.to_bytes()).to_os_string()
            ),
        })
    }

    fn try_from_ffi_cstring(ffi_cstring: CString) -> TwoStrResult<Self> {
        Ok(match ffi_cstring.into_string() {
            Ok(utf8) => Self::String(utf8),
            Err(e) => Self::OsString(
                OsStr::from_bytes(e.into_cstring().as_bytes()).to_os_string()
            ),
        })
    }
    
    fn try_into_ffi_cstring(self) -> TwoStrResult<CString> {
        let result = match self {
            TwoString::String(utf8) => CString::new(utf8.as_bytes()),
            TwoString::OsString(ffi) => CString::new(ffi.as_encoded_bytes())
        };
        
        result.map_err(|_| TwoStrError::IntoCString)
    }
    
    fn try_to_ffi_cstring(&self) -> TwoStrResult<CString> {
        let result = match self {
            TwoString::String(utf8) => CString::new(utf8.as_bytes()),
            TwoString::OsString(ffi) => CString::new(ffi.as_encoded_bytes())
        };
        
        result.map_err(|_| TwoStrError::IntoCString)
    }
}

impl TryFrom<&CStr> for TwoString {
    type Error = TwoStrError;

    fn try_from(ffi_cstr: &CStr) -> Result<Self, Self::Error> {
        Self::try_from_ffi_cstr(ffi_cstr)
    }
}

impl TryFrom<CString> for TwoString {
    type Error = TwoStrError;

    fn try_from(ffi_cstring: CString) -> Result<Self, Self::Error> {
        Self::try_from_ffi_cstring(ffi_cstring)
    }
}

impl TryFrom<TwoString> for CString {
    type Error = TwoStrError;
    
    fn try_from(two_string: TwoString) -> Result<Self, Self::Error> {
        two_string.try_into_ffi_cstring()
    }
}

impl TryFrom<&TwoString> for CString {
    type Error = TwoStrError;
    
    fn try_from(two_string: &TwoString) -> Result<Self, Self::Error> {
        two_string.try_to_ffi_cstring()
    }
}
