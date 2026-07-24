//! Native decode via ffmpeg-next (libav statically linked, no ffmpeg in PATH).
//! Feature `decode-ffmpeg-native`.
//!
//! Step 1: toolchain static linking verification. The decode body (open via
//! ffmpeg::format::input + best video stream + meta; read/decode/sws->RGB24
//! loop) is filled in the next pass by compiler error feedback -- ffmpeg-next
//! module names changed across versions and I won't reconstruct them from
//! memory after past API errors.

use std::path::Path;

use crate::error::{Result, TvaError};
use crate::frame::{Frame, VideoMeta};
use crate::traits::FrameDecoder;

pub struct FfmpegNativeDecoder {
    meta: VideoMeta,
}

impl FfmpegNativeDecoder {
    pub fn open(_path: &Path) -> Result<Self> {
        Err(TvaError::AdapterNotEnabled(
            "decode-ffmpeg-native: toolchain linked; decode body is the next pass",
        ))
    }
}

impl FrameDecoder for FfmpegNativeDecoder {
    fn metadata(&self) -> VideoMeta {
        self.meta.clone()
    }
    fn next_frame(&mut self) -> Option<Frame> {
        None
    }
}
