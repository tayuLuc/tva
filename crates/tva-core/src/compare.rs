//! Per-frame comparison: raw pixel diff, CIELAB Delta E, SSIM.
//! Unified comparator with configurable method and threshold.

use palette::{Lab, Srgb, FromColor};

/// Mean absolute difference per byte. Range: [0, 255].
pub fn diff_raw(a: &[u8], b: &[u8]) -> f64 {
    assert_eq!(a.len(), b.len());
    let sum: u64 = a
        .iter()
        .zip(b.iter())
        .map(|(&x, &y)| x.abs_diff(y) as u64)
        .sum();
    sum as f64 / a.len() as f64
}

/// Mean CIE76 Delta E across all pixels. Range: [0, ~100+].
pub fn diff_cielab(a: &[u8], b: &[u8], width: u32, height: u32) -> f64 {
    let n = (width * height) as usize;
    let mut sum_de: f64 = 0.0;

    for i in 0..n {
        let off = i * 3;
        let ca: Srgb<u8> = Srgb::new(a[off], a[off + 1], a[off + 2]);
        let cb: Srgb<u8> = Srgb::new(b[off], b[off + 1], b[off + 2]);

        let la: Lab = Lab::from_color(ca.into_format::<f32>());
        let lb: Lab = Lab::from_color(cb.into_format::<f32>());

        let dl = la.l - lb.l;
        let da = la.a - lb.a;
        let db = la.b - lb.b;
        sum_de += (dl * dl + da * da + db * db).sqrt() as f64;
    }

    sum_de / n as f64
}

/// SSIM similarity. Range: [0, 1]. 1.0 = identical.
pub fn diff_ssim(a: &[u8], b: &[u8], width: u32, height: u32) -> f64 {
    let ctx = dssim_core::Dssim::new();
    let img_a = ctx.create_image_rgb(a, width, height).unwrap();
    let img_b = ctx.create_image_rgb(b, width, height).unwrap();
    let (ssim, _maps) = ctx.compare(&img_a, &img_b);
    ssim as f64
}

/// Comparison method selector.
#[derive(Clone, Debug)]
pub enum CompareMethod {
    Raw { threshold: f64 },
    CieLab { threshold: f64 },
    Ssim { threshold: f64 },
}

impl CompareMethod {
    pub fn default_raw() -> Self {
        CompareMethod::Raw { threshold: 1.0 }
    }
    pub fn default_cielab() -> Self {
        CompareMethod::CieLab { threshold: 2.0 }
    }
    pub fn default_ssim() -> Self {
        CompareMethod::Ssim { threshold: 0.98 }
    }
}

impl Default for CompareMethod {
    fn default() -> Self {
        Self::default_cielab()
    }
}

/// Result of comparing two frames.
#[derive(Clone, Debug)]
pub struct CompareResult {
    pub score: f64,
    pub is_duplicate: bool,
    pub diff_pixel_ratio: f64,
}

/// Unified frame comparison.
pub fn compare_frames(
    a: &[u8],
    b: &[u8],
    width: u32,
    height: u32,
    method: &CompareMethod,
) -> CompareResult {
    let (score, is_dup) = match method {
        CompareMethod::Raw { threshold } => {
            let s = diff_raw(a, b);
            (s, s < *threshold)
        }
        CompareMethod::CieLab { threshold } => {
            let s = diff_cielab(a, b, width, height);
            (s, s < *threshold)
        }
        CompareMethod::Ssim { threshold } => {
            let s = diff_ssim(a, b, width, height);
            (s, s > *threshold)
        }
    };

    let n = (width * height) as usize;
    let diff_count: u64 = (0..n)
        .map(|i| {
            let off = i * 3;
            if a[off] != b[off] || a[off + 1] != b[off + 1] || a[off + 2] != b[off + 2] {
                1
            } else {
                0
            }
        })
        .sum();

    CompareResult {
        score,
        is_duplicate: is_dup,
        diff_pixel_ratio: diff_count as f64 / n as f64,
    }
}
