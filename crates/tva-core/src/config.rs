/// Config for the analysis pipeline.
/// Compiled comparator/smoother/decoder — use adapters with features.
pub struct PipelineConfig {
    /// Порог для определения дубликата.
    /// Сравнивается с score от `FrameComparator::compare`.
    /// Если `higher_is_similar`: score > threshold → duplicate.
    /// Если !`higher_is_similar`: score < threshold → duplicate.
    pub duplicate_threshold: f64,
    pub detect_tears: bool,
    pub tear_threshold_high: f64,
    pub tear_threshold_low: f64,
    pub detect_resolution: bool,
    pub resolution_sample_interval: u32,
    pub smooth_window: usize,
    pub smooth_polyorder: usize,
    pub dismiss_tear_percentage: f64,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            duplicate_threshold: 0.98,
            detect_tears: true,
            tear_threshold_high: 30.0,
            tear_threshold_low: 5.0,
            detect_resolution: false,
            resolution_sample_interval: 30,
            smooth_window: 21,
            smooth_polyorder: 3,
            dismiss_tear_percentage: 0.5,
        }
    }
}
