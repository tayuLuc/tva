//! Адаптер `image-compare`. Feature `compare-image`.
//!
//! Comparator'ы stateless (порог дубликата живёт в
//! `PipelineConfig.duplicate_threshold`), только считают score.
//! `metric_by_name` — фабрика, возвращающая `(comparator, default_threshold)`.

use crate::error::{Result, TvaError};
use crate::pixel_buffer::PixelBuffer;
use crate::traits::FrameComparator;
use image::RgbImage;
use image_compare::Algorithm;

fn to_rgb(p: &PixelBuffer) -> RgbImage {
    RgbImage::from_raw(p.width(), p.height(), p.as_bytes().to_vec()).expect("PixelBuffer validated dimensions")
}

pub struct SsimComparator;
pub struct HybridComparator;

impl FrameComparator for SsimComparator {
    fn compare(&self, a: &PixelBuffer, b: &PixelBuffer) -> Result<f64> {
        let (ia, ib) = (to_rgb(a), to_rgb(b));
        image_compare::rgb_similarity_structure(&Algorithm::MSSIMSimple, &ia, &ib)
            .map(|r| r.score)
            .map_err(|e| TvaError::CompareFailed(e.to_string()))
    }
    fn higher_is_similar(&self) -> bool {
        true
    }
    fn name(&self) -> &'static str {
        "ssim"
    }
}

impl FrameComparator for HybridComparator {
    fn compare(&self, a: &PixelBuffer, b: &PixelBuffer) -> Result<f64> {
        let (ia, ib) = (to_rgb(a), to_rgb(b));
        image_compare::rgb_hybrid_compare(&ia, &ib).map(|r| r.score).map_err(|e| TvaError::CompareFailed(e.to_string()))
    }
    fn higher_is_similar(&self) -> bool {
        true
    }
    fn name(&self) -> &'static str {
        "hybrid"
    }
}

/// Comparator + дефолтный порог в нативной шкале метрики.
///
/// Порог интерпретирует `DedupState`: для `higher_is_similar` score > threshold
/// = дубликат. Все метрики здесь similarity-based (выше = похожее).
pub struct MetricSpec {
    pub comparator: Box<dyn FrameComparator>,
    pub default_threshold: f64,
}

/// Фабрика по имени метрики. Регистронезависима, trim.
pub fn metric_by_name(name: &str) -> Result<MetricSpec> {
    match name.trim().to_ascii_lowercase().as_str() {
        "ssim" => Ok(MetricSpec { comparator: Box::new(SsimComparator), default_threshold: 0.98 }),
        "hybrid" => Ok(MetricSpec { comparator: Box::new(HybridComparator), default_threshold: 0.95 }),
        other => Err(TvaError::UnknownMetric(other.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_metrics() {
        for n in ["ssim", "hybrid"] {
            assert!(metric_by_name(n).is_ok(), "{n}");
        }
    }

    #[test]
    fn case_and_trim() {
        assert!(metric_by_name("  SSIM ").is_ok());
        assert!(metric_by_name("Hybrid").is_ok());
    }

    #[test]
    fn unknown_is_err() {
        assert!(matches!(metric_by_name("psnr"), Err(TvaError::UnknownMetric(_))));
    }

    #[test]
    fn similarity_flags_consistent() {
        assert!(metric_by_name("ssim").unwrap().comparator.higher_is_similar());
    }
}
