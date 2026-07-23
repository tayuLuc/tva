use crate::error::{Result, TvaError};
use crate::traits::Smoother;
use staged_sg_filter::sav_gol;

pub struct SavgolSmoother {
    pub window: usize,
    pub polyorder: usize,
}

impl Smoother for SavgolSmoother {
    fn smooth(&self, data: &[f64]) -> Result<Vec<f64>> {
        if data.is_empty() {
            return Ok(Vec::new());
        }
        if self.window <= self.polyorder + 1 {
            return Err(TvaError::SmoothingFailed(
                format!("window {} must be > polyorder+1 ({})", self.window, self.polyorder + 1),
            ));
        }
        if self.window > data.len() {
            return Err(TvaError::SmoothingFailed(
                format!("window {} exceeds data length {}", self.window, data.len()),
            ));
        }
        let mut buf = data.to_vec();
        sav_gol(&mut buf, self.window, self.polyorder)
            .map_err(|e| TvaError::SmoothingFailed(e.to_string()))?;
        Ok(buf)
    }
}
