use serde::Serialize;

use crate::detect::TearInfo;
use crate::frame::VideoMeta;
use crate::metrics::{FrameMetric, SummaryMetrics};
use crate::resolution::ResolutionResult;

/// Complete analysis report.
#[derive(Debug, Clone, Serialize)]
pub struct Report {
    pub schema_version: u32,
    pub meta: VideoMeta,
    pub summary: SummaryMetrics,
    pub frames: Vec<FrameMetric>,
    pub fps_smoothed: Vec<f64>,
    pub tears: Vec<TearInfo>,
    pub resolution: Option<Vec<ResolutionResult>>,
}
