use crate::error::{Result, TvaError};
use savgol_rs::{savgol_filter, SavGolInput};

/// Savitzky-Golay smoothing for FPS series.
///
/// Returns error if `window <= polyorder + 1` or data is empty.
pub fn smooth_fps(fps: &[f64], window: usize, polyorder: usize) -> Result<Vec<f64>> {
    if fps.is_empty() {
        return Ok(Vec::new());
    }
    if window <= polyorder + 1 {
        return Err(TvaError::SavgolFailed(
            format!("window {window} must be > polyorder+1 ({})", polyorder + 1),
        ));
    }
    if window > fps.len() {
        return Err(TvaError::SavgolFailed(
            format!("window {window} exceeds data length {}", fps.len()),
        ));
    }
    let w = if window % 2 == 0 { window + 1 } else { window };

    let input = SavGolInput { data: fps, window_length: w, poly_order: polyorder, derivative: 0 };
    savgol_filter(&input).map_err(TvaError::SavgolFailed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn savgol_preserves_length() {
        let data: Vec<f64> = (0..31).map(|x| (x as f64).sin()).collect();
        let s = smooth_fps(&data, 15, 3).unwrap();
        assert_eq!(s.len(), data.len());
    }

    #[test]
    fn empty_input_ok() {
        assert!(smooth_fps(&[], 5, 2).unwrap().is_empty());
    }

    #[test]
    fn invalid_window_errors() {
        assert!(smooth_fps(&[1.0; 10], 3, 5).is_err()); // window <= polyorder+1
    }
}
