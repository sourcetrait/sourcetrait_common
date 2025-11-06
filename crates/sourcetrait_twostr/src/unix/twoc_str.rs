use crate::*;
use crate::unix::*;

impl<'s> TwoStrC<'s> for TwoStr<'s> {
    fn try_from_ffi_cstr(ffi_cstr: &'s CStr) -> TwoStrResult<Self> {
        Ok(match ffi_cstr.to_str() {
            Ok(utf8_str) => Self::Str(utf8_str),
            Err(_) => Self::OsStr(
                OsStr::from_bytes(ffi_cstr.to_bytes())
            ),
        })
    }
    
    fn try_into_ffi_cstring(self) -> TwoStrResult<CString> {
        let result = match self {
            TwoStr::Str(utf8_str) => CString::new(utf8_str.as_bytes()),
            TwoStr::OsStr(ffi_str) => CString::new(ffi_str.as_encoded_bytes())
        };
        
        result.map_err(|_| TwoStrError::IntoCString)
    }
    
    
    fn try_to_ffi_cstring(&self) -> TwoStrResult<CString> {
        let result = match self {
            TwoStr::Str(utf8_str) => CString::new(utf8_str.as_bytes()),
            TwoStr::OsStr(ffi_str) => CString::new(ffi_str.as_encoded_bytes())
        };
        
        result.map_err(|_| TwoStrError::IntoCString)
    }
}

impl<'s> TryFrom<&'s CStr> for TwoStr<'s> {
    type Error = TwoStrError;

    fn try_from(ffi_cstr: &'s CStr) -> Result<Self, Self::Error> {
        Self::try_from_ffi_cstr(ffi_cstr)
    }
}

impl TryFrom<TwoStr<'_>> for CString {
    type Error = TwoStrError;
    
    fn try_from(two_str: TwoStr<'_>) -> Result<Self, Self::Error> {
        two_str.try_into_ffi_cstring()
    }
}

impl TryFrom<&TwoStr<'_>> for CString {
    type Error = TwoStrError;
    
    fn try_from(two_str_ref: &TwoStr<'_>) -> Result<Self, Self::Error> {
        two_str_ref.try_to_ffi_cstring()
    }
}
