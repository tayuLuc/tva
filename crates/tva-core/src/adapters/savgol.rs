use crate::error::{Result, TvaError};
use crate::traits::Smoother;
use savgol_rs::{savgol_filter, SavGolInput};

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
            return Err(TvaError::SmoothingFailed(format!(
                "window {} must be > polyorder+1 ({})",
                self.window,
                self.polyorder + 1
            )));
        }
        if self.window > data.len() {
            return Err(TvaError::SmoothingFailed(format!(
                "window {} exceeds data length {}",
                self.window,
                data.len()
            )));
        }
        let w = if self.window % 2 == 0 { self.window + 1 } else { self.window };
        let input = SavGolInput { data, window_length: w, poly_order: self.polyorder, derivative: 0 };
        savgol_filter(&input).map_err(TvaError::SmoothingFailed)
    }
}
