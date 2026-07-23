//! WASM bindings for tva-core.
//! Receives RGB `ImageData` frames from WebCodecs, returns JSON analysis.

use wasm_bindgen::prelude::*;
use tva_core::pipeline::{self, FrameSource, VideoMeta, PipelineConfig, NullSink};
use rgb::RGB8;

/// Analyze decoded video frames from the browser.
/// `data` — flat RGB buffer (3 bytes per pixel), `width`/`height` — frame dimensions.
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
        fn next_frame(&mut self) -> Option<Vec<RGB8>> {
            if self.index >= self.frame_count {
                return None;
            }
            let frame_size = (self.width * self.height * 3) as usize;
            let offset = self.index * frame_size;
            let raw = &self.data[offset..offset + frame_size];
            let pixels: Vec<RGB8> = raw.chunks_exact(3).map(|c| RGB8 { r: c[0], g: c[1], b: c[2] }).collect();
            self.index += 1;
            Some(pixels)
        }
    }

    let mut source = BufSource { data, width, height, frame_count, index: 0 };
    let config = PipelineConfig::default();
    let mut sink = NullSink;
    match pipeline::analyze(&mut source, &config, &mut sink) {
        Ok(report) => serde_json::to_string(&report).unwrap(),
        Err(e) => format!(r#"{{"error":"{}"}}"#, e),
    }
}

/// Return crate version.
#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
