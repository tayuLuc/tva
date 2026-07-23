use crate::error::Result;
use crate::traits::Smoother;

#[derive(Debug, Clone, Copy, Default)]
pub struct IdentitySmoother;

impl Smoother for IdentitySmoother {
    fn smooth(&self, data: &[f64]) -> Result<Vec<f64>> {
        Ok(data.to_vec())
    }
}
