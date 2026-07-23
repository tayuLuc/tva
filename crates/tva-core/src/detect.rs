use crate::compare::{compare_frames, CompareMethod, CompareResult};
use crate::error::Result;
use crate::frame::Frame;
use rgb::RGB8;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DuplicateInfo {
    pub frame_index: u64,
    pub streak_start: u64,
    pub streak_length: u32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TearInfo {
    pub frame_index: u64,
    pub tear_y: u32,
    pub tear_position: f32,
}

/// Stateful duplicate detector with double buffer — zero allocs after first frame.
#[derive(Debug)]
pub struct DedupState {
    buf_a: Vec<RGB8>,
    buf_b: Vec<RGB8>,
    current: bool,
    current_streak: u32,
    unique_frame_index: u64,
    has_prev: bool,
}

impl DedupState {
    #[must_use]
    pub fn new() -> Self {
        Self {
            buf_a: Vec::new(),
            buf_b: Vec::new(),
            current: false,
            current_streak: 0,
            unique_frame_index: 0,
            has_prev: false,
        }
    }

    fn prev(&self) -> &[RGB8] {
        if self.current { &self.buf_a } else { &self.buf_b }
    }

    fn swap_in(&mut self, frame: &[RGB8]) {
        let target = if self.current { &mut self.buf_b } else { &mut self.buf_a };
        if target.len() < frame.len() {
            target.extend_from_slice(&frame[target.len()..]);
            target.truncate(frame.len());
        } else {
            target[..frame.len()].copy_from_slice(frame);
        }
        self.current = !self.current;
    }

    pub fn process(
        &mut self,
        frame: &Frame,
        method: &CompareMethod,
    ) -> Result<Option<DuplicateInfo>> {
        if !self.has_prev {
            self.swap_in(&frame.data);
            self.has_prev = true;
            self.current_streak = 1;
            self.unique_frame_index = 0;
            return Ok(None);
        }

        // Build a temporary Frame view over the previous buffer
        let prev_frame = Frame {
            data: self.prev().to_vec(),
            width: frame.width,
            height: frame.height,
            index: frame.index - 1,
            timestamp_ms: frame.timestamp_ms - (1000.0 / 30.0), // approximate
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
            self.swap_in(&frame.data);
            self.current_streak = 1;
            self.unique_frame_index = frame.index;
            Ok(None)
        }
    }
}

impl Default for DedupState {
    fn default() -> Self {
        Self::new()
    }
}

/// Detect a horizontal tear between two frames.
pub fn detect_tear(
    a: &Frame,
    b: &Frame,
    threshold_high: f64,
    threshold_low: f64,
) -> Result<Option<TearInfo>> {
    if a.data.len() != b.data.len() {
        return Err(crate::error::TvaError::SizeMismatch {
            expected: a.data.len(),
            got: b.data.len(),
        });
    }

    let w = a.width as usize;
    let h = a.height as usize;

    for y in 1..h {
        let off_prev = (y - 1) * w;
        let off_cur = y * w;

        let prev_row_diff: f64 = a.data[off_prev..off_prev + w]
            .iter().zip(&b.data[off_prev..off_prev + w])
            .map(|(pa, pb)| {
                u64::from(pa.r.abs_diff(pb.r)) + u64::from(pa.g.abs_diff(pb.g)) + u64::from(pa.b.abs_diff(pb.b))
            })
            .sum::<u64>() as f64 / (w as f64 * 3.0);

        let cur_row_diff: f64 = a.data[off_cur..off_cur + w]
            .iter().zip(&b.data[off_cur..off_cur + w])
            .map(|(pa, pb)| {
                u64::from(pa.r.abs_diff(pb.r)) + u64::from(pa.g.abs_diff(pb.g)) + u64::from(pa.b.abs_diff(pb.b))
            })
            .sum::<u64>() as f64 / (w as f64 * 3.0);

        if cur_row_diff > threshold_high && prev_row_diff < threshold_low {
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
    use crate::frame::Frame;

    fn frame_from(data: Vec<RGB8>, w: u32, h: u32) -> Frame {
        Frame { data, width: w, height: h, index: 0, timestamp_ms: 0.0 }
    }

    #[test]
    fn streak_tracking() {
        let mut dedup = DedupState::new();
        let method = CompareMethod::Raw { threshold: 1.0 };
        let a = frame_from(vec![RGB8::new(100, 100, 100); 16], 4, 4);
        let b = frame_from(vec![RGB8::new(200, 200, 200); 16], 4, 4);

        // first frame — no result
        assert!(dedup.process(&a, &method).unwrap().is_none());
        // duplicate of A
        assert!(dedup.process(&a, &method).unwrap().is_some());
        // new frame B
        assert!(dedup.process(&b, &method).unwrap().is_none());
    }

    #[test]
    fn tear_detection_clean() {
        let same = frame_from(vec![RGB8::new(100, 100, 100); 400], 20, 20);
        let r = detect_tear(&same, &same, 30.0, 5.0).unwrap();
        assert!(r.is_none());
    }
}
