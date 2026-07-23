use crate::error::Result;
use crate::frame::Frame;
use crate::pixel_buffer::PixelBuffer;
use crate::traits::FrameComparator;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DuplicateInfo {
    pub frame_index: u64,
    pub streak_start: u64,
    pub streak_length: u32,
}

pub struct DedupState {
    prev: Option<Vec<u8>>,
    prev_size: (u32, u32),
    current_streak: u32,
    unique_frame_index: u64,
    has_prev: bool,
    threshold: f64,
    higher_is_similar: bool,
}

impl DedupState {
    pub fn new(threshold: f64, higher_is_similar: bool) -> Self {
        Self {
            prev: None,
            prev_size: (0, 0),
            current_streak: 0,
            unique_frame_index: 0,
            has_prev: false,
            threshold,
            higher_is_similar,
        }
    }

    pub fn process(&mut self, frame: &Frame, comparator: &dyn FrameComparator) -> Result<Option<DuplicateInfo>> {
        if !self.has_prev {
            self.prev = Some(frame.data.as_bytes().to_vec());
            self.prev_size = (frame.data.width(), frame.data.height());
            self.has_prev = true;
            self.current_streak = 1;
            self.unique_frame_index = 0;
            return Ok(None);
        }

        let prev_data = self.prev.as_ref().expect("has_prev guarantees data");
        let prev_buf = PixelBuffer::new(prev_data.clone(), self.prev_size.0, self.prev_size.1)?;
        let score = comparator.compare(&frame.data, &prev_buf)?;

        if (self.higher_is_similar && score > self.threshold) || (!self.higher_is_similar && score < self.threshold) {
            self.prev = Some(prev_buf.into_bytes());
            self.current_streak += 1;
            Ok(Some(DuplicateInfo {
                frame_index: frame.index,
                streak_start: self.unique_frame_index,
                streak_length: self.current_streak,
            }))
        } else {
            self.prev = Some(frame.data.as_bytes().to_vec());
            self.prev_size = (frame.data.width(), frame.data.height());
            self.current_streak = 1;
            self.unique_frame_index = frame.index;
            Ok(None)
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TearInfo {
    pub frame_index: u64,
    pub tear_y: u32,
    pub tear_position: f32,
}

pub fn detect_tear(a: &Frame, b: &Frame, th_high: f64, th_low: f64) -> Result<Option<TearInfo>> {
    let ab = a.data.as_bytes();
    let bb = b.data.as_bytes();
    let w = a.data.width() as usize;
    let h = a.data.height() as usize;
    let row = w * 3;
    for y in 1..h {
        let po = (y - 1) * row;
        let co = y * row;
        let pd: f64 =
            ab[po..po + row].iter().zip(&bb[po..po + row]).map(|(&x, &y)| u64::from(x.abs_diff(y))).sum::<u64>() as f64
                / w as f64;
        let cd: f64 =
            ab[co..co + row].iter().zip(&bb[co..co + row]).map(|(&x, &y)| u64::from(x.abs_diff(y))).sum::<u64>() as f64
                / w as f64;
        if cd > th_high && pd < th_low {
            return Ok(Some(TearInfo { frame_index: a.index, tear_y: y as u32, tear_position: y as f32 / h as f32 }));
        }
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frame::Frame;
    use crate::pixel_buffer::PixelBuffer;
    struct Dummy;
    impl FrameComparator for Dummy {
        fn compare(&self, a: &PixelBuffer, b: &PixelBuffer) -> Result<f64> {
            Ok(if a.as_bytes() == b.as_bytes() { 1.0 } else { 0.0 })
        }
        fn higher_is_similar(&self) -> bool {
            true
        }
        fn name(&self) -> &'static str {
            "dummy"
        }
    }
    fn f(d: Vec<u8>, w: u32, h: u32, i: u64) -> Frame {
        Frame { data: PixelBuffer::new(d, w, h).unwrap(), index: i, timestamp_ms: 0.0 }
    }
    #[test]
    fn streak() {
        let mut d = DedupState::new(0.5, true);
        assert!(d.process(&f(vec![100; 12], 2, 2, 0), &Dummy).unwrap().is_none());
        assert!(d.process(&f(vec![100; 12], 2, 2, 1), &Dummy).unwrap().is_some());
        assert!(d.process(&f(vec![200; 12], 2, 2, 2), &Dummy).unwrap().is_none());
    }
}
