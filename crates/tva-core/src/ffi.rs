//! C FFI entry points for embedding (Python, Node, C++, etc.).
//! Use cbindgen to auto-generate `tva.h`.

use crate::{analyze, AnalyzeOpts};
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::path::Path;

/// Analyze a video file and return JSON. Caller must free with `tva_free_string`.
#[no_mangle]
pub extern "C" fn tva_analyze(path: *const c_char) -> *mut c_char {
    let c_str = unsafe { CStr::from_ptr(path) };
    let path_str = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return make_error("invalid UTF-8 path"),
    };
    let opts = AnalyzeOpts::default();
    match analyze(Path::new(path_str), opts) {
        Ok(report) => {
            let json = serde_json::to_string(&report).unwrap();
            CString::new(json).unwrap().into_raw()
        }
        Err(e) => make_error(&e.to_string()),
    }
}

/// Free a string returned by TVA.
#[no_mangle]
pub extern "C" fn tva_free_string(s: *mut c_char) {
    if !s.is_null() {
        unsafe {
            drop(CString::from_raw(s));
        }
    }
}

fn make_error(msg: &str) -> *mut c_char {
    CString::new(format!(r#"{{"error":"{}"}}"#, msg))
        .unwrap()
        .into_raw()
}
