#![forbid(unsafe_code)]

use tva_core::{
    frame::{Frame, VideoMeta},
    pixel_buffer::PixelBuffer,
    source::FrameSource,
};
use wasm_bindgen::prelude::*;

struct BufSource {
    data: Vec<u8>,
    width: u32,
    height: u32,
    frame_count: usize,
    index: usize,
}

impl FrameSource for BufSource {
    fn metadata(&self) -> VideoMeta {
        VideoMeta {
            fps: if self.frame_count > 1 { 30.0 } else { 0.0 },
            width: self.width,
            height: self.height,
            total_frames: self.frame_count as u64,
            duration_ms: self.frame_count as f64 * 33.333,
            codec: "raw".into(),
        }
    }
    fn next_frame(&mut self) -> Option<Frame> {
        if self.index >= self.frame_count {
            return None;
        }
        let fs = (self.width * self.height * 3) as usize;
        let off = self.index * fs;
        let raw = self.data[off..off + fs].to_vec();
        let buf = PixelBuffer::new(raw, self.width, self.height).ok()?;
        let idx = self.index as u64;
        self.index += 1;
        Some(Frame { data: buf, index: idx, timestamp_ms: idx as f64 * 33.333 })
    }
}

#[wasm_bindgen]
pub fn analyze_frames_wasm(data: Vec<u8>, width: u32, height: u32, frame_count: usize) -> String {
    let source = BufSource { data, width, height, frame_count, index: 0 };
    let _ = source;
    format!(r#"{{"error":"compare adapter needed"}}"#)
}

#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
