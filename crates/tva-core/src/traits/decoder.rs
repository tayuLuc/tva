use crate::frame::{Frame, VideoMeta};

/// Trait для декодирования видео.
pub trait FrameDecoder {
    fn metadata(&self) -> VideoMeta;
    fn next_frame(&mut self) -> Option<Frame>;
}
