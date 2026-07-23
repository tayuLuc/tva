use crate::frame::{Frame, VideoMeta};

/// Source of video frames. Implementations: FFmpeg, WebCodecs, test data.
pub trait FrameSource {
    fn next_frame(&mut self) -> Option<Frame>;
    fn metadata(&self) -> VideoMeta;
}
