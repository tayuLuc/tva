use crate::error::Result;
use crate::frame::{Frame, VideoMeta};
use std::path::Path;

/// ponytail: stub — ffmpeg via video-rs when `decode` feature is on.
pub fn decode_file(_path: &Path) -> Result<(Vec<Frame>, VideoMeta)> {
    Err(crate::error::TvaError::Decode("not implemented — needs --features decode".into()))
}
