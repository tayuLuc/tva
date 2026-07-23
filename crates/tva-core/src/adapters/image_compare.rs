use crate::error::{Result, TvaError};
use crate::pixel_buffer::PixelBuffer;
use crate::traits::FrameComparator;
use image::DynamicImage;

macro_rules! to_dynamic {
    ($p:expr) => {
        DynamicImage::ImageRgb8(
            image::RgbImage::from_raw($p.width(), $p.height(), $p.as_bytes().to_vec())
                .expect("PixelBuffer validated dimensions"),
        )
    };
}

pub struct SsimComparator;

impl FrameComparator for SsimComparator {
    fn compare(&self, a: &PixelBuffer, b: &PixelBuffer) -> Result<f64> {
        let img_a = to_dynamic!(a);
        let img_b = to_dynamic!(b);
        let result = image_compare::rgb_similarity_structure(&img_a, &img_b)
            .map_err(TvaError::CompareFailed)?;
        Ok(result.score)
    }
    fn higher_is_similar(&self) -> bool { true }
    fn name(&self) -> &'static str { "ssim" }
}

pub struct HybridComparator;

impl FrameComparator for HybridComparator {
    fn compare(&self, a: &PixelBuffer, b: &PixelBuffer) -> Result<f64> {
        let img_a = to_dynamic!(a);
        let img_b = to_dynamic!(b);
        let result = image_compare::rgb_hybrid_compare(&img_a, &img_b)
            .map_err(TvaError::CompareFailed)?;
        Ok(result.score)
    }
    fn higher_is_similar(&self) -> bool { true }
    fn name(&self) -> &'static str { "hybrid" }
}
