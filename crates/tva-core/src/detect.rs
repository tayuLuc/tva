use image::DynamicImage;

use crate::compare::{compare_frames, CompareMethod};
use crate::error::Result;
use crate::frame::Frame;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DuplicateInfo {
    pub frame_index: u64,
    pub streak_start: u64,
    pub streak_length: u32,
}

/// Stateful duplicate detector with double-buffered DynamicImage.
pub struct DedupState {
    buf_a: Option<DynamicImage>,
    buf_b: Option<DynamicImage>,
    prev_is_a: bool,
    current_streak: u32,
    unique_frame_index: u64,
    has_prev: bool,
}

impl DedupState {
    #[must_use]
    pub fn new() -> Self {
        Self {
            buf_a: None,
            buf_b: None,
            prev_is_a: true,
            current_streak: 0,
            unique_frame_index: 0,
            has_prev: false,
        }
    }

    fn prev_frame(&self) -> Option<&DynamicImage> {
        if self.prev_is_a { self.buf_a.as_ref() } else { self.buf_b.as_ref() }
    }

    pub fn process(&mut self, frame: &Frame, method: &CompareMethod) -> Result<Option<DuplicateInfo>> {
        if !self.has_prev {
            self.buf_a = Some(frame.data.clone());
            self.has_prev = true;
            self.current_streak = 1;
            self.unique_frame_index = 0;
            return Ok(None);
        }

        let prev_data = self.prev_frame().expect("has_prev guarantees data");
        let prev_frame = Frame {
            data: prev_data.clone(),
            index: self.unique_frame_index,
            timestamp_ms: 0.0,
        };

        let result = compare_frames(frame, &prev_frame, method)?;

        if result.is_duplicate {
            self.current_streak += 1;
            Ok(Some(DuplicateInfo {
                frame_index: frame.index,
                streak_start: self.unique_frame_index,
                streak_length: self.current_streak,
            }))
        } else {
            let target = if self.prev_is_a { &mut self.buf_b } else { &mut self.buf_a };
            *target = Some(frame.data.clone());
            self.prev_is_a = !self.prev_is_a;
            self.current_streak = 1;
            self.unique_frame_index = frame.index;
            Ok(None)
        }
    }
}

impl Default for DedupState {
    fn default() -> Self { Self::new() }
}

/// Information about a detected screen tear.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TearInfo {
    pub frame_index: u64,
    pub tear_y: u32,
    pub tear_position: f32,
}

/// Detect a horizontal tear between two frames.
/// ponytail: per-row scanline diff; no SIMD yet.
pub fn detect_tear(
    a: &Frame,
    b: &Frame,
    threshold_high: f64,
    threshold_low: f64,
) -> Result<Option<TearInfo>> {
    let a_rgb = a.data.as_rgb8().ok_or(crate::error::TvaError::CompareFailed("not RGB8".into()))?;
    let b_rgb = b.data.as_rgb8().ok_or(crate::error::TvaError::CompareFailed("not RGB8".into()))?;

    let w = a_rgb.width() as usize;
    let h = a_rgb.height() as usize;

    for y in 1..h {
        let prev_off = (y - 1) * w;
        let cur_off = y * w;

        let prev_row: f64 = a_rgb.as_raw()[prev_off * 3..(prev_off + w) * 3]
            .iter().zip(&b_rgb.as_raw()[prev_off * 3..(prev_off + w) * 3])
            .map(|(x, y)| u64::from(x.abs_diff(*y)))
            .sum::<u64>() as f64 / w as f64;

        let cur_row: f64 = a_rgb.as_raw()[cur_off * 3..(cur_off + w) * 3]
            .iter().zip(&b_rgb.as_raw()[cur_off * 3..(cur_off + w) * 3])
            .map(|(x, y)| u64::from(x.abs_diff(*y)))
            .sum::<u64>() as f64 / w as f64;

        if cur_row > threshold_high && prev_row < threshold_low {
            return Ok(Some(TearInfo {
                frame_index: a.index,
                tear_y: y as u32,
                tear_position: y as f32 / h as f32,
            }));
        }
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{DynamicImage, Rgb, RgbImage};
    use crate::frame::Frame;

    fn solid(r: u8, g: u8, b: u8, w: u32, h: u32) -> Frame {
        Frame { data: DynamicImage::ImageRgb8(RgbImage::from_pixel(w, h, Rgb([r, g, b]))), index: 0, timestamp_ms: 0.0 }
    }

    #[test]
    fn streak_tracking() {
        let mut d = DedupState::new();
        let m = CompareMethod::Ssim { threshold: 0.98 };
        let a = solid(100, 100, 100, 4, 4);
        let b = solid(200, 200, 200, 4, 4);

        assert!(d.process(&a, &m).unwrap().is_none());
        assert!(d.process(&solid(100, 100, 100, 4, 4), &m).unwrap().is_some());
        assert!(d.process(&b, &m).unwrap().is_none());
    }
}
