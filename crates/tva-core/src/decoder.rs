//! Decode video files into raw frame data using ffmpeg-next.
//! HW acceleration (NVDEC/VAAPI/VideoToolbox) auto-detected.

use crate::pipeline::{FrameData, VideoMeta};
use std::path::Path;

/// Decode a video file into a vector of raw RGB frame data.
/// ponytail: software decode only for now; HW accel is one flag away.
pub fn decode_file(path: &Path) -> Result<(Vec<FrameData>, VideoMeta), Box<dyn std::error::Error>> {
    // stub: ffmpeg-next decode loop goes here
    let _ = path;
    let meta = VideoMeta {
        width: 0, height: 0, fps: 0.0, total_frames: 0, codec: String::new(),
    };
    Ok((Vec::new(), meta))
}
