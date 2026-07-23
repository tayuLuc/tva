use serde::Serialize;

use crate::error::{Result, TvaError};
use crate::frame::Frame;

#[derive(Debug, Clone, Serialize)]
pub struct ResolutionResult {
    pub cutoff_x: u32,
    pub cutoff_y: u32,
    pub estimated_native: (u32, u32),
}

const STANDARD_RESOLUTIONS: [(u32, u32); 6] = [
    (640, 360), (1280, 720), (1440, 810),
    (1560, 873), (1600, 900), (1920, 1080),
];

/// Raw 2D FFT resolution detection (core: no rustfft dependency).
/// Actual FFT is parameterized via a closure to keep the core dependency-free.
/// The `fft` feature provides a default implementation.
pub fn detect_resolution<F>(frame: &Frame, fft_2d: F) -> Result<ResolutionResult>
where
    F: Fn(&[f64], usize, usize) -> Result<Vec<Vec<f64>>>,
{
    let w = frame.data.width() as usize;
    let h = frame.data.height() as usize;
    if w < 4 || h < 4 { return Err(TvaError::FftFailed("frame too small".into())); }

    // Grayscale via weighted RGB (Rec. 709)
    let gray: Vec<f64> = frame.data.as_bytes().chunks_exact(3).map(|c| {
        (0.2126 * f64::from(c[0]) + 0.7152 * f64::from(c[1]) + 0.0722 * f64::from(c[2])) / 255.0
    }).collect();

    // Invoke FFT — caller provides implementation
    let magnitude = fft_2d(&gray, w, h)?;

    let cx = w / 2; let cy = h / 2;
    let max_r = cx.max(cy);
    let mut radial_power = vec![0.0f64; max_r]; let mut radial_count = vec![0u64; max_r];

    for y in 0..h { for x in 0..w {
        let r = (((x as i64 - cx as i64).pow(2) + (y as i64 - cy as i64).pow(2)) as f64).sqrt() as usize;
        if r < max_r { radial_power[r] += magnitude[y * w + x]; radial_count[r] += 1; }
    }}
    for (p, c) in radial_power.iter_mut().zip(&radial_count) { if *c > 0 { *p /= *c as f64; } }

    let peak = radial_power.iter().skip(1).cloned().fold(0.0f64, f64::max);
    if peak == 0.0 {
        return Ok(ResolutionResult { cutoff_x: w as u32, cutoff_y: h as u32, estimated_native: (w as u32, h as u32) });
    }
    let cr = radial_power.iter().skip(1).position(|&p| p < peak * 0.01).unwrap_or(max_r - 1) + 1;
    let cx_f = (cr as f64 / cx as f64 * w as f64 / 2.0) as u32 * 2;
    let cy_f = (cr as f64 / cy as f64 * h as f64 / 2.0) as u32 * 2;
    let est = STANDARD_RESOLUTIONS.iter().min_by_key(|&&(ww, hh)| (i64::from(ww) - i64::from(cx_f)).abs() + (i64::from(hh) - i64::from(cy_f)).abs()).copied().unwrap_or((w as u32, h as u32));
    Ok(ResolutionResult { cutoff_x: cx_f, cutoff_y: cy_f, estimated_native: est })
}

/// FFT via rustfft (feature = "fft").
#[cfg(feature = "fft")]
pub fn default_fft_2d(gray: &[f64], width: usize, height: usize) -> Result<Vec<Vec<f64>>> {
    use rustfft::num_complex::Complex;
    use rustfft::FftPlanner;

    let mut planner = FftPlanner::<f64>::new();
    let fft_h = planner.plan_fft_forward(width);
    let mut spectrum = vec![Complex::new(0.0, 0.0); width * height];
    let mut row_buf = vec![Complex::new(0.0, 0.0); width];

    for y in 0..height {
        for x in 0..width { row_buf[x] = Complex::new(gray[y * width + x], 0.0); }
        fft_h.process(&mut row_buf);
        spectrum[y * width..(y + 1) * width].copy_from_slice(&row_buf);
    }
    let fft_v = planner.plan_fft_forward(height);
    let mut col_buf = vec![Complex::new(0.0, 0.0); height];
    for x in 0..width {
        for y in 0..height { col_buf[y] = spectrum[y * width + x]; }
        fft_v.process(&mut col_buf);
        for y in 0..height { spectrum[y * width + x] = col_buf[y]; }
    }
    // Return magnitude spectrum (fftshifted)
    let cx = width / 2; let cy = height / 2;
    let mut mag = vec![vec![0.0f64; width]; height];
    for y in 0..height { for x in 0..width {
        let (sx, sy) = ((x + cx) % width, (y + cy) % height);
        mag[sy][sx] = spectrum[y * width + x].norm();
    }}
    Ok(mag)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frame::Frame;
    use crate::pixel_buffer::PixelBuffer;

    fn dummy_fft(_gray: &[f64], _w: usize, _h: usize) -> Result<Vec<Vec<f64>>> {
        Ok(vec![vec![1.0; _w]; _h])
    }

    #[test]
    fn small_frame_errors() {
        let f = Frame { data: PixelBuffer::new(vec![0u8; 12], 2, 2).unwrap(), index: 0, timestamp_ms: 0.0 };
        assert!(detect_resolution(&f, dummy_fft).is_err());
    }
}
