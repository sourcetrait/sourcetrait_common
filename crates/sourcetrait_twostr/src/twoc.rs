use crate::*;

pub trait TwoStringC: Sized {
    fn try_from_ffi_cstr(ffi_cstr: &CStr) -> TwoStrResult<Self>;

    fn try_from_ffi_cstring(ffi_cstring: CString) -> TwoStrResult<Self>;
    
    fn try_into_ffi_cstring(self) -> TwoStrResult<CString>;
    
    fn try_to_ffi_cstring(&self) -> TwoStrResult<CString>;
}

pub trait TwoStrC<'s>: Sized {
    fn try_from_ffi_cstr(ffi_cstr: &'s CStr) -> TwoStrResult<Self>;
    
    fn try_into_ffi_cstring(self) -> TwoStrResult<CString>;
    
    fn try_to_ffi_cstring(&self) -> TwoStrResult<CString>;
}
