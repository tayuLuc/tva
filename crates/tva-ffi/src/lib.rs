#![allow(unsafe_code)]

use std::ffi::{CStr, CString};
use std::os::raw::c_char;

#[no_mangle]
pub extern "C" fn tva_analyze(path: *const c_char) -> *mut c_char {
    let _ = unsafe { CStr::from_ptr(path) };
    CString::new(r#"{"error":"not implemented"}"#).unwrap().into_raw()
}

#[no_mangle]
pub extern "C" fn tva_free_string(s: *mut c_char) {
    if !s.is_null() { unsafe { drop(CString::from_raw(s)); } }
}
