//! Decode video files into raw RGB8 frame data via ffmpeg-next.

use crate::error::{Result, TvaError};
use crate::pipeline::VideoMeta;
use rgb::RGB8;
use std::path::Path;

/// Decode a video file into RGB8 frames + metadata.
/// ponytail: stub — ffmpeg-next decode loop when implemented.
pub fn decode_file(path: &Path) -> Result<(Vec<Vec<RGB8>>, VideoMeta)> {
    let _ = path;
    Err(TvaError::Decode("not implemented yet".into()))
}
