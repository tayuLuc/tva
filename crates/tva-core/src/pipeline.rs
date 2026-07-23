//! Streaming pipeline: one frame at a time, bounded memory.

use crate::compare::CompareMethod;
use crate::detect::{DedupState, TearInfo};
use crate::error::Result;
use crate::metrics::{compute_frame_metrics, compute_summary, FrameMetric, SummaryMetrics};
use crate::resolution::{detect_resolution, rgb_to_gray, ResolutionResult};
use crate::smooth::smooth_fps;
use rgb::RGB8;

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
            compare_method: CompareMethod::CieLab { threshold: 2.0 },
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

#[derive(Debug, Clone)]
pub struct VideoMeta {
    pub width: u32,
    pub height: u32,
    pub fps: f64,
    pub total_frames: u64,
    pub codec: String,
}

#[derive(Debug, Clone)]
pub struct Report {
    pub schema_version: u32,
    pub meta: VideoMeta,
    pub summary: SummaryMetrics,
    pub frames: Vec<FrameMetric>,
    pub fps_smoothed: Vec<f64>,
    pub tears: Vec<TearInfo>,
    pub resolution: Option<Vec<ResolutionResult>>,
}

pub enum AnalysisEvent {
    Progress { frame: u64, total: u64 },
    Error { frame: u64, message: String },
}

pub trait FrameSource {
    fn metadata(&self) -> VideoMeta;
    fn next_frame(&mut self) -> Option<Vec<RGB8>>;
}

pub trait EventSink {
    fn on_event(&mut self, event: AnalysisEvent);
}

pub struct NullSink;
impl EventSink for NullSink {
    fn on_event(&mut self, _event: AnalysisEvent) {}
}

/// Streaming analysis pipeline.
pub fn analyze(
    source: &mut dyn FrameSource,
    config: &PipelineConfig,
    events: &mut dyn EventSink,
) -> Result<Report> {
    let meta = source.metadata();
    let container_fps = meta.fps;
    let (w, h) = (meta.width as usize, meta.height as usize);

    let mut dedup = DedupState::new();
    let mut streaks: Vec<u32> = Vec::new();
    let mut tears: Vec<TearInfo> = Vec::new();
    let mut resolutions: Vec<ResolutionResult> = Vec::new();
    let mut prev_frame: Option<Vec<RGB8>> = None;
    let mut frame_index: u64 = 0;

    while let Some(frame) = source.next_frame() {
        if let Some(dup) = dedup.process(&frame, frame_index, w, h, &config.compare_method)? {
            if let Some(last) = streaks.last_mut() {
                *last = dup.streak_length;
            }
        } else {
            streaks.push(1);
        }

        if config.detect_tears {
            if let Some(ref prev) = prev_frame {
                if let Some(mut tear) = crate::detect::detect_tear(
                    prev, &frame, w, h,
                    config.tear_threshold_high, config.tear_threshold_low,
                ) {
                    tear.frame_index = frame_index;
                    tears.push(tear);
                }
            }
        }

        if config.detect_resolution && frame_index % config.resolution_sample_interval as u64 == 0 {
            let gray = rgb_to_gray(&frame);
            if let Ok(res) = detect_resolution(&gray, w, h) {
                resolutions.push(res);
            }
        }

        events.on_event(AnalysisEvent::Progress { frame: frame_index, total: meta.total_frames });
        prev_frame = Some(frame);
        frame_index += 1;
    }

    let frame_metrics = compute_frame_metrics(&streaks, container_fps);
    let summary = compute_summary(&frame_metrics, tears.len() as u64);

    let fps_raw: Vec<f64> = frame_metrics.iter().map(|m| m.instantaneous_fps).collect();
    let fps_smoothed = smooth_fps(&fps_raw, config.smooth_window, config.smooth_polyorder)
        .unwrap_or(fps_raw);

    Ok(Report {
        schema_version: 1,
        meta,
        summary,
        frames: frame_metrics,
        fps_smoothed,
        tears,
        resolution: if config.detect_resolution { Some(resolutions) } else { None },
    })
}
