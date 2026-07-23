//! C FFI entry points for embedding (Python, Node, C++, etc.).
//! ponytail: wraps pipeline::analyze with a no-frame source stub.

use crate::pipeline::{self, FrameSource, VideoMeta, PipelineConfig, NullSink};
use rgb::RGB8;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

struct StubSource;
impl FrameSource for StubSource {
    fn metadata(&self) -> VideoMeta {
        VideoMeta { width: 0, height: 0, fps: 0.0, total_frames: 0, codec: String::new() }
    }
    fn next_frame(&mut self) -> Option<Vec<RGB8>> {
        None
    }
}

/// Analyze and return JSON. Caller must free with `tva_free_string`.
#[no_mangle]
pub extern "C" fn tva_analyze(path: *const c_char) -> *mut c_char {
    let c_str = unsafe { CStr::from_ptr(path) };
    let path_str = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return make_error("invalid UTF-8 path"),
    };
    let _ = path_str;
    let mut source = StubSource;
    let config = PipelineConfig::default();
    let mut sink = NullSink;
    match pipeline::analyze(&mut source, &config, &mut sink) {
        Ok(report) => {
            let json = serde_json::to_string(&report).unwrap();
            CString::new(json).unwrap().into_raw()
        }
        Err(e) => make_error(&e.to_string()),
    }
}

#[no_mangle]
pub extern "C" fn tva_free_string(s: *mut c_char) {
    if !s.is_null() {
        unsafe { drop(CString::from_raw(s)); }
    }
}

fn make_error(msg: &str) -> *mut c_char {
    CString::new(format!(r#"{{"error":"{}"}}"#, msg)).unwrap().into_raw()
}
