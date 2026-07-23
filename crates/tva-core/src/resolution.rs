use crate::error::{Result, TvaError};
use crate::frame::Frame;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum UpscaleVerdict {
    Native,
    LikelyUpscaled,
    #[default]
    Uncertain,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolutionResult {
    pub cutoff_x: u32,
    pub cutoff_y: u32,
    pub estimated_native: (u32, u32),
    #[serde(default)]
    pub container: (u32, u32),
    #[serde(default)]
    pub upscale_ratio_h: f64,
    #[serde(default)]
    pub verdict: UpscaleVerdict,
    #[serde(default)]
    pub confidence: f64,
    #[serde(default)]
    pub sharpness: f64,
}

const STANDARD_RESOLUTIONS: [(u32, u32); 6] =
    [(640, 360), (1280, 720), (1440, 810), (1560, 873), (1600, 900), (1920, 1080)];

#[must_use]
pub fn rgb_to_gray(bytes: &[u8]) -> Vec<f64> {
    bytes
        .chunks_exact(3)
        .map(|c| (0.2126 * f64::from(c[0]) + 0.7152 * f64::from(c[1]) + 0.0722 * f64::from(c[2])) / 255.0)
        .collect()
}

#[must_use]
pub fn laplacian_sharpness(gray: &[f64], w: usize, h: usize) -> f64 {
    if w < 3 || h < 3 || gray.len() != w * h {
        return 0.0;
    }
    let mut sum = 0.0_f64;
    let mut n = 0_u64;
    for y in 1..h - 1 {
        for x in 1..w - 1 {
            let c = gray[y * w + x];
            sum +=
                (4.0 * c - gray[(y - 1) * w + x] - gray[(y + 1) * w + x] - gray[y * w + x - 1] - gray[y * w + x + 1])
                    .abs();
            n += 1;
        }
    }
    if n == 0 {
        0.0
    } else {
        sum / n as f64
    }
}

#[must_use]
pub fn judge(ratio_h: f64, cutoff_sharpness: f64, aspect_match: bool) -> (UpscaleVerdict, f64) {
    let mut confidence = (cutoff_sharpness * 1.5).clamp(0.0, 1.0);
    if !aspect_match {
        confidence *= 0.5;
    }
    let verdict = if !aspect_match {
        UpscaleVerdict::Uncertain
    } else if ratio_h <= 1.15 {
        UpscaleVerdict::Native
    } else if ratio_h >= 1.30 {
        UpscaleVerdict::LikelyUpscaled
    } else {
        UpscaleVerdict::Uncertain
    };
    let verdict =
        if confidence < 0.3 && verdict != UpscaleVerdict::Uncertain { UpscaleVerdict::Uncertain } else { verdict };
    (verdict, confidence)
}

pub fn detect_resolution<F>(frame: &Frame, fft_2d: F) -> Result<ResolutionResult>
where
    F: Fn(&[f64], usize, usize) -> Result<Vec<Vec<f64>>>,
{
    let w = frame.data.width() as usize;
    let h = frame.data.height() as usize;
    if w < 4 || h < 4 {
        return Err(TvaError::FftFailed("frame too small".into()));
    }

    let gray = rgb_to_gray(frame.data.as_bytes());
    let sharpness = laplacian_sharpness(&gray, w, h);
    let magnitude = fft_2d(&gray, w, h)?;

    let cx = w / 2;
    let cy = h / 2;
    let max_r = cx.max(cy);
    let mut radial_power = vec![0.0_f64; max_r];
    let mut radial_count = vec![0_u64; max_r];

    for (sy, row) in magnitude.iter().enumerate().take(h) {
        for (sx, val) in row.iter().enumerate().take(w) {
            let r = (((sx as i64 - cx as i64).pow(2) + (sy as i64 - cy as i64).pow(2)) as f64).sqrt() as usize;
            if r < max_r {
                radial_power[r] += val;
                radial_count[r] += 1;
            }
        }
    }
    for (p, c) in radial_power.iter_mut().zip(&radial_count) {
        if *c > 0 {
            *p /= *c as f64;
        }
    }

    let peak = radial_power.iter().skip(1).cloned().fold(0.0_f64, f64::max);
    if peak == 0.0 {
        return Ok(native_fallback(w, h, sharpness));
    }

    let cr = radial_power.iter().skip(1).position(|&p| p < peak * 0.01).unwrap_or(max_r - 1) + 1;
    let before = radial_power[cr.saturating_sub(1)];
    let after = radial_power[(cr + 1).min(max_r - 1)];
    let cutoff_sharpness = ((before - after) / peak).max(0.0);

    let cutoff_x = (cr as f64 / cx as f64 * w as f64 / 2.0) as u32 * 2;
    let cutoff_y = (cr as f64 / cy as f64 * h as f64 / 2.0) as u32 * 2;

    let estimated = STANDARD_RESOLUTIONS
        .iter()
        .min_by_key(|&&(ww, hh)| {
            (i64::from(ww) - i64::from(cutoff_x)).abs() + (i64::from(hh) - i64::from(cutoff_y)).abs()
        })
        .copied()
        .unwrap_or((w as u32, h as u32));

    let (cw, ch) = (w as u32, h as u32);
    let (_, nh) = estimated;
    let ratio_h = if nh > 0 { f64::from(ch) / f64::from(nh) } else { 1.0 };
    let aspect_c = cw as f64 / ch.max(1) as f64;
    let aspect_n = estimated.0 as f64 / estimated.1.max(1) as f64;
    let aspect_match = (aspect_c - aspect_n).abs() <= 0.15;
    let (verdict, confidence) = judge(ratio_h, cutoff_sharpness, aspect_match);

    Ok(ResolutionResult {
        cutoff_x,
        cutoff_y,
        estimated_native: estimated,
        container: (cw, ch),
        upscale_ratio_h: ratio_h,
        verdict,
        confidence,
        sharpness,
    })
}

fn native_fallback(w: usize, h: usize, sharpness: f64) -> ResolutionResult {
    ResolutionResult {
        cutoff_x: w as u32,
        cutoff_y: h as u32,
        estimated_native: (w as u32, h as u32),
        container: (w as u32, h as u32),
        upscale_ratio_h: 1.0,
        verdict: UpscaleVerdict::Uncertain,
        confidence: 0.0,
        sharpness,
    }
}

#[cfg(feature = "fft")]
pub fn default_fft_2d(gray: &[f64], width: usize, height: usize) -> Result<Vec<Vec<f64>>> {
    use rustfft::num_complex::Complex;
    use rustfft::FftPlanner;
    let mut planner = FftPlanner::<f64>::new();
    let fft_h = planner.plan_fft_forward(width);
    let mut spectrum = vec![Complex::new(0.0, 0.0); width * height];
    let mut row_buf = vec![Complex::new(0.0, 0.0); width];
    for y in 0..height {
        for x in 0..width {
            row_buf[x] = Complex::new(gray[y * width + x], 0.0);
        }
        fft_h.process(&mut row_buf);
        spectrum[y * width..(y + 1) * width].copy_from_slice(&row_buf);
    }
    let fft_v = planner.plan_fft_forward(height);
    let mut col_buf = vec![Complex::new(0.0, 0.0); height];
    for x in 0..width {
        for y in 0..height {
            col_buf[y] = spectrum[y * width + x];
        }
        fft_v.process(&mut col_buf);
        for y in 0..height {
            spectrum[y * width + x] = col_buf[y];
        }
    }
    let cx = width / 2;
    let cy = height / 2;
    let mut mag = vec![vec![0.0_f64; width]; height];
    for y in 0..height {
        for x in 0..width {
            let (sx, sy) = ((x + cx) % width, (y + cy) % height);
            mag[sy][sx] = spectrum[y * width + x].norm();
        }
    }
    Ok(mag)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pixel_buffer::PixelBuffer;

    fn frame(gray_val: u8, w: u32, h: u32) -> Frame {
        let mut data = Vec::with_capacity((w * h * 3) as usize);
        for _ in 0..(w * h) {
            data.extend_from_slice(&[gray_val, gray_val, gray_val]);
        }
        Frame { data: PixelBuffer::new(data, w, h).unwrap(), index: 0, timestamp_ms: 0.0 }
    }
    fn dummy_fft(_: &[f64], w: usize, h: usize) -> Result<Vec<Vec<f64>>> {
        Ok(vec![vec![1.0; w]; h])
    }

    #[test]
    fn small_frame_errors() {
        assert!(detect_resolution(&frame(128, 2, 2), dummy_fft).is_err());
    }
    #[test]
    fn flat_spectrum_is_uncertain() {
        let r = detect_resolution(&frame(128, 16, 16), dummy_fft).unwrap();
        assert_eq!(r.verdict, UpscaleVerdict::Uncertain);
    }
    #[test]
    fn laplacian_flat_is_low() {
        assert!(laplacian_sharpness(&vec![0.5; 64], 8, 8) < f64::EPSILON);
    }
    #[test]
    fn laplacian_edge_is_high() {
        let mut g = vec![0.0; 64];
        for y in 0..8 {
            for x in 4..8 {
                g[y * 8 + x] = 1.0;
            }
        }
        assert!(laplacian_sharpness(&g, 8, 8) > 0.0);
    }
    #[test]
    fn judge_native() {
        assert_eq!(judge(1.0, 0.8, true).0, UpscaleVerdict::Native);
    }
    #[test]
    fn judge_upscaled() {
        assert_eq!(judge(4.5, 0.8, true).0, UpscaleVerdict::LikelyUpscaled);
    }
    #[test]
    fn judge_aspect_mismatch_is_uncertain() {
        let (v, c) = judge(2.0, 0.8, false);
        assert_eq!(v, UpscaleVerdict::Uncertain);
        assert!(c <= 0.5);
    }
    #[test]
    fn judge_low_confidence_demotes() {
        assert_eq!(judge(2.0, 0.05, true).0, UpscaleVerdict::Uncertain);
    }
}
