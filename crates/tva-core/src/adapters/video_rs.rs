//! Адаптер `video-rs` → `FrameDecoder`. Feature `decode-ffmpeg`.

use crate::frame::{Frame, VideoMeta};
use crate::pixel_buffer::PixelBuffer;
use crate::traits::FrameDecoder;
use std::path::Path;
use video_rs::Locator;

pub struct VideoRsDecoder {
    inner: Option<video_rs::Video>,
    meta: VideoMeta,
    index: u64,
}

impl VideoRsDecoder {
    pub fn open(path: &Path) -> Result<Self, crate::error::TvaError> {
        let video = video_rs::Video::new(&Locator::File(path.into()))
            .map_err(|e| crate::error::TvaError::Decode(e.to_string()))?;
        let fps = video.fps();
        let total = video.frame_count() as u64;
        let duration = if fps > 0.0 { total as f64 / fps * 1000.0 } else { 0.0 };
        let meta = VideoMeta {
            fps,
            width: video.width(),
            height: video.height(),
            total_frames: total,
            duration_ms: duration,
            codec: "h264".into(),
        };
        Ok(Self { inner: Some(video), meta, index: 0 })
    }
}

impl FrameDecoder for VideoRsDecoder {
    fn metadata(&self) -> VideoMeta {
        self.meta.clone()
    }
    fn next_frame(&mut self) -> Option<Frame> {
        // ponytail: stub — full video-rs frame loop when implemented
        None
    }
}
