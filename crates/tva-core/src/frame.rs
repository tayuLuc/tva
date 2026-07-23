use rgb::RGB8;

/// Unified video frame with pixel data and metadata.
#[derive(Debug, Clone)]
pub struct Frame {
    pub data: Vec<RGB8>,
    pub width: u32,
    pub height: u32,
    pub index: u64,
    pub timestamp_ms: f64,
}

impl Frame {
    pub fn pixel_count(&self) -> usize {
        (self.width * self.height) as usize
    }

    /// Panics in debug if data length doesn't match dimensions.
    pub fn validate(&self) {
        debug_assert_eq!(self.data.len(), self.pixel_count());
    }
}

/// Video-level metadata.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VideoMeta {
    pub fps: f64,
    pub width: u32,
    pub height: u32,
    pub total_frames: u64,
    pub duration_ms: f64,
    pub codec: String,
}
