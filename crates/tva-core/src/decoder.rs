//! FFmpeg decoder — reads video frames into `Frame`.

use crate::error::{Result, TvaError};
use crate::frame::{Frame, VideoMeta};
use std::path::Path;

/// Decode a video file into frames + metadata.
/// ponytail: stub — ffmpeg-next decode loop when implemented.
pub fn decode_file(_path: &Path) -> Result<(Vec<Frame>, VideoMeta)> {
    Err(TvaError::Decode("not implemented yet".into()))
}
