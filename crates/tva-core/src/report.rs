use serde::{Deserialize, Serialize};
use crate::detect::TearInfo;
use crate::frame::VideoMeta;
use crate::metrics::{FrameMetric, SummaryMetrics};
use crate::resolution::ResolutionResult;

/// Schema policy:
/// - Поля НЕ удаляются и НЕ переименовываются
/// - Новые поля — Option<T> с #[serde(default)]
/// - schema_version инкрементируется только при breaking change
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Report {
    pub schema_version: u32,
    pub meta: VideoMeta,
    pub summary: SummaryMetrics,
    pub frames: Vec<FrameMetric>,
    pub fps_smoothed: Vec<f64>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tears: Option<Vec<TearInfo>>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolution: Option<Vec<ResolutionResult>>,
}
