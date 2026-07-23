use crate::error::Result;
use crate::events::{AnalysisEvent, EventSink};
use crate::frame::{Frame, VideoMeta};
use crate::pixel_buffer::PixelBuffer;
use crate::traits::{FrameComparator, FrameDecoder};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct DegradationConfig {
    pub max_time_drift_ms: f64,
}

impl Default for DegradationConfig {
    fn default() -> Self {
        Self { max_time_drift_ms: 16.0 }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SizeMismatch {
    pub a: (u32, u32),
    pub b: (u32, u32),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DegradationSummary {
    pub mean_score: f64,
    pub mean_similarity: f64,
    pub min_similarity: f64,
    pub quality_drop_pct: f64,
    pub pairs_compared: u64,
    pub pairs_dropped: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DegradationPoint {
    pub timestamp_ms: f64,
    pub similarity: f64,
    pub diff_pixel_ratio: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DegradationReport {
    pub schema_version: u32,
    pub source_a: VideoMeta,
    pub source_b: VideoMeta,
    pub summary: DegradationSummary,
    pub profile: Vec<DegradationPoint>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size_mismatch: Option<SizeMismatch>,
}

#[must_use]
pub fn diff_pixel_ratio(a: &[u8], b: &[u8]) -> f64 {
    debug_assert_eq!(a.len(), b.len());
    if a.is_empty() {
        return 0.0;
    }
    let n = a.len() / 3;
    let diff = a
        .chunks_exact(3)
        .zip(b.chunks_exact(3))
        .filter(|(ca, cb)| ca[0] != cb[0] || ca[1] != cb[1] || ca[2] != cb[2])
        .count() as u64;
    diff as f64 / n as f64
}

#[must_use]
pub fn diff_heatmap(a: &[u8], b: &[u8]) -> Vec<u8> {
    debug_assert_eq!(a.len(), b.len());
    a.chunks_exact(3)
        .zip(b.chunks_exact(3))
        .map(|(ca, cb)| {
            let d = (u16::from(ca[0].abs_diff(cb[0]))
                + u16::from(ca[1].abs_diff(cb[1]))
                + u16::from(ca[2].abs_diff(cb[2])))
                / 3;
            d as u8
        })
        .collect()
}

fn normalize(score: f64, higher_is_similar: bool) -> f64 {
    if higher_is_similar {
        score.clamp(0.0, 1.0)
    } else {
        (1.0 - score / 255.0).clamp(0.0, 1.0)
    }
}

pub fn compare_sources(
    a: &mut dyn FrameDecoder,
    b: &mut dyn FrameDecoder,
    comparator: &dyn FrameComparator,
    config: &DegradationConfig,
    events: &mut dyn EventSink,
) -> Result<DegradationReport> {
    let meta_a = a.metadata();
    let meta_b = b.metadata();

    if (meta_a.width, meta_a.height) != (meta_b.width, meta_b.height) {
        let (wa, ha) = (meta_a.width, meta_a.height);
        let (wb, hb) = (meta_b.width, meta_b.height);
        return Ok(DegradationReport {
            schema_version: 1,
            source_a: meta_a,
            source_b: meta_b,
            summary: empty_summary(),
            profile: Vec::new(),
            size_mismatch: Some(SizeMismatch { a: (wa, ha), b: (wb, hb) }),
        });
    }

    let higher = comparator.higher_is_similar();
    let drift = config.max_time_drift_ms;
    let mut fa = a.next_frame();
    let mut fb = b.next_frame();
    let mut profile: Vec<DegradationPoint> = Vec::new();
    let mut sum_score = 0.0_f64;
    let mut sum_sim = 0.0_f64;
    let mut min_sim = f64::INFINITY;
    let mut compared = 0_u64;
    let mut dropped = 0_u64;

    while let (Some(x), Some(y)) = (fa.take(), fb.take()) {
        let d = (x.timestamp_ms - y.timestamp_ms).abs();
        if d <= drift {
            let score = comparator.compare(&x.data, &y.data)?;
            let sim = normalize(score, higher);
            let ratio = diff_pixel_ratio(x.data.as_bytes(), y.data.as_bytes());
            sum_score += score;
            sum_sim += sim;
            if sim < min_sim {
                min_sim = sim;
            }
            compared += 1;
            profile.push(DegradationPoint { timestamp_ms: x.timestamp_ms, similarity: sim, diff_pixel_ratio: ratio });
            events.on_event(AnalysisEvent::ComparedPair { timestamp_ms: x.timestamp_ms, similarity: sim });
            fa = a.next_frame();
            fb = b.next_frame();
        } else if x.timestamp_ms < y.timestamp_ms {
            dropped += 1;
            fa = a.next_frame();
            fb = Some(y);
        } else {
            fa = Some(x);
            fb = b.next_frame();
        }
    }
    while let Some(_) = fa {
        dropped += 1;
        fa = a.next_frame();
    }

    let n = compared.max(1) as f64;
    let mean_score = if compared > 0 { sum_score / n } else { 0.0 };
    let mean_sim = if compared > 0 { sum_sim / n } else { 0.0 };
    let min_sim = if compared > 0 { min_sim } else { 0.0 };

    events.on_event(AnalysisEvent::Progress { frame: compared, total: meta_a.total_frames.min(meta_b.total_frames) });

    Ok(DegradationReport {
        schema_version: 1,
        source_a: meta_a,
        source_b: meta_b,
        summary: DegradationSummary {
            mean_score,
            mean_similarity: mean_sim,
            min_similarity: min_sim,
            quality_drop_pct: (1.0 - mean_sim) * 100.0,
            pairs_compared: compared,
            pairs_dropped: dropped,
        },
        profile,
        size_mismatch: None,
    })
}

fn empty_summary() -> DegradationSummary {
    DegradationSummary {
        mean_score: 0.0,
        mean_similarity: 0.0,
        min_similarity: 0.0,
        quality_drop_pct: 0.0,
        pairs_compared: 0,
        pairs_dropped: 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pixel_buffer::PixelBuffer;

    struct VecSource {
        frames: Vec<Frame>,
        meta: VideoMeta,
        i: usize,
    }
    impl VecSource {
        fn new(frames: Vec<Frame>, fps: f64, w: u32, h: u32) -> Self {
            let total = frames.len() as u64;
            Self {
                meta: VideoMeta {
                    fps,
                    width: w,
                    height: h,
                    total_frames: total,
                    duration_ms: total as f64 / fps * 1000.0,
                    codec: "test".into(),
                },
                frames,
                i: 0,
            }
        }
    }
    impl FrameDecoder for VecSource {
        fn metadata(&self) -> VideoMeta {
            self.meta.clone()
        }
        fn next_frame(&mut self) -> Option<Frame> {
            let f = self.frames.get(self.i)?.clone();
            self.i += 1;
            Some(f)
        }
    }

    fn mk(rgb: u8, w: u32, h: u32, ts: f64, idx: u64) -> Frame {
        Frame { data: PixelBuffer::new(vec![rgb; (w * h * 3) as usize], w, h).unwrap(), index: idx, timestamp_ms: ts }
    }

    struct Eq;
    impl FrameComparator for Eq {
        fn compare(&self, a: &PixelBuffer, b: &PixelBuffer) -> Result<f64> {
            Ok(if a.as_bytes() == b.as_bytes() { 1.0 } else { 0.0 })
        }
        fn higher_is_similar(&self) -> bool {
            true
        }
        fn name(&self) -> &'static str {
            "eq"
        }
    }

    struct Noop;
    impl EventSink for Noop {
        fn on_event(&mut self, _: AnalysisEvent) {}
    }

    #[test]
    fn identical_sources_zero_drop() {
        let fa = vec![mk(100, 4, 4, 0.0, 0), mk(100, 4, 4, 100.0, 1)];
        let fb = vec![mk(100, 4, 4, 0.0, 0), mk(100, 4, 4, 100.0, 1)];
        let mut a = VecSource::new(fa, 10.0, 4, 4);
        let mut b = VecSource::new(fb, 10.0, 4, 4);
        let r = compare_sources(&mut a, &mut b, &Eq, &DegradationConfig::default(), &mut Noop).unwrap();
        assert!(r.size_mismatch.is_none());
        assert_eq!(r.summary.pairs_compared, 2);
        assert!((r.summary.quality_drop_pct).abs() < f64::EPSILON);
    }

    #[test]
    fn different_sources_full_drop() {
        let fa = vec![mk(100, 4, 4, 0.0, 0)];
        let fb = vec![mk(200, 4, 4, 0.0, 0)];
        let mut a = VecSource::new(fa, 10.0, 4, 4);
        let mut b = VecSource::new(fb, 10.0, 4, 4);
        let r = compare_sources(&mut a, &mut b, &Eq, &DegradationConfig::default(), &mut Noop).unwrap();
        assert_eq!(r.summary.pairs_compared, 1);
        assert!((r.summary.quality_drop_pct - 100.0).abs() < f64::EPSILON);
    }

    #[test]
    fn time_matching_across_different_fps() {
        let fa = vec![mk(50, 4, 4, 0.0, 0), mk(50, 4, 4, 100.0, 1), mk(50, 4, 4, 200.0, 2)];
        let fb = vec![
            mk(50, 4, 4, 0.0, 0),
            mk(50, 4, 4, 50.0, 1),
            mk(50, 4, 4, 100.0, 2),
            mk(50, 4, 4, 150.0, 3),
            mk(50, 4, 4, 200.0, 4),
        ];
        let mut a = VecSource::new(fa, 10.0, 4, 4);
        let mut b = VecSource::new(fb, 20.0, 4, 4);
        let r =
            compare_sources(&mut a, &mut b, &Eq, &DegradationConfig { max_time_drift_ms: 10.0 }, &mut Noop).unwrap();
        assert_eq!(r.summary.pairs_compared, 3);
        assert_eq!(r.summary.pairs_dropped, 0);
    }

    #[test]
    fn size_mismatch_aborts_without_reading() {
        let fa = vec![mk(50, 4, 4, 0.0, 0)];
        let fb = vec![mk(50, 8, 8, 0.0, 0)];
        let mut a = VecSource::new(fa, 10.0, 4, 4);
        let mut b = VecSource::new(fb, 10.0, 8, 8);
        let r = compare_sources(&mut a, &mut b, &Eq, &DegradationConfig::default(), &mut Noop).unwrap();
        assert!(r.size_mismatch.is_some());
        assert!(r.profile.is_empty());
        assert_eq!(r.summary.pairs_compared, 0);
    }

    #[test]
    fn heatmap_identical_is_zero() {
        let a = vec![100u8; 12];
        assert!(diff_heatmap(&a, &a).iter().all(|&v| v == 0));
    }
    #[test]
    fn heatmap_different_is_nonzero() {
        assert!(diff_heatmap(&vec![0u8; 12], &vec![255u8; 12]).iter().all(|&v| v == 255));
    }
    #[test]
    fn diff_ratio_counts_pixels_not_bytes() {
        let a = vec![0, 0, 0, 0, 0, 0];
        let b = vec![0, 0, 0, 255, 255, 255];
        assert!((diff_pixel_ratio(&a, &b) - 0.5).abs() < f64::EPSILON);
    }
}
