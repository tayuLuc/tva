use crate::error::{Result, TvaError};
use crate::pixel_buffer::PixelBuffer;
use crate::traits::FrameComparator;
use rgb::RGB8;

pub struct DssimComparator {
    pub threshold: f64,
}

impl FrameComparator for DssimComparator {
    fn compare(&self, a: &PixelBuffer, b: &PixelBuffer) -> Result<f64> {
        let ctx = dssim_core::Dssim::new();
        let w = a.width() as usize;
        let h = a.height() as usize;

        let pixels_a: Vec<RGB8> = a
            .as_bytes()
            .chunks_exact(3)
            .map(|c| RGB8::new(c[0], c[1], c[2]))
            .collect();
        let pixels_b: Vec<RGB8> = b
            .as_bytes()
            .chunks_exact(3)
            .map(|c| RGB8::new(c[0], c[1], c[2]))
            .collect();

        let img_a = ctx
            .create_image_rgb(&pixels_a, w, h)
            .ok_or(TvaError::CompareFailed("dssim create_image_a".into()))?;
        let img_b = ctx
            .create_image_rgb(&pixels_b, w, h)
            .ok_or(TvaError::CompareFailed("dssim create_image_b".into()))?;

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
