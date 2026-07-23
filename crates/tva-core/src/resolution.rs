//! Определение нативного разрешения видео через FFT-based cutoff.
//! Берём яркостный канал, 2D FFT, радиальное усреднение,
//! ищем частоту среза по порогу мощности, маппим на стандартные разрешения.

use rustfft::FftPlanner;
use num_complex::Complex;

/// Результат определения разрешения.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
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

/// Детектировать нативное разрешение из grayscale-кадра через 2D FFT.
///
/// 1. FFT по строкам, затем по столбцам.
/// 2. Амплитудный спектр → радиальное усреднение → 1D профиль.
/// 3. Частота среза: где мощность падает ниже 1% от пика.
/// 4. Ближайшее стандартное разрешение.
pub fn detect_resolution(gray: &[f64], width: usize, height: usize) -> ResolutionResult {
    let mut planner = FftPlanner::<f64>::new();
    let fft_h = planner.plan_fft_forward(width);

    // 1. FFT по строкам
    let mut spectrum = vec![Complex::new(0.0, 0.0); width * height];
    for y in 0..height {
        let mut row: Vec<Complex<f64>> = (0..width)
            .map(|x| Complex::new(gray[y * width + x], 0.0))
            .collect();
        fft_h.process(&mut row);
        for x in 0..width {
            spectrum[y * width + x] = row[x];
        }
    }

    // 2. FFT по столбцам
    let fft_v = planner.plan_fft_forward(height);
    for x in 0..width {
        let mut col: Vec<Complex<f64>> = (0..height)
            .map(|y| spectrum[y * width + x])
            .collect();
        fft_v.process(&mut col);
        for y in 0..height {
            spectrum[y * width + x] = col[y];
        }
    }

    // 3. Радиальное усреднение (центрированный спектр)
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
            if r < radial_power.len() {
                radial_power[r] += mag;
                radial_count[r] += 1;
            }
        }
    }

    for i in 0..radial_power.len() {
        if radial_count[i] > 0 {
            radial_power[i] /= radial_count[i] as f64;
        }
    }

    // 4. Частота среза: где мощность < 1% от пика
    let peak = radial_power[1..].iter().cloned().fold(0.0f64, f64::max);
    let threshold = peak * 0.01;
    let mut cutoff_r = radial_power.len();
    for (i, &p) in radial_power.iter().enumerate().skip(1) {
        if p < threshold {
            cutoff_r = i;
            break;
        }
    }

    // 5. Маппинг на ближайшее стандартное разрешение
    let cutoff_x = (cutoff_r as f64 / cx as f64 * (width as f64 / 2.0)) as u32 * 2;
    let cutoff_y = (cutoff_r as f64 / cy as f64 * (height as f64 / 2.0)) as u32 * 2;

    let estimated = STANDARD_RESOLUTIONS
        .iter()
        .min_by_key(|&&(w, h)| {
            let dw = (w as i64 - cutoff_x as i64).abs();
            let dh = (h as i64 - cutoff_y as i64).abs();
            dw + dh
        })
        .copied()
        .unwrap_or((width as u32, height as u32));

    ResolutionResult {
        cutoff_x,
        cutoff_y,
        estimated_native: estimated,
    }
}

/// Конвертировать RGB в grayscale (simple luminance).
pub fn rgb_to_gray(rgb: &[u8], width: u32, height: u32) -> Vec<f64> {
    let n = (width * height) as usize;
    let mut gray = vec![0.0f64; n];
    for i in 0..n {
        let off = i * 3;
        let r = rgb[off] as f64;
        let g = rgb[off + 1] as f64;
        let b = rgb[off + 2] as f64;
        gray[i] = 0.299 * r + 0.587 * g + 0.114 * b;
    }
    gray
}
