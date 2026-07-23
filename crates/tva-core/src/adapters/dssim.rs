//! Адаптер `dssim-core` (multi-core DSSIM). Feature `compare-dssim`.

use crate::error::{Result, TvaError};
use crate::pixel_buffer::PixelBuffer;
use crate::traits::FrameComparator;

pub struct DssimComparator {
    pub threshold: f64,
}

impl FrameComparator for DssimComparator {
    fn compare(&self, a: &PixelBuffer, b: &PixelBuffer) -> Result<f64> {
        let ctx = dssim_core::Dssim::new();
        let w = a.width() as usize;
        let h = a.height() as usize;
        let img_a =
            ctx.create_image_rgb(a.as_bytes(), w, h).ok_or(TvaError::CompareFailed("dssim create_image_a".into()))?;
        let img_b =
            ctx.create_image_rgb(b.as_bytes(), w, h).ok_or(TvaError::CompareFailed("dssim create_image_b".into()))?;
        // dssim-core принимает &[u8] (RGB) напрямую
        // create_image_rgb(&[u8], width, height) -> Option<DssimImage>
        let (val, _) = ctx.compare(&img_a, &img_b);
        Ok(val.into())
    }
    fn higher_is_similar(&self) -> bool {
        true
    }
    fn name(&self) -> &'static str {
        "dssim"
    }
}
