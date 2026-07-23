use crate::error::{Result, TvaError};
use palette::{IntoColor, Lab, Srgb};
use rgb::RGB8;

/// Comparison method with threshold.
#[derive(Debug, Clone)]
pub enum CompareMethod {
    Raw { threshold: f64 },
    CieLab { threshold: f64 },
    Ssim { threshold: f64 },
}

impl Default for CompareMethod {
    fn default() -> Self {
        Self::CieLab { threshold: 2.0 }
    }
}

/// Result of comparing two frames.
#[derive(Debug, Clone)]
pub struct CompareResult {
    pub score: f64,
    pub is_duplicate: bool,
    pub diff_pixel_ratio: f64,
}

/// Mean absolute difference per byte. Range: [0.0, 255.0].
#[must_use]
pub fn diff_raw(a: &[u8], b: &[u8]) -> f64 {
    debug_assert_eq!(a.len(), b.len());
    let sum: u64 = a.iter().zip(b).map(|(&x, &y)| u64::from(x.abs_diff(y))).sum();
    sum as f64 / a.len() as f64
}

/// Mean CIE76 Delta E across all pixels. Range: [0.0, ~100+].
#[must_use]
pub fn diff_cielab(a: &[RGB8], b: &[RGB8]) -> f64 {
    debug_assert_eq!(a.len(), b.len());
    let n = a.len();
    if n == 0 {
        return 0.0;
    }
    let sum_de: f64 = a
        .iter()
        .zip(b)
        .map(|(pa, pb)| {
            let ca: Srgb<f32> = Srgb::new(
                f32::from(pa.r) / 255.0,
                f32::from(pa.g) / 255.0,
                f32::from(pa.b) / 255.0,
            );
            let cb: Srgb<f32> = Srgb::new(
                f32::from(pb.r) / 255.0,
                f32::from(pb.g) / 255.0,
                f32::from(pb.b) / 255.0,
            );
            let la: Lab = ca.into_color();
            let lb: Lab = cb.into_color();

            let dl = la.l - lb.l;
            let da = la.a - lb.a;
            let db = la.b - lb.b;
            f64::from((dl * dl + da * da + db * db).sqrt())
        })
        .sum();
    sum_de / n as f64
}

/// SSIM via dssim-core. Range: [0.0, 1.0]. 1.0 = identical.
pub fn diff_ssim(a: &[RGB8], b: &[RGB8], width: usize, height: usize) -> Result<f64> {
    let ctx = dssim_core::Dssim::new();
    let img_a = ctx
        .create_image_rgb(a, width, height)
        .ok_or(TvaError::SsimFailed)?;
    let img_b = ctx
        .create_image_rgb(b, width, height)
        .ok_or(TvaError::SsimFailed)?;
    let (val, _maps) = ctx.compare(&img_a, &img_b);
    Ok(val.into())
}

/// Unified frame comparison.
pub fn compare_frames(
    a: &[RGB8],
    b: &[RGB8],
    width: usize,
    height: usize,
    method: &CompareMethod,
) -> Result<CompareResult> {
    if a.len() != b.len() {
        return Err(TvaError::SizeMismatch { expected: a.len(), got: b.len() });
    }
    if a.is_empty() {
        return Err(TvaError::EmptyFrame);
    }

    let (score, is_duplicate) = match method {
        CompareMethod::Raw { threshold } => {
            let a_bytes = bytemuck_cast_slice(a);
            let b_bytes = bytemuck_cast_slice(b);
            let s = diff_raw(a_bytes, b_bytes);
            (s, s < *threshold)
        }
        CompareMethod::CieLab { threshold } => {
            let s = diff_cielab(a, b);
            (s, s < *threshold)
        }
        CompareMethod::Ssim { threshold } => {
            let s = diff_ssim(a, b, width, height)?;
            (s, s > *threshold)
        }
    };

    let n = a.len();
    let diff_count = a
        .iter()
        .zip(b)
        .filter(|(pa, pb)| pa.r != pb.r || pa.g != pb.g || pa.b != pb.b)
        .count();

    Ok(CompareResult {
        score,
        is_duplicate,
        diff_pixel_ratio: diff_count as f64 / n as f64,
    })
}

/// SAFETY: RGB8 is #[repr(C)] with three u8 fields, no padding.
/// ponytail: bytemuck dep not worth it for one cast
unsafe fn bytemuck_cast_slice(pixels: &[RGB8]) -> &[u8] {
    std::slice::from_raw_parts(pixels.as_ptr().cast::<u8>(), pixels.len() * 3)
}
