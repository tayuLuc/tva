//! Smoothing algorithms for frame metric series.
//! Wraps `savgol-rs` for Savitzky-Golay filter.

/// Savitzky-Golay smoothing для ряда мгновенного FPS.
/// - `window`: размер окна (нечётное, ≥ polyorder+2). Рекомендация: 15–31.
/// - `polyorder`: порядок полинома, обычно 3.
pub fn smooth_fps(fps: &[f64], window: usize, polyorder: usize) -> Vec<f64> {
    let window = if window % 2 == 0 { window + 1 } else { window };
    match savgol_rs::savgol_filter(fps, window, polyorder) {
        Ok(result) => result,
        Err(_) => fps.to_vec(),
    }
}
