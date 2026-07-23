//! tva-core — Temporal Video Analyzer core engine.
//! Headless, no knowledge of GUI/CLI/web. Pure data in, data out.

mod decoder;
mod compare;
mod detect;
mod metrics;
mod smooth;
mod overlay;
mod export;
pub mod ffi;

use std::path::Path;
use image::RgbImage;

/// A single decoded video frame.
#[derive(Clone)]
pub struct Frame {
    pub pts: i64,
    pub duration: i64,
    pub image: RgbImage,
}

/// Analysis options.
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct AnalyzeOpts {
    pub metrics: Vec<String>,
    pub skip_duplicates: bool,
    pub skip_tears: bool,
}

impl Default for AnalyzeOpts {
    fn default() -> Self {
        Self {
            metrics: vec!["fps".into(), "duplicates".into(), "tears".into()],
            skip_duplicates: false,
            skip_tears: false,
        }
    }
}

/// Per-frame metrics.
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct FrameMetric {
    pub frame_num: usize,
    pub pts_us: i64,
    pub delta_us: i64,
    pub duplicate: bool,
    pub tear_line: Option<u32>,
    pub delta_cielab: f64,
}

/// Analysis report.
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct Report {
    pub path: String,
    pub total_frames: usize,
    pub duration_secs: f64,
    pub real_fps: f64,
    pub nominal_fps: f64,
    pub duplicate_count: usize,
    pub tear_count: usize,
    pub one_percent_low_fps: f64,
    pub p99_frametime_ms: f64,
    pub frames: Vec<FrameMetric>,
}

/// Full pipeline: decode file → analyze → report.
pub fn analyze(path: &Path, opts: AnalyzeOpts) -> Result<Report, Box<dyn std::error::Error>> {
    let frames = decoder::decode_file(path)?;
    let report = analyze_frames(&frames, opts);
    Ok(report)
}

/// Analyze already-decoded frames (for WASM / in-memory use).
pub fn analyze_frames(frames: &[Frame], opts: AnalyzeOpts) -> Report {
    let total_frames = frames.len();
    let duration_secs = if frames.len() > 1 {
        (frames.last().unwrap().pts - frames.first().unwrap().pts) as f64 / 1_000_000.0
    } else {
        0.0
    };
    let nominal_fps = if total_frames > 1 && duration_secs > 0.0 {
        total_frames as f64 / duration_secs
    } else {
        0.0
    };

    let mut metrics_list: Vec<FrameMetric> = Vec::with_capacity(total_frames);
    let mut prev: Option<&Frame> = None;
    let mut dup_count = 0;
    let mut tear_count = 0;

    for (i, frame) in frames.iter().enumerate() {
        let (dup, tear) = if let Some(p) = prev {
            let is_dup = !opts.skip_duplicates && detect::is_duplicate(frame, p);
            let tear_l = if !opts.skip_tears {
                detect::find_tear_line(frame)
            } else {
                None
            };
            (is_dup, tear_l)
        } else {
            (false, None)
        };

        if dup {
            dup_count += 1;
        }
        if tear.is_some() {
            tear_count += 1;
        }

        let delta = prev.map(|p| frame.pts - p.pts).unwrap_or(0);
        let delta_cielab = prev.map(|p| compare::cielab_diff(frame, p)).unwrap_or(0.0);

        metrics_list.push(FrameMetric {
            frame_num: i + 1,
            pts_us: frame.pts,
            delta_us: delta,
            duplicate: dup,
            tear_line: tear,
            delta_cielab,
        });
        prev = Some(frame);
    }

    let deltas: Vec<f64> = metrics_list
        .iter()
        .skip(1)
        .map(|m| m.delta_us as f64 / 1000.0)
        .collect();
    let real_fps = if duration_secs > 0.0 {
        total_frames as f64 / duration_secs
    } else {
        0.0
    };
    let one_percent_low_fps = metrics::one_percent_low(&deltas);
    let p99_frametime_ms = metrics::p99(&deltas);

    Report {
        path: String::new(),
        total_frames,
        duration_secs,
        real_fps,
        nominal_fps,
        duplicate_count: dup_count,
        tear_count,
        one_percent_low_fps,
        p99_frametime_ms,
        frames: metrics_list,
    }
}

/// Render FPS overlay on video and export to file.
pub fn overlay_video(
    path: &Path,
    report: &Report,
    out: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    overlay::render(path, report, out)
}
