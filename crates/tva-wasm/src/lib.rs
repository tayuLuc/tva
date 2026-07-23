//! WASM bindings for tva-core.
//! Receives RGBA `ImageData` frames from WebCodecs, returns JSON analysis.

use wasm_bindgen::prelude::*;
use tva_core::{Frame, AnalyzeOpts};

/// Analyze decoded video frames from the browser.
/// `data` — flat RGBA buffer, `width`/`height` — frame dimensions.
/// Returns JSON-serialized `Report`.
#[wasm_bindgen]
pub fn analyze_frames_wasm(
    data: Vec<u8>,
    width: u32,
    height: u32,
    frame_count: usize,
) -> String {
    // ponytail: reconstruct frames from flat RGBA buffer, one ImageData at a time
    let frame_size = (width * height * 4) as usize;
    let frames: Vec<Frame> = (0..frame_count)
        .map(|i| {
            let offset = i * frame_size;
            let pixels = data[offset..offset + frame_size].to_vec();
            let img = image::RgbImage::from_raw(width, height, pixels)
                .expect("invalid frame data");
            Frame {
                pts: i as i64 * 33_333,
                duration: 33_333,
                image: img,
            }
        })
        .collect();

    let opts = AnalyzeOpts::default();
    let report = tva_core::analyze_frames(&frames, opts);
    serde_json::to_string(&report).unwrap()
}

/// Return crate version.
#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
