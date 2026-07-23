use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct FrameMetric {
    pub container_frame: u64,
    pub unique_frame: u64,
    pub streak_length: u32,
    pub real_frame_time_ms: f64,
    pub instantaneous_fps: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct SummaryMetrics {
    pub avg_fps: f64,
    pub fps_1_low: f64,
    pub fps_01_low: f64,
    pub p90_frame_time_ms: f64,
    pub p99_frame_time_ms: f64,
    pub total_container_frames: u64,
    pub total_unique_frames: u64,
    pub duplicate_count: u64,
    pub tear_count: u64,
}

#[must_use]
pub fn compute_frame_metrics(streaks: &[u32], container_fps: f64) -> Vec<FrameMetric> {
    let t = 1000.0 / container_fps;
    let mut cf: u64 = 0;
    streaks.iter().enumerate().map(|(i, &s)| {
        let sf = f64::from(s);
        let m = FrameMetric { container_frame: cf, unique_frame: i as u64, streak_length: s, real_frame_time_ms: sf * t, instantaneous_fps: container_fps / sf };
        cf += u64::from(s);
        m
    }).collect()
}

#[must_use]
pub fn compute_summary(metrics: &[FrameMetric], tear_count: u64) -> SummaryMetrics {
    let n = metrics.len();
    if n == 0 { return SummaryMetrics { avg_fps: 0.0, fps_1_low: 0.0, fps_01_low: 0.0, p90_frame_time_ms: 0.0, p99_frame_time_ms: 0.0, total_container_frames: 0, total_unique_frames: 0, duplicate_count: 0, tear_count }; }

    let mut fs: Vec<f64> = metrics.iter().map(|m| m.instantaneous_fps).collect();
    fs.sort_by(f64::total_cmp);
    let p1 = ((n as f64 * 0.01).ceil() as usize).max(1);
    let p01 = ((n as f64 * 0.001).ceil() as usize).max(1);
    let f1l = fs[..p1].iter().sum::<f64>() / p1 as f64;
    let f01l = fs[..p01].iter().sum::<f64>() / p01 as f64;

    let mut ft: Vec<f64> = metrics.iter().map(|m| m.real_frame_time_ms).collect();
    ft.sort_by(f64::total_cmp);
    let p90 = ((n as f64 * 0.90).ceil() as usize).min(n).saturating_sub(1);
    let p99 = ((n as f64 * 0.99).ceil() as usize).min(n).saturating_sub(1);

    SummaryMetrics {
        avg_fps: fs.iter().sum::<f64>() / n as f64,
        fps_1_low: f1l, fps_01_low: f01l,
        p90_frame_time_ms: ft[p90], p99_frame_time_ms: ft[p99],
        total_container_frames: metrics.iter().map(|m| u64::from(m.streak_length)).sum(),
        total_unique_frames: n as u64,
        duplicate_count: metrics.iter().map(|m| u64::from(m.streak_length)).sum::<u64>() - n as u64,
        tear_count,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty() { let s = compute_summary(&[], 0); assert_eq!(s.avg_fps, 0.0); }

    #[test]
    fn one_percent_low() {
        let mut s = vec![1u32; 99]; s.push(6);
        let m = compute_frame_metrics(&s, 60.0);
        let sm = compute_summary(&m, 0);
        assert!(sm.fps_1_low < 15.0);
    }
}
