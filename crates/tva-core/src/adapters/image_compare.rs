//! Адаптер `image-compare`. Активируется feature `compare-image`.

use crate::error::{Result, TvaError};
use crate::pixel_buffer::PixelBuffer;
use crate::traits::FrameComparator;

pub struct SsimComparator {
    pub threshold: f64,
}

impl FrameComparator for SsimComparator {
    fn compare(&self, a: &PixelBuffer, b: &PixelBuffer) -> Result<f64> {
        let img_a = a.to_dynamic_image();
        let img_b = b.to_dynamic_image();
        image_compare::rgb_similarity_structure(&img_a, &img_b, image_compare::Metric::Ssim)
            .map_err(TvaError::CompareFailed)
    }
    fn higher_is_similar(&self) -> bool { true }
    fn name(&self) -> &'static str { "ssim" }
}

pub struct MsSsimComparator {
    pub threshold: f64,
}

impl FrameComparator for MsSsimComparator {
    fn compare(&self, a: &PixelBuffer, b: &PixelBuffer) -> Result<f64> {
        let img_a = a.to_dynamic_image();
        let img_b = b.to_dynamic_image();
        image_compare::rgb_similarity_structure(&img_a, &img_b, image_compare::Metric::Mssim)
            .map_err(TvaError::CompareFailed)
    }
    fn higher_is_similar(&self) -> bool { true }
    fn name(&self) -> &'static str { "mssim" }
}

pub struct HybridComparator {
    pub threshold: f64,
}

impl FrameComparator for HybridComparator {
    fn compare(&self, a: &PixelBuffer, b: &PixelBuffer) -> Result<f64> {
        let img_a = a.to_dynamic_image();
        let img_b = b.to_dynamic_image();
        image_compare::rgb_similarity_structure(&img_a, &img_b, image_compare::Metric::Hybrid)
            .map_err(TvaError::CompareFailed)
    }
    fn higher_is_similar(&self) -> bool { true }
    fn name(&self) -> &'static str { "hybrid" }
}

pub struct MadComparator {
    pub threshold: f64,
}

impl FrameComparator for MadComparator {
    fn compare(&self, a: &PixelBuffer, b: &PixelBuffer) -> Result<f64> {
        let img_a = a.to_dynamic_image();
        let img_b = b.to_dynamic_image();
        image_compare::rgb_diffing_structure(&img_a, &img_b, image_compare::Metric::Mad)
            .map_err(TvaError::CompareFailed)
    }
    fn higher_is_similar(&self) -> bool { false }
    fn name(&self) -> &'static str { "mad" }
}

pub struct SadComparator {
    pub threshold: f64,
}

impl FrameComparator for SadComparator {
    fn compare(&self, a: &PixelBuffer, b: &PixelBuffer) -> Result<f64> {
        let img_a = a.to_dynamic_image();
        let img_b = b.to_dynamic_image();
        image_compare::rgb_diffing_structure(&img_a, &img_b, image_compare::Metric::Sad)
            .map_err(TvaError::CompareFailed)
    }
    fn higher_is_similar(&self) -> bool { false }
    fn name(&self) -> &'static str { "sad" }
}
