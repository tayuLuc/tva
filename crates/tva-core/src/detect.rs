use crate::compare::{compare_frames, CompareMethod};
use crate::error::Result;
use rgb::RGB8;

#[derive(Debug, Clone)]
pub struct DuplicateInfo {
    pub frame_index: u64,
    pub streak_start: u64,
    pub streak_length: u32,
}

#[derive(Debug, Clone)]
pub struct TearInfo {
    pub frame_index: u64,
    pub tear_y: u32,
    pub tear_position: f32,
}

/// Stateful duplicate detector. Feed frames one at a time.
#[derive(Debug)]
pub struct DedupState {
    prev_frame: Vec<RGB8>,
    current_streak: u32,
    unique_frame_index: u64,
    has_prev: bool,
}

impl DedupState {
    #[must_use]
    pub fn new() -> Self {
        Self {
            prev_frame: Vec::new(),
            current_streak: 0,
            unique_frame_index: 0,
            has_prev: false,
        }
    }

    pub fn process(
        &mut self,
        frame: &[RGB8],
        frame_index: u64,
        width: usize,
        height: usize,
        method: &CompareMethod,
    ) -> Result<Option<DuplicateInfo>> {
        if !self.has_prev {
            self.prev_frame = frame.to_vec();
            self.has_prev = true;
            self.current_streak = 1;
            self.unique_frame_index = 0;
            return Ok(None);
        }

        let result = compare_frames(frame, &self.prev_frame, width, height, method)?;

        if result.is_duplicate {
            self.current_streak += 1;
            Ok(Some(DuplicateInfo {
                frame_index,
                streak_start: self.unique_frame_index,
                streak_length: self.current_streak,
            }))
        } else {
            self.prev_frame = frame.to_vec();
            self.current_streak = 1;
            self.unique_frame_index = frame_index;
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
#[must_use]
pub fn detect_tear(
    a: &[RGB8],
    b: &[RGB8],
    width: usize,
    height: usize,
    threshold_high: f64,
    threshold_low: f64,
) -> Option<TearInfo> {
    if a.len() != width * height || b.len() != width * height {
        return None;
    }

    let row_diffs: Vec<f64> = (0..height)
        .map(|y| {
            let row = &a[y * width..(y + 1) * width];
            let row_b = &b[y * width..(y + 1) * width];
            let sum: u64 = row
                .iter()
                .zip(row_b)
                .map(|(pa, pb)| {
                    u64::from(pa.r.abs_diff(pb.r))
                        + u64::from(pa.g.abs_diff(pb.g))
                        + u64::from(pa.b.abs_diff(pb.b))
                })
                .sum();
            sum as f64 / (width as f64 * 3.0)
        })
        .collect();

    for y in 1..height {
        if row_diffs[y] > threshold_high && row_diffs[y - 1] < threshold_low {
            return Some(TearInfo {
                frame_index: 0,
                tear_y: y as u32,
                tear_position: y as f32 / height as f32,
            });
        }
    }
    None
}
