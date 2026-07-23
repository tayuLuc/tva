use image_compare::{rgb_diffing_structure, rgb_similarity_structure, Metric};

use crate::error::{Result, TvaError};
use crate::frame::Frame;

#[derive(Debug, Clone)]
pub enum CompareMethod {
    Mad { threshold: f64 },
    Sad { threshold: f64 },
    Ssim { threshold: f64 },
    MsSsim { threshold: f64 },
    Hybrid { threshold: f64 },
}

impl Default for CompareMethod {
    fn default() -> Self {
        Self::Ssim { threshold: 0.98 }
    }
}

#[derive(Debug, Clone)]
pub struct CompareResult {
    pub score: f64,
    pub is_duplicate: bool,
}

pub fn compare_frames(a: &Frame, b: &Frame, method: &CompareMethod) -> Result<CompareResult> {
    let (score, is_duplicate) = match method {
        CompareMethod::Mad { threshold } => {
            let s = rgb_diffing_structure(&a.data, &b.data, Metric::Mad)
                .map_err(TvaError::CompareFailed)?;
            (s, s < *threshold)
        }
        CompareMethod::Sad { threshold } => {
            let s = rgb_diffing_structure(&a.data, &b.data, Metric::Sad)
                .map_err(TvaError::CompareFailed)?;
            (s, s < *threshold)
        }
        CompareMethod::Ssim { threshold } => {
            let s = rgb_similarity_structure(&a.data, &b.data, Metric::Ssim)
                .map_err(TvaError::CompareFailed)?;
            (s, s > *threshold)
        }
        CompareMethod::MsSsim { threshold } => {
            let s = rgb_similarity_structure(&a.data, &b.data, Metric::Mssim)
                .map_err(TvaError::CompareFailed)?;
            (s, s > *threshold)
        }
        CompareMethod::Hybrid { threshold } => {
            let s = rgb_similarity_structure(&a.data, &b.data, Metric::Hybrid)
                .map_err(TvaError::CompareFailed)?;
            (s, s > *threshold)
        }
    };

    Ok(CompareResult { score, is_duplicate })
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{DynamicImage, Rgb, RgbImage};
    use crate::frame::Frame;

    fn solid(r: u8, g: u8, b: u8, w: u32, h: u32, idx: u64) -> Frame {
        Frame { data: DynamicImage::ImageRgb8(RgbImage::from_pixel(w, h, Rgb([r, g, b]))), index: idx, timestamp_ms: 0.0 }
    }

    #[test]
    fn identical_ssim_is_duplicate() {
        let a = solid(128, 64, 32, 16, 16, 0);
        let r = compare_frames(&a, &a, &CompareMethod::Ssim { threshold: 0.98 }).unwrap();
        assert!(r.is_duplicate);
        assert!(r.score > 0.99);
    }

    #[test]
    fn different_ssim_not_duplicate() {
        let a = solid(0, 0, 0, 16, 16, 0);
        let b = solid(255, 255, 255, 16, 16, 1);
        let r = compare_frames(&a, &b, &CompareMethod::Ssim { threshold: 0.98 }).unwrap();
        assert!(!r.is_duplicate);
    }

    #[test]
    fn mad_identical_is_zero() {
        let a = solid(100, 100, 100, 8, 8, 0);
        let r = compare_frames(&a, &a, &CompareMethod::Mad { threshold: 2.0 }).unwrap();
        assert!(r.is_duplicate);
        assert!(r.score < f64::EPSILON);
    }

    #[test]
    fn sad_opposite_is_nonzero() {
        let a = solid(0, 0, 0, 8, 8, 0);
        let b = solid(255, 255, 255, 8, 8, 1);
        let r = compare_frames(&a, &b, &CompareMethod::Sad { threshold: 1.0 }).unwrap();
        assert!(!r.is_duplicate);
        assert!(r.score > 0.0);
    }

    #[test]
    fn hybrid_identical() {
        let a = solid(100, 100, 100, 16, 16, 0);
        let r = compare_frames(&a, &a, &CompareMethod::Hybrid { threshold: 0.95 }).unwrap();
        assert!(r.is_duplicate);
    }
}
