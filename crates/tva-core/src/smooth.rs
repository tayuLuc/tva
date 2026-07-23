use crate::error::{Result, TvaError};
use savgol_rs::{savgol_filter, SavGolInput};

/// Savitzky-Golay smoothing for FPS series.
/// `window` — window size (must be odd, > polyorder). Clamped if invalid.
/// `polyorder` — polynomial order, typically 3.
pub fn smooth_fps(fps: &[f64], window: usize, polyorder: usize) -> Result<Vec<f64>> {
    if fps.is_empty() {
        return Ok(Vec::new());
    }

    let mut w = window.max(polyorder + 2);
    if w > fps.len() {
        w = fps.len();
    }
    if w % 2 == 0 {
        w = w.saturating_sub(1);
    }
    // ponytail: minimum window is polyorder + 2, odd. If data is tiny, no filtering.
    if w < 3 || w <= polyorder {
        return Ok(fps.to_vec());
    }

    let input = SavGolInput {
        data: fps,
        window_length: w,
        poly_order: polyorder,
        derivative: 0,
    };
    savgol_filter(&input).map_err(TvaError::SavgolFailed)
}
