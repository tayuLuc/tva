//! C FFI entry points for embedding (Python, Node, C++, etc.).
//! Use cbindgen to auto-generate `tva.h`.
//! ponytail: simplest path — wraps pipeline::analyze with a file-source.

use crate::pipeline::{self, FrameSource, FrameData, VideoMeta, PipelineConfig, NullSink};
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::path::Path;

struct FileSource {
    path: String,
    meta: VideoMeta,
    done: bool,
}

impl FrameSource for FileSource {
    fn metadata(&self) -> VideoMeta {
        self.meta.clone()
    }
    fn next_frame(&mut self) -> Option<FrameData> {
        if self.done {
            return None;
        }
        self.done = true;
        None
    }
}

/// Analyze a video file and return JSON. Caller must free with `tva_free_string`.
#[no_mangle]
pub extern "C" fn tva_analyze(path: *const c_char) -> *mut c_char {
    let c_str = unsafe { CStr::from_ptr(path) };
    let path_str = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return make_error("invalid UTF-8 path"),
    };
    let meta = VideoMeta {
        width: 0, height: 0, fps: 0.0, total_frames: 0, codec: String::new(),
    };
    let mut source = FileSource {
        path: path_str.to_string(),
        meta,
        done: false,
    };
    let config = PipelineConfig::default();
    let mut sink = NullSink;
    let report = pipeline::analyze(&mut source, &config, &mut sink);
    let json = serde_json::to_string(&report).unwrap();
    CString::new(json).unwrap().into_raw()
}

/// Free a string returned by TVA.
#[no_mangle]
pub extern "C" fn tva_free_string(s: *mut c_char) {
    if !s.is_null() {
        unsafe { drop(CString::from_raw(s)); }
    }
}

fn make_error(msg: &str) -> *mut c_char {
    CString::new(format!(r#"{{"error":"{}"}}"#, msg)).unwrap().into_raw()
}
