use crate::error::{Result, TvaError};
use crate::frame::Frame;
use bytemuck::cast_slice;
use palette::{IntoColor, Lab, Oklab, Srgb};
use rgb::RGB8;

#[derive(Debug, Clone)]
pub enum CompareMethod {
    Raw { threshold: f64 },
    /// Oklab Delta E — perceptual, ~3-5× cheaper than CIE76 Lab.
    Oklab { threshold: f64 },
    /// Full CIE76 Lab Delta E — slower but standard.
    CieLab { threshold: f64 },
    Ssim { threshold: f64 },
}

impl Default for CompareMethod {
    fn default() -> Self {
        Self::Oklab { threshold: 2.0 }
    }
}

#[derive(Debug, Clone)]
pub struct CompareResult {
    pub score: f64,
    pub is_duplicate: bool,
    pub diff_pixel_ratio: f64,
}

/// Mean absolute byte difference. Range [0, 255].
#[must_use]
pub fn diff_raw(a: &[u8], b: &[u8]) -> f64 {
    debug_assert_eq!(a.len(), b.len());
    let sum: u64 = a.iter().zip(b).map(|(&x, &y)| u64::from(x.abs_diff(y))).sum();
    sum as f64 / a.len() as f64
}

/// Mean Oklab Delta E — perceptual, cheaper than full Lab.
#[must_use]
pub fn diff_oklab(a: &Frame, b: &Frame) -> f64 {
    debug_assert_eq!(a.data.len(), b.data.len());
    let n = a.data.len();
    if n == 0 {
        return 0.0;
    }
    let sum: f64 = a.data.iter().zip(&b.data)
        .map(|(pa, pb)| {
            let ca: Srgb<f32> = Srgb::new(f32::from(pa.r)/255.0, f32::from(pa.g)/255.0, f32::from(pa.b)/255.0);
            let cb: Srgb<f32> = Srgb::new(f32::from(pb.r)/255.0, f32::from(pb.g)/255.0, f32::from(pb.b)/255.0);
            let la: Oklab = ca.into_color();
            let lb: Oklab = cb.into_color();
            let (dl, da, db) = (la.l - lb.l, la.a - lb.a, la.b - lb.b);
            f64::from((dl*dl + da*da + db*db).sqrt())
        })
        .sum();
    sum / n as f64
}

/// Mean CIE76 Lab Delta E — full perceptual reference.
#[must_use]
pub fn diff_cielab(a: &Frame, b: &Frame) -> f64 {
    debug_assert_eq!(a.data.len(), b.data.len());
    let n = a.data.len();
    if n == 0 { return 0.0; }
    let sum: f64 = a.data.iter().zip(&b.data)
        .map(|(pa, pb)| {
            let ca: Srgb<f32> = Srgb::new(f32::from(pa.r)/255.0, f32::from(pa.g)/255.0, f32::from(pa.b)/255.0);
            let cb: Srgb<f32> = Srgb::new(f32::from(pb.r)/255.0, f32::from(pb.g)/255.0, f32::from(pb.b)/255.0);
            let la: Lab = ca.into_color();
            let lb: Lab = cb.into_color();
            let (dl, da, db) = (la.l - lb.l, la.a - lb.a, la.b - lb.b);
            f64::from((dl*dl + da*da + db*db).sqrt())
        })
        .sum();
    sum / n as f64
}

/// SSIM via dssim-core. Range [0, 1], 1.0 = identical.
pub fn diff_ssim(a: &Frame, b: &Frame) -> Result<f64> {
    let ctx = dssim_core::Dssim::new();
    let w = a.width as usize;
    let h = a.height as usize;
    let img_a = ctx.create_image_rgb(&a.data, w, h).ok_or(TvaError::SsimFailed)?;
    let img_b = ctx.create_image_rgb(&b.data, w, h).ok_or(TvaError::SsimFailed)?;
    let (val, _) = ctx.compare(&img_a, &img_b);
    Ok(val.into())
}

/// Unified frame comparison.
pub fn compare_frames(a: &Frame, b: &Frame, method: &CompareMethod) -> Result<CompareResult> {
    if a.data.len() != b.data.len() {
        return Err(TvaError::SizeMismatch { expected: a.data.len(), got: b.data.len() });
    }
    if a.data.is_empty() {
        return Err(TvaError::EmptyFrame);
    }

    let (score, is_duplicate) = match method {
        CompareMethod::Raw { threshold } => {
            let s = diff_raw(cast_slice(&a.data), cast_slice(&b.data));
            (s, s < *threshold)
        }
        CompareMethod::Oklab { threshold } => (diff_oklab(a, b), |s: f64| s < *threshold),
        CompareMethod::CieLab { threshold } => (diff_cielab(a, b), |s: f64| s < *threshold),
        CompareMethod::Ssim { threshold } => (diff_ssim(a, b)?, |s: f64| s > *threshold),
    };

    let diff_count = a.data.iter().zip(&b.data)
        .filter(|(pa, pb)| pa.r != pb.r || pa.g != pb.g || pa.b != pb.b)
        .count();

    Ok(CompareResult {
        score,
        is_duplicate,
        diff_pixel_ratio: diff_count as f64 / a.data.len() as f64,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frame::Frame;

    fn f(pixels: Vec<RGB8>, w: u32, h: u32) -> Frame {
        Frame { data: pixels, width: w, height: h, index: 0, timestamp_ms: 0.0 }
    }

    #[test]
    fn identical_raw_score_is_zero() {
        let fa = f(vec![RGB8::new(128, 64, 32); 100], 10, 10);
        assert_eq!(diff_raw(cast_slice(&fa.data), cast_slice(&fa.data)), 0.0);
    }

    #[test]
    fn abs_diff_no_underflow() {
        let d = diff_raw(&[0u8; 4], &[255u8; 4]);
        assert!((d - 255.0).abs() < f64::EPSILON);
    }

    #[test]
    fn identical_frames_are_duplicates_raw() {
        let fa = f(vec![RGB8::new(128, 64, 32); 100], 10, 10);
        let r = compare_frames(&fa, &fa, &CompareMethod::Raw { threshold: 1.0 }).unwrap();
        assert!(r.is_duplicate);
        assert_eq!(r.score, 0.0);
    }

    #[test]
    fn different_frames_are_not_duplicates() {
        let a = f(vec![RGB8::new(0, 0, 0); 100], 10, 10);
        let b = f(vec![RGB8::new(255, 255, 255); 100], 10, 10);
        let r = compare_frames(&a, &b, &CompareMethod::Raw { threshold: 1.0 }).unwrap();
        assert!(!r.is_duplicate);
    }

    #[test]
    fn empty_frame_errors() {
        let a = f(vec![], 0, 0);
        let r = compare_frames(&a, &a, &CompareMethod::default());
        assert!(r.is_err());
    }

    #[test]
    fn ssim_identical_is_one() {
        let fa = f(vec![RGB8::new(100, 100, 100); 400], 20, 20);
        let s = diff_ssim(&fa, &fa).unwrap();
        assert!((s - 1.0).abs() < 0.001);
    }

    #[test]
    fn oklab_agrees_on_extremes() {
        let a = f(vec![RGB8::new(0, 0, 0); 100], 10, 10);
        let b = f(vec![RGB8::new(255, 255, 255); 100], 10, 10);
        let raw = compare_frames(&a, &b, &CompareMethod::Raw { threshold: 1.0 }).unwrap();
        let ok = compare_frames(&a, &b, &CompareMethod::Oklab { threshold: 2.0 }).unwrap();
        assert!(!raw.is_duplicate);
        assert!(!ok.is_duplicate);
    }
}
