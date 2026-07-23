//! WASM bindings for tva-core.
//! Receives RGBA `ImageData` frames from WebCodecs, returns JSON analysis.

use wasm_bindgen::prelude::*;
use tva_core::pipeline::{self, FrameSource, FrameData, VideoMeta, PipelineConfig, NullSink};
use tva_core::resolution;

/// Analyze decoded video frames from the browser.
/// `data` — flat RGBA buffer (RGB, not RGBA), `width`/`height` — frame dimensions.
/// Returns JSON-serialized `Report`.
#[wasm_bindgen]
pub fn analyze_frames_wasm(
    data: Vec<u8>,
    width: u32,
    height: u32,
    frame_count: usize,
) -> String {
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
                width: self.width,
                height: self.height,
                fps: if self.frame_count > 1 { 30.0 } else { 0.0 },
                total_frames: self.frame_count as u64,
                codec: "raw".into(),
            }
        }
        fn next_frame(&mut self) -> Option<FrameData> {
            if self.index >= self.frame_count {
                return None;
            }
            let frame_size = (self.width * self.height * 3) as usize;
            let offset = self.index * frame_size;
            let data = self.data[offset..offset + frame_size].to_vec();
            let idx = self.index as u64;
            self.index += 1;
            Some(FrameData { data, index: idx })
        }
    }

    let mut source = BufSource {
        data,
        width,
        height,
        frame_count,
        index: 0,
    };
    let config = PipelineConfig::default();
    let mut sink = NullSink;
    let report = pipeline::analyze(&mut source, &config, &mut sink);
    serde_json::to_string(&report).unwrap()
}

/// Return crate version.
#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
