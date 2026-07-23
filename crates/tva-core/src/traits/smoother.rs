use crate::error::Result;

/// Trait для сглаживания временнуго ряда (FPS).
pub trait Smoother: Send + Sync {
    fn smooth(&self, data: &[f64]) -> Result<Vec<f64>>;
}
