use crate::error::{Result, TvaError};
use crate::frame::Frame;
use image::RgbImage;
use image_similarity::{compare_images, Metric, ImageScore};

#[derive(Debug, Clone)]
pub enum CompareMethod {
    Sad { threshold: f64 },
    Ssim { threshold: f64 },
    MsSsim { threshold: f64 },
    Psnr { threshold: f64 },
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

fn frame_to_image(frame: &Frame) -> Result<RgbImage> {
    let raw: Vec<u8> = frame.data.iter().flat_map(|p| [p.r, p.g, p.b]).collect();
    RgbImage::from_raw(frame.width, frame.height, raw)
        .ok_or(TvaError::CompareFailed("invalid frame dimensions".into()))
}

pub fn compare_frames(a: &Frame, b: &Frame, method: &CompareMethod) -> Result<CompareResult> {
    let img_a = frame_to_image(a)?;
    let img_b = frame_to_image(b)?;

    let (metric, higher_is_similar) = match method {
        CompareMethod::Sad { .. } => (Metric::Sad, false),
        CompareMethod::Ssim { .. } => (Metric::Ssim, true),
        CompareMethod::MsSsim { .. } => (Metric::MsSsim, true),
        CompareMethod::Psnr { .. } => (Metric::Psnr, true),
    };

    let result =
        compare_images(&img_a, &img_b, &metric).map_err(|e| TvaError::CompareFailed(e.to_string()))?;

    let score = result.score();
    let threshold = match method {
        CompareMethod::Sad { threshold }
        | CompareMethod::Ssim { threshold }
        | CompareMethod::MsSsim { threshold }
        | CompareMethod::Psnr { threshold } => *threshold,
    };

    Ok(CompareResult {
        score,
        is_duplicate: if higher_is_similar { score > threshold } else { score < threshold },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frame::Frame;
    use rgb::RGB8;

    fn f(pixels: Vec<RGB8>, w: u32, h: u32) -> Frame {
        Frame { data: pixels, width: w, height: h, index: 0, timestamp_ms: 0.0 }
    }

    #[test]
    fn identical_sad_is_zero() {
        let a = f(vec![RGB8::new(128, 64, 32); 100], 10, 10);
        let r = compare_frames(&a, &a, &CompareMethod::Sad { threshold: 1.0 }).unwrap();
        assert!(r.is_duplicate);
        assert_eq!(r.score, 0.0);
    }

    #[test]
    fn opposite_frames_not_duplicate() {
        let a = f(vec![RGB8::new(0, 0, 0); 100], 10, 10);
        let b = f(vec![RGB8::new(255, 255, 255); 100], 10, 10);
        let r = compare_frames(&a, &b, &CompareMethod::Sad { threshold: 1.0 }).unwrap();
        assert!(!r.is_duplicate);
    }

    #[test]
    fn identical_ssim_is_one() {
        let a = f(vec![RGB8::new(100, 100, 100); 400], 20, 20);
        let r = compare_frames(&a, &a, &CompareMethod::Ssim { threshold: 0.99 }).unwrap();
        assert!(r.is_duplicate);
        assert!((r.score - 1.0).abs() < 0.001);
    }

    #[test]
    fn ssim_detects_difference() {
        let a = f(vec![RGB8::new(0, 0, 0); 400], 20, 20);
        let b = f(vec![RGB8::new(255, 255, 255); 400], 20, 20);
        let r = compare_frames(&a, &b, &CompareMethod::Ssim { threshold: 0.5 }).unwrap();
        assert!(!r.is_duplicate);
        assert!(r.score < 0.1);
    }

    #[test]
    fn empty_frame_errors() {
        let a = f(vec![], 0, 0);
        assert!(compare_frames(&a, &a, &CompareMethod::default()).is_err());
    }

    #[test]
    fn psnr_identical_is_high() {
        let a = f(vec![RGB8::new(128, 128, 128); 400], 20, 20);
        let r = compare_frames(&a, &a, &CompareMethod::Psnr { threshold: 50.0 }).unwrap();
        assert!(r.is_duplicate);
        assert!(r.score > 100.0);
    }
}
