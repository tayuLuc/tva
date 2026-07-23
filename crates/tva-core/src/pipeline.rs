//! Streaming analysis pipeline. Wires FrameSource → compare/detect → Report.

use crate::config::PipelineConfig;
use crate::detect::{DedupState, TearInfo};
use crate::error::Result;
use crate::events::{AnalysisEvent, EventSink};
use crate::frame::Frame;
use crate::metrics::{compute_frame_metrics, compute_summary, SummaryMetrics};
use crate::report::Report;
use crate::resolution::{detect_resolution, ResolutionResult};
use crate::smooth::smooth_fps;
use crate::source::FrameSource;

/// Run the full analysis pipeline over a frame source.
pub fn analyze(
    source: &mut dyn FrameSource,
    config: &PipelineConfig,
    events: &mut dyn EventSink,
) -> Result<Report> {
    let meta = source.metadata();
    let container_fps = meta.fps;

    let mut dedup = DedupState::new();
    let mut streaks: Vec<u32> = Vec::new();
    let mut tears: Vec<TearInfo> = Vec::new();
    let mut resolutions: Vec<ResolutionResult> = Vec::new();
    let mut prev_frame_data: Option<Frame> = None;
    let mut frame_count: u64 = 0;

    while let Some(frame) = source.next_frame() {
        // 1. Duplicate detection
        if let Some(dup) = dedup.process(&frame, &config.compare_method)? {
            if let Some(last) = streaks.last_mut() {
                *last = dup.streak_length;
            }
            events.on_event(AnalysisEvent::DuplicateFound {
                frame: frame.index,
                streak: dup.streak_length,
            });
        } else {
            streaks.push(1);
        }

        // 2. Tear detection
        if config.detect_tears {
            if let Some(ref prev) = prev_frame_data {
                if let Some(mut tear) = crate::detect::detect_tear(
                    prev, &frame,
                    config.tear_threshold_high, config.tear_threshold_low,
                )? {
                    tear.frame_index = frame.index;
                    tears.push(tear);
                    events.on_event(AnalysisEvent::TearDetected {
                        frame: frame.index,
                        position: tear.tear_position,
                    });
                }
            }
        }

        // 3. Resolution detection (sampled)
        if config.detect_resolution && frame_count % config.resolution_sample_interval as u64 == 0 {
            if let Ok(res) = detect_resolution(&frame) {
                resolutions.push(res);
            }
        }

        // 4. Second-boundary event
        if frame.timestamp_ms > 0.0 && frame_count > 0 {
            let prev_sec = ((prev_frame_data.as_ref().map_or(0.0, |f| f.timestamp_ms)) / 1000.0) as u32;
            let cur_sec = (frame.timestamp_ms / 1000.0) as u32;
            if cur_sec > prev_sec {
                events.on_event(AnalysisEvent::SecondComplete {
                    second: cur_sec,
                    unique_frames: streaks.len() as u32,
                });
            }
        }

        events.on_event(AnalysisEvent::Progress {
            frame: frame.index,
            total: meta.total_frames,
        });

        prev_frame_data = Some(frame);
        frame_count += 1;
    }

    // 5. Metrics + smoothing
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
