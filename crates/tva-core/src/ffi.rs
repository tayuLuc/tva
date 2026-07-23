//! C FFI entry points for embedding (Python, Node, C++).
//! ponytail: stub — no-op source.

use crate::config::PipelineConfig;
use crate::events::NullSink;
use crate::frame::{Frame, VideoMeta};
use crate::pipeline;
use crate::source::FrameSource;
use rgb::RGB8;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

struct StubSource;

impl FrameSource for StubSource {
    fn metadata(&self) -> VideoMeta {
        VideoMeta { fps: 0.0, width: 0, height: 0, total_frames: 0, duration_ms: 0.0, codec: String::new() }
    }
    fn next_frame(&mut self) -> Option<Frame> {
        None
    }
}

#[no_mangle]
pub extern "C" fn tva_analyze(path: *const c_char) -> *mut c_char {
    let _ = unsafe { CStr::from_ptr(path) };
    match pipeline::analyze(&mut StubSource, &PipelineConfig::default(), &mut NullSink) {
        Ok(r) => CString::new(serde_json::to_string(&r).unwrap()).unwrap().into_raw(),
        Err(e) => CString::new(format!(r#"{{"error":"{}"}}"#, e)).unwrap().into_raw(),
    }
}

#[no_mangle]
pub extern "C" fn tva_free_string(s: *mut c_char) {
    if !s.is_null() { unsafe { drop(CString::from_raw(s)); } }
}
