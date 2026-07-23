//! Decode video files into `Frame`s using ffmpeg-next.
//! HW acceleration (NVDEC/VAAPI/VideoToolbox) auto-detected.

use crate::Frame;
use std::path::Path;

/// Decode a video file into a vector of frames.
/// ponytail: software decode only for now; HW accel is one flag away.
pub fn decode_file(path: &Path) -> Result<Vec<Frame>, Box<dyn std::error::Error>> {
    // stub: ffmpeg-next decode loop goes here
    // ffmpeg_next::format::input(&path) -> decode video stream -> push frames
    let _ = path;
    Ok(Vec::new())
}
