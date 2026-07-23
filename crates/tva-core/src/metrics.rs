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
    let frame_time = 1000.0 / container_fps;
    let mut container_frame: u64 = 0;

    streaks
        .iter()
        .enumerate()
        .map(|(i, &streak)| {
            let streak_f = f64::from(streak);
            let m = FrameMetric {
                container_frame,
                unique_frame: i as u64,
                streak_length: streak,
                real_frame_time_ms: streak_f * frame_time,
                instantaneous_fps: container_fps / streak_f,
            };
            container_frame += u64::from(streak);
            m
        })
        .collect()
}

#[must_use]
pub fn compute_summary(metrics: &[FrameMetric], tear_count: u64) -> SummaryMetrics {
    let n = metrics.len();
    if n == 0 {
        return SummaryMetrics {
            avg_fps: 0.0, fps_1_low: 0.0, fps_01_low: 0.0,
            p90_frame_time_ms: 0.0, p99_frame_time_ms: 0.0,
            total_container_frames: 0, total_unique_frames: 0,
            duplicate_count: 0, tear_count,
        };
    }

    let mut fps_sorted: Vec<f64> = metrics.iter().map(|m| m.instantaneous_fps).collect();
    fps_sorted.sort_by(f64::total_cmp);

    let pct_1 = ((n as f64 * 0.01).ceil() as usize).max(1);
    let pct_01 = ((n as f64 * 0.001).ceil() as usize).max(1);

    let fps_1_low = fps_sorted[..pct_1].iter().sum::<f64>() / pct_1 as f64;
    let fps_01_low = fps_sorted[..pct_01].iter().sum::<f64>() / pct_01 as f64;

    let mut ft_sorted: Vec<f64> = metrics.iter().map(|m| m.real_frame_time_ms).collect();
    ft_sorted.sort_by(f64::total_cmp);

    let p90_idx = ((n as f64 * 0.90).ceil() as usize).min(n).saturating_sub(1);
    let p99_idx = ((n as f64 * 0.99).ceil() as usize).min(n).saturating_sub(1);

    let avg_fps = fps_sorted.iter().sum::<f64>() / n as f64;
    let total_container: u64 = metrics.iter().map(|m| u64::from(m.streak_length)).sum();

    SummaryMetrics {
        avg_fps,
        fps_1_low,
        fps_01_low,
        p90_frame_time_ms: ft_sorted[p90_idx],
        p99_frame_time_ms: ft_sorted[p99_idx],
        total_container_frames: total_container,
        total_unique_frames: n as u64,
        duplicate_count: total_container - n as u64,
        tear_count,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_metrics_are_zero() {
        let s = compute_summary(&[], 0);
        assert_eq!(s.avg_fps, 0.0);
        assert_eq!(s.tear_count, 0);
    }

    #[test]
    fn metrics_1_percent_low() {
        // 100 unique frames: 99 at 60fps, 1 at 10fps (streak=6)
        let mut streaks = vec![1u32; 99];
        streaks.push(6);
        let m = compute_frame_metrics(&streaks, 60.0);
        let s = compute_summary(&m, 0);
        assert!(s.fps_1_low < 15.0); // 1% low catches the 10fps outlier
        assert!(s.avg_fps > 50.0);
        assert_eq!(s.duplicate_count, 5); // one streak of 6 = 5 extra
    }

    #[test]
    fn frame_metrics_length() {
        let streaks = vec![1, 2, 1, 3];
        let m = compute_frame_metrics(&streaks, 60.0);
        assert_eq!(m.len(), 4);
        assert_eq!(m[0].instantaneous_fps, 60.0);
        assert_eq!(m[1].instantaneous_fps, 30.0);
    }
}
