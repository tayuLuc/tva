use crate::error::{Result, TvaError};
use crate::frame::Frame;
use rustfft::num_complex::Complex;
use rustfft::FftPlanner;
use serde::Serialize;

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

pub fn detect_resolution(frame: &Frame) -> Result<ResolutionResult> {
    let width = frame.width() as usize;
    let height = frame.height() as usize;
    if width < 4 || height < 4 {
        return Err(TvaError::FftFailed("frame too small".into()));
    }

    let gray_img = frame.data.to_luma8();
    let gray: Vec<f64> = gray_img.as_raw().iter().map(|&p| p as f64 / 255.0).collect();

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

    let cx = width / 2;
    let cy = height / 2;
    let max_r = cx.max(cy);
    let mut radial_power = vec![0.0f64; max_r];
    let mut radial_count = vec![0u64; max_r];

    for y in 0..height {
        for x in 0..width {
            let (sx, sy) = ((x + cx) % width, (y + cy) % height);
            let mag = spectrum[sy * width + sx].norm();
            let r = (((x as i64 - cx as i64).pow(2) + (y as i64 - cy as i64).pow(2)) as f64).sqrt() as usize;
            if r < max_r { radial_power[r] += mag; radial_count[r] += 1; }
        }
    }
    for (p, c) in radial_power.iter_mut().zip(&radial_count) { if *c > 0 { *p /= *c as f64; } }

    let peak = radial_power.iter().skip(1).cloned().fold(0.0f64, f64::max);
    if peak == 0.0 {
        return Ok(ResolutionResult {
            cutoff_x: width as u32, cutoff_y: height as u32,
            estimated_native: (width as u32, height as u32),
        });
    }

    let cutoff_r = radial_power.iter().skip(1).position(|&p| p < peak * 0.01).unwrap_or(max_r - 1) + 1;
    let cutoff_x = (cutoff_r as f64 / cx as f64 * width as f64 / 2.0) as u32 * 2;
    let cutoff_y = (cutoff_r as f64 / cy as f64 * height as f64 / 2.0) as u32 * 2;

    let estimated = STANDARD_RESOLUTIONS.iter()
        .min_by_key(|&&(w, h)| (i64::from(w) - i64::from(cutoff_x)).abs() + (i64::from(h) - i64::from(cutoff_y)).abs())
        .copied().unwrap_or((width as u32, height as u32));

    Ok(ResolutionResult { cutoff_x, cutoff_y, estimated_native: estimated })
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{DynamicImage, Rgb, RgbImage};
    use crate::frame::Frame;

    #[test]
    fn small_frame_errors() {
        let f = Frame { data: DynamicImage::ImageRgb8(RgbImage::from_pixel(2, 2, Rgb([0, 0, 0]))), index: 0, timestamp_ms: 0.0 };
        assert!(detect_resolution(&f).is_err());
    }

    #[test]
    fn uniform_frame_ok() {
        let f = Frame { data: DynamicImage::ImageRgb8(RgbImage::from_pixel(1920, 1080, Rgb([128, 128, 128]))), index: 0, timestamp_ms: 0.0 };
        let r = detect_resolution(&f).unwrap();
        assert_eq!(r.estimated_native, (1920, 1080));
    }
}
