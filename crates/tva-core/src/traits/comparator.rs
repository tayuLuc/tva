use crate::error::Result;
use crate::pixel_buffer::PixelBuffer;

/// Trait для сравнения двух кадров.
/// Алгоритмы вечны — этот трейт не меняется.
pub trait FrameComparator: Send + Sync {
    /// Сравнить два кадра. Возвращает score.
    /// Дубликат определяется через `higher_is_similar` + threshold.
    fn compare(&self, a: &PixelBuffer, b: &PixelBuffer) -> Result<f64>;

    /// Если true: больше score = более похожи (SSIM).
    /// Если false: меньше score = более похожи (MAD/SAD).
    fn higher_is_similar(&self) -> bool;

    /// Человеческое имя метрики (ssim, mad, hybrid…).
    fn name(&self) -> &'static str;
}
