use crate::config::PipelineConfig;
use crate::detect::{DedupState, TearInfo};
use crate::error::Result;
use crate::events::{AnalysisEvent, EventSink};
use crate::frame::Frame;
use crate::metrics::{compute_frame_metrics, compute_summary};
use crate::pixel_buffer::PixelBuffer;
use crate::report::Report;
use crate::resolution::ResolutionResult;
use crate::traits::{FrameComparator, FrameDecoder, Smoother};

#[allow(unused_assignments)]
pub fn analyze(
    source: &mut dyn FrameDecoder,
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
    let mut tear_suppressed: u64 = 0;
    #[allow(unused_mut)]
    let mut resolutions: Vec<ResolutionResult> = Vec::new();
    #[allow(unused_assignments, unused_variables)]
    let mut frame_counter: u64 = 0;

    while let Some(frame) = source.next_frame() {
        let peek = dedup.peek(&frame, comparator)?;

        let tear = if config.detect_tears && !peek.first {
            dedup.prev_bytes().and_then(|prev| {
                let prev_frame = Frame {
                    data: PixelBuffer::new(prev.to_vec(), meta.width, meta.height).ok()?,
                    index: frame.index.saturating_sub(1),
                    timestamp_ms: 0.0,
                };
                crate::detect::detect_tear(&prev_frame, &frame, config.tear_threshold_high, config.tear_threshold_low)
                    .ok()?
            })
        } else {
            None
        };

        let suppress =
            tear.as_ref().map(|t| f64::from(t.new_fraction) >= config.dismiss_tear_percentage).unwrap_or(false);

        let is_unique = !peek.first && !peek.dup_by_pixels && !suppress;

        let dup = dedup.commit(&frame, is_unique);
        if peek.first || is_unique {
            streaks.push(1);
        } else if let Some(last) = streaks.last_mut() {
            *last = dup.as_ref().map(|d| d.streak_length).unwrap_or(*last + 1);
        }

        if suppress {
            tear_suppressed += 1;
        }

        if let Some(mut t) = tear {
            t.frame_index = frame.index;
            let pos = t.tear_position;
            tears.push(t);
            events.on_event(AnalysisEvent::TearDetected { frame: frame.index, position: pos });
        }
        if let Some(d) = &dup {
            events.on_event(AnalysisEvent::DuplicateFound { frame: frame.index, streak: d.streak_length });
        }

        #[cfg(feature = "fft")]
        if config.detect_resolution && frame_counter.is_multiple_of(config.resolution_sample_interval as u64) {
            if let Ok(res) = crate::resolution::detect_resolution(&frame, crate::resolution::default_fft_2d) {
                resolutions.push(res);
            }
        }

        events.on_event(AnalysisEvent::Progress { frame: frame.index, total: meta.total_frames });
        frame_counter += 1;
    }

    let fm = compute_frame_metrics(&streaks, cfps);
    let summary = compute_summary(&fm, tears.len() as u64, tear_suppressed);
    let fps_raw: Vec<f64> = fm.iter().map(|m| m.instantaneous_fps).collect();
    let fps_smooth = smoother.smooth(&fps_raw).unwrap_or_else(|_| fps_raw.clone());

    Ok(Report {
        schema_version: 1,
        meta,
        summary,
        frames: fm,
        fps_smoothed: fps_smooth,
        tears: Some(tears).filter(|t| !t.is_empty()),
        resolution: if cfg!(feature = "fft") && config.detect_resolution { Some(resolutions) } else { None },
    })
}
