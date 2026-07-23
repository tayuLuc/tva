//! Smoothing algorithms for frame metric series.

/// Savitzky-Golay smoothing (window=5, order=2).
/// ponytail: hardcoded 5-point quadratic; generic impl when more window sizes are needed.
pub fn savitzky_golay(data: &[f64]) -> Vec<f64> {
    let c = [-3.0, 12.0, 17.0, 12.0, -3.0];
    let half = 2;
    data.iter()
        .enumerate()
        .map(|(i, _)| {
            if i < half || i >= data.len() - half {
                data[i]
            } else {
                let sum: f64 = (0..5).map(|j| data[i + j - half] * c[j]).sum();
                (sum / 35.0).max(0.0)
            }
        })
        .collect()
}

/// Median filter (window=3).
pub fn median_3(data: &[f64]) -> Vec<f64> {
    data.windows(3)
        .map(|w| {
            let mut v = [w[0], w[1], w[2]];
            v.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
            v[1]
        })
        .collect()
}
