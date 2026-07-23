use crate::compare::CompareMethod;

/// Configuration for the analysis pipeline.
#[derive(Debug, Clone)]
pub struct PipelineConfig {
    pub compare_method: CompareMethod,
    pub detect_tears: bool,
    pub tear_threshold_high: f64,
    pub tear_threshold_low: f64,
    pub detect_resolution: bool,
    pub resolution_sample_interval: u32,
    pub smooth_window: usize,
    pub smooth_polyorder: usize,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            compare_method: CompareMethod::Ssim { threshold: 0.98 },
            detect_tears: true,
            tear_threshold_high: 30.0,
            tear_threshold_low: 5.0,
            detect_resolution: false,
            resolution_sample_interval: 30,
            smooth_window: 21,
            smooth_polyorder: 3,
        }
    }
}
