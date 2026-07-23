use crate::config::PipelineConfig;
use crate::detect::{DedupState, TearInfo};
use crate::error::Result;
use crate::events::{AnalysisEvent, EventSink};
use crate::frame::Frame;
use crate::metrics::{compute_frame_metrics, compute_summary};
use crate::report::Report;
use crate::resolution::{detect_resolution, ResolutionResult};
use crate::source::FrameSource;
use crate::traits::{FrameComparator, Smoother};

/// Generic pipeline: параметризован трейтами, не крейтами.
pub fn analyze(
    source: &mut dyn FrameSource,
    config: &PipelineConfig,
    comparator: &dyn FrameComparator,
    smoother: &dyn Smoother,
    events: &mut dyn EventSink,
) -> Result<Report> {
    let meta = source.metadata();
    let cfps = meta.fps;

    let mut dedup = DedupState::new(config.duplicate_threshold, comparator.higher_is_similar());
    let mut streaks: Vec<u32> = Vec::new();
    let mut tears: Vec<TearInfo> = Vec::new();
    let mut resolutions: Vec<ResolutionResult> = Vec::new();
    let mut pframe: Option<Frame> = None;
    let mut fc: u64 = 0;

    while let Some(frame) = source.next_frame() {
        if let Some(dup) = dedup.process(&frame, comparator)? {
            if let Some(l) = streaks.last_mut() { *l = dup.streak_length; }
            events.on_event(AnalysisEvent::DuplicateFound { frame: frame.index, streak: dup.streak_length });
        } else { streaks.push(1); }

        if config.detect_tears {
            if let Some(ref p) = pframe {
                if let Some(mut t) = crate::detect::detect_tear(p, &frame, config.tear_threshold_high, config.tear_threshold_low)? {
                    t.frame_index = frame.index;
                    tears.push(t);
                    events.on_event(AnalysisEvent::TearDetected { frame: frame.index, position: t.tear_position });
                }
            }
        }

        #[cfg(feature = "fft")]
        if config.detect_resolution && fc % config.resolution_sample_interval as u64 == 0 {
            if let Ok(res) = detect_resolution(&frame, crate::resolution::default_fft_2d) { resolutions.push(res); }
        }

        events.on_event(AnalysisEvent::Progress { frame: frame.index, total: meta.total_frames });
        pframe = Some(frame);
        fc += 1;
    }

    let fm = compute_frame_metrics(&streaks, cfps);
    let summary = compute_summary(&fm, tears.len() as u64);
    let fps_raw: Vec<f64> = fm.iter().map(|m| m.instantaneous_fps).collect();
    let fps_smooth = smoother.smooth(&fps_raw).unwrap_or(fps_raw);

    Ok(Report {
        schema_version: 1,
        meta, summary,
        frames: fm,
        fps_smoothed: fps_smooth,
        tears: Some(tears).filter(|t| !t.is_empty()),
        resolution: if cfg!(feature = "fft") && config.detect_resolution { Some(resolutions) } else { None },
    })
}
