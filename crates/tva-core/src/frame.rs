use image::DynamicImage;
use serde::Serialize;

/// A single decoded video frame.
#[derive(Debug, Clone)]
pub struct Frame {
    pub data: DynamicImage,
    pub index: u64,
    pub timestamp_ms: f64,
}

impl Frame {
    #[must_use]
    pub fn width(&self) -> u32 {
        self.data.width()
    }

    #[must_use]
    pub fn height(&self) -> u32 {
        self.data.height()
    }

    #[must_use]
    pub fn pixel_count(&self) -> usize {
        (self.width() * self.height()) as usize
    }
}

/// Video metadata from the container.
#[derive(Debug, Clone, Serialize)]
pub struct VideoMeta {
    pub fps: f64,
    pub width: u32,
    pub height: u32,
    pub total_frames: u64,
    pub duration_ms: f64,
    pub codec: String,
}
