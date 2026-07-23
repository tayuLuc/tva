use crate::pixel_buffer::PixelBuffer;
use serde::Serialize;

/// Один видео-кадр. Ядро не знает о внешних типах изображений.
#[derive(Debug, Clone)]
pub struct Frame {
    pub data: PixelBuffer,
    pub index: u64,
    pub timestamp_ms: f64,
}

/// Метаданные видео.
#[derive(Debug, Clone, Serialize, serde::Deserialize)]
pub struct VideoMeta {
    pub fps: f64,
    pub width: u32,
    pub height: u32,
    pub total_frames: u64,
    pub duration_ms: f64,
    pub codec: String,
}
