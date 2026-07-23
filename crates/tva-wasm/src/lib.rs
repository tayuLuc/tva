#![forbid(unsafe_code)]

use wasm_bindgen::prelude::*;
use tva_core::{
    config::PipelineConfig,
    events::NullSink,
    frame::{Frame, VideoMeta},
    pipeline,
    source::FrameSource,
};
use rgb::RGB8;

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
        let frame_size = (self.width * self.height * 3) as usize;
        let offset = self.index * frame_size;
        let raw = &self.data[offset..offset + frame_size];
        let pixels: Vec<RGB8> = raw.chunks_exact(3)
            .map(|c| RGB8 { r: c[0], g: c[1], b: c[2] })
            .collect();
        let idx = self.index as u64;
        self.index += 1;
        Some(Frame {
            data: pixels,
            width: self.width,
            height: self.height,
            index: idx,
            timestamp_ms: idx as f64 * 33.333,
        })
    }
}

#[wasm_bindgen]
pub fn analyze_frames_wasm(
    data: Vec<u8>,
    width: u32,
    height: u32,
    frame_count: usize,
) -> String {
    let mut source = BufSource { data, width, height, frame_count, index: 0 };
    match pipeline::analyze(&mut source, &PipelineConfig::default(), &mut NullSink) {
        Ok(report) => serde_json::to_string(&report).unwrap(),
        Err(e) => format!(r#"{{"error":"{}"}}"#, e),
    }
}

#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
