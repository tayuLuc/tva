use crate::error::{Result, TvaError};
use staged_sg_filter::savgol;

/// Savitzky-Golay smoothing for FPS series (SIMD in-place).
///
/// Constraints: window must be odd and ≥ polyorder+2, ≤ data.len().
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
    let mut data = fps.to_vec();
    savgol(&mut data, window, polyorder)
        .map_err(|e| TvaError::SavgolFailed(e.to_string()))?;
    Ok(data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preserves_length() {
        let data: Vec<f64> = (0..31).map(|x| (x as f64 * 0.2).sin()).collect();
        let s = smooth_fps(&data, 15, 3).unwrap();
        assert_eq!(s.len(), data.len());
    }

    #[test]
    fn empty_ok() {
        assert!(smooth_fps(&[], 5, 2).unwrap().is_empty());
    }

    #[test]
    fn rejects_even_window() {
        assert!(smooth_fps(&[1.0; 10], 10, 3).is_err());
    }

    #[test]
    fn rejects_small_window() {
        assert!(smooth_fps(&[1.0; 10], 3, 5).is_err());
    }
}
