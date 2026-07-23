//! Streaming pipeline: one frame at a time, bounded memory.
//! Собирает вместе compare, detect, metrics, smooth, resolution.

use crate::compare::CompareMethod;
use crate::detect::{DedupState, TearInfo};
use crate::metrics::{compute_frame_metrics, compute_summary, FrameMetric, SummaryMetrics};
use crate::resolution::{detect_resolution, rgb_to_gray, ResolutionResult};
use crate::smooth::smooth_fps;

/// Конфигурация конвейера.
#[derive(Clone, Debug)]
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

/// Метаданные видео.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct VideoMeta {
    pub width: u32,
    pub height: u32,
    pub fps: f64,
    pub total_frames: u64,
    pub codec: String,
}

/// Итоговый отчёт анализа.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Report {
    pub schema_version: u32,
    pub meta: VideoMeta,
    pub summary: SummaryMetrics,
    pub frames: Vec<FrameMetric>,
    pub fps_smoothed: Vec<f64>,
    pub tears: Vec<TearInfo>,
    pub resolution: Option<Vec<ResolutionResult>>,
}

/// События конвейера (для колбэков).
pub enum AnalysisEvent {
    Progress { frame: u64, total: u64 },
    Error { frame: u64, message: String },
}

/// Трейт источника кадров (FFmpeg, WebCodecs, тестовый).
pub trait FrameSource {
    fn metadata(&self) -> VideoMeta;
    fn next_frame(&mut self) -> Option<FrameData>;
}

/// Сырой кадр: плоский RGB-буфер.
#[derive(Clone, Debug)]
pub struct FrameData {
    pub data: Vec<u8>,
    pub index: u64,
}

/// Трейт для получения событий (прогресс, ошибки).
pub trait EventSink {
    fn on_event(&mut self, event: AnalysisEvent);
}

/// Заглушка для EventSink, когда колбэки не нужны.
pub struct NullSink;
impl EventSink for NullSink {
    fn on_event(&mut self, _event: AnalysisEvent) {}
}

/// Streaming pipeline: читает кадры из FrameSource, выдаёт Report.
///
/// ## Батчи
///
/// 1. Duplicate detection через `DedupState`
/// 2. Tear detection между предыдущим и текущим кадром
/// 3. Resolution detection (каждый N-й кадр)
/// 4. Итоговые метрики + сглаживание FPS
pub fn analyze(
    source: &mut dyn FrameSource,
    config: &PipelineConfig,
    events: &mut dyn EventSink,
) -> Report {
    let meta = source.metadata();
    let container_fps = meta.fps;
    let (w, h) = (meta.width, meta.height);

    let mut dedup = DedupState::new();
    let mut streaks: Vec<u32> = Vec::new();
    let mut tears: Vec<TearInfo> = Vec::new();
    let mut resolutions: Vec<ResolutionResult> = Vec::new();
    let mut prev_frame: Option<Vec<u8>> = None;

    while let Some(frame) = source.next_frame() {
        // 1. Duplicate detection
        if let Some(dup) = dedup.process(
            &frame.data,
            frame.index,
            &config.compare_method,
            w,
            h,
        ) {
            if let Some(last) = streaks.last_mut() {
                *last = dup.streak_length;
            }
        } else {
            streaks.push(1);
        }

        // 2. Tear detection
        if config.detect_tears {
            if let Some(ref prev) = prev_frame {
                if let Some(mut tear) = crate::detect::detect_tear(
                    prev,
                    &frame.data,
                    w,
                    h,
                    config.tear_threshold_high,
                    config.tear_threshold_low,
                ) {
                    tear.frame_index = frame.index;
                    tears.push(tear);
                }
            }
        }

        // 3. Resolution detection (sampled)
        if config.detect_resolution
            && frame.index % config.resolution_sample_interval as u64 == 0
        {
            let gray = rgb_to_gray(&frame.data, w, h);
            resolutions.push(detect_resolution(&gray, w as usize, h as usize));
        }

        // 4. Прогресс
        events.on_event(AnalysisEvent::Progress {
            frame: frame.index,
            total: meta.total_frames,
        });

        prev_frame = Some(frame.data);
    }

    // 5. Метрики
    let frame_metrics = compute_frame_metrics(&streaks, container_fps);
    let summary = compute_summary(&frame_metrics, tears.len() as u64);

    // 6. Сглаживание
    let fps_raw: Vec<f64> = frame_metrics
        .iter()
        .map(|m| m.instantaneous_fps)
        .collect();
    let fps_smoothed = smooth_fps(&fps_raw, config.smooth_window, config.smooth_polyorder);

    Report {
        schema_version: 1,
        meta,
        summary,
        frames: frame_metrics,
        fps_smoothed,
        tears,
        resolution: if config.detect_resolution {
            Some(resolutions)
        } else {
            None
        },
    }
}
