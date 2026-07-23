//! Statistical metrics: 1% low FPS, P99 frametime, percentiles.

/// 1% low FPS from frametime deltas (ms).
pub fn one_percent_low(deltas_ms: &[f64]) -> f64 {
    if deltas_ms.is_empty() {
        return 0.0;
    }
    let mut v = deltas_ms.to_vec();
    v.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
    let n = (v.len() as f64 * 0.01) as usize;
    if n == 0 {
        return 1000.0 / v.first().copied().unwrap_or(16.67);
    }
    let avg: f64 = v[..n.min(v.len())].iter().sum::<f64>() / n.max(1) as f64;
    if avg > 0.0 {
        1000.0 / avg
    } else {
        0.0
    }
}

/// P99 frametime in milliseconds.
pub fn p99(deltas_ms: &[f64]) -> f64 {
    if deltas_ms.is_empty() {
        return 0.0;
    }
    let mut v = deltas_ms.to_vec();
    v.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
    let idx = ((v.len() as f64) * 0.99) as usize;
    v.get(idx.min(v.len().saturating_sub(1)))
        .copied()
        .unwrap_or(0.0)
}
