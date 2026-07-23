//! tva-core — Temporal Video Analyzer core engine.
//! Headless, no knowledge of GUI/CLI/web. Pure data in, data out.

mod decoder;
mod compare;
mod detect;
mod metrics;
mod smooth;
mod resolution;
mod pipeline;
mod overlay;
mod export;
pub mod ffi;

pub use compare::{CompareMethod, CompareResult, compare_frames, diff_raw, diff_cielab, diff_ssim};
pub use detect::{DedupState, DuplicateInfo, TearInfo, detect_tear};
pub use metrics::{FrameMetric, SummaryMetrics, compute_frame_metrics, compute_summary};
pub use smooth::smooth_fps;
pub use resolution::{ResolutionResult, detect_resolution, rgb_to_gray};
pub use pipeline::{
    PipelineConfig, VideoMeta, Report, AnalysisEvent, FrameSource, FrameData, EventSink, NullSink,
    analyze,
};
pub use decoder::decode_file;
pub use export::{to_json, to_csv};
pub use overlay::overlay_video;
