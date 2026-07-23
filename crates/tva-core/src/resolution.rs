use crate::error::{Result, TvaError};
use rgb::RGB8;
use rustfft::num_complex::Complex;
use rustfft::FftPlanner;

#[derive(Debug, Clone)]
pub struct ResolutionResult {
    pub cutoff_x: u32,
    pub cutoff_y: u32,
    pub estimated_native: (u32, u32),
}

const STANDARD_RESOLUTIONS: [(u32, u32); 6] = [
    (640, 360),
    (1280, 720),
    (1440, 810),
    (1560, 873),
    (1600, 900),
    (1920, 1080),
];

/// Detect native resolution from a grayscale frame via 2D FFT.
pub fn detect_resolution(gray: &[f64], width: usize, height: usize) -> Result<ResolutionResult> {
    if gray.len() != width * height {
        return Err(TvaError::SizeMismatch { expected: width * height, got: gray.len() });
    }
    if width < 4 || height < 4 {
        return Err(TvaError::FftFailed("frame too small".into()));
    }

    let mut planner = FftPlanner::<f64>::new();
    let fft_h = planner.plan_fft_forward(width);
    let mut spectrum: Vec<Complex<f64>> = vec![Complex::new(0.0, 0.0); width * height];
    let mut row_buf: Vec<Complex<f64>> = vec![Complex::new(0.0, 0.0); width];

    for y in 0..height {
        for x in 0..width {
            row_buf[x] = Complex::new(gray[y * width + x], 0.0);
        }
        fft_h.process(&mut row_buf);
        spectrum[y * width..(y + 1) * width].copy_from_slice(&row_buf);
    }

    let fft_v = planner.plan_fft_forward(height);
    let mut col_buf: Vec<Complex<f64>> = vec![Complex::new(0.0, 0.0); height];
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
    let max_r = cx.max(cy);
    let mut radial_power = vec![0.0f64; max_r];
    let mut radial_count = vec![0u64; max_r];

    for y in 0..height {
        for x in 0..width {
            let sx = (x + cx) % width;
            let sy = (y + cy) % height;
            let mag = spectrum[sy * width + sx].norm();
            let dx = x as i64 - cx as i64;
            let dy = y as i64 - cy as i64;
            let r = ((dx * dx + dy * dy) as f64).sqrt() as usize;
            if r < max_r {
                radial_power[r] += mag;
                radial_count[r] += 1;
            }
        }
    }

    for (power, count) in radial_power.iter_mut().zip(&radial_count) {
        if *count > 0 {
            *power /= *count as f64;
        }
    }

    let peak = radial_power.iter().skip(1).cloned().fold(0.0f64, f64::max);
    if peak == 0.0 {
        return Ok(ResolutionResult {
            cutoff_x: width as u32,
            cutoff_y: height as u32,
            estimated_native: (width as u32, height as u32),
        });
    }

    let cutoff_r = radial_power
        .iter()
        .skip(1)
        .position(|&p| p < peak * 0.01)
        .unwrap_or(max_r - 1)
        + 1;

    let cutoff_x = (cutoff_r as f64 / cx as f64 * width as f64 / 2.0) as u32 * 2;
    let cutoff_y = (cutoff_r as f64 / cy as f64 * height as f64 / 2.0) as u32 * 2;

    let estimated = STANDARD_RESOLUTIONS
        .iter()
        .min_by_key(|&&(w, h)| {
            let dw = i64::from(w) - i64::from(cutoff_x);
            let dh = i64::from(h) - i64::from(cutoff_y);
            dw.abs() + dh.abs()
        })
        .copied()
        .unwrap_or((width as u32, height as u32));

    Ok(ResolutionResult { cutoff_x, cutoff_y, estimated_native: estimated })
}

/// Convert RGB8 to grayscale [0.0, 1.0] (Rec. 709 luminance).
#[must_use]
pub fn rgb_to_gray(pixels: &[RGB8]) -> Vec<f64> {
    pixels.iter().map(|p| {
        (0.2126 * f64::from(p.r) + 0.7152 * f64::from(p.g) + 0.0722 * f64::from(p.b)) / 255.0
    }).collect()
}
