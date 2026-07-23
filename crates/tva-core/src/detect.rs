//! Detect duplicate frames and screen tears.

use crate::compare::{compare_frames, CompareMethod, CompareResult};

// ── Duplicate detection ─────────────────────────────────────────────

/// Information about a detected duplicate.
#[derive(Clone, Debug)]
pub struct DuplicateInfo {
    pub frame_index: u64,
    pub streak_start: u64,
    pub streak_length: u32,
}

/// Stateful duplicate detector, отслеживает streak'и.
pub struct DedupState {
    prev_frame: Vec<u8>,
    current_streak: u32,
    unique_frame_index: u64,
    has_prev: bool,
}

impl DedupState {
    pub fn new() -> Self {
        Self {
            prev_frame: Vec::new(),
            current_streak: 0,
            unique_frame_index: 0,
            has_prev: false,
        }
    }

    /// Обработать кадр. Возвращает `Some` если кадр — дубликат.
    pub fn process(
        &mut self,
        frame: &[u8],
        frame_index: u64,
        method: &CompareMethod,
        width: u32,
        height: u32,
    ) -> Option<DuplicateInfo> {
        let frame_size = (width * height * 3) as usize;
        if !self.has_prev {
            self.prev_frame = frame[..frame_size].to_vec();
            self.has_prev = true;
            self.current_streak = 1;
            self.unique_frame_index = 0;
            return None;
        }

        let result = compare_frames(frame, &self.prev_frame, width, height, method);

        if result.is_duplicate {
            self.current_streak += 1;
            Some(DuplicateInfo {
                frame_index,
                streak_start: self.unique_frame_index,
                streak_length: self.current_streak,
            })
        } else {
            self.prev_frame = frame[..frame_size].to_vec();
            self.current_streak = 1;
            self.unique_frame_index = frame_index;
            None
        }
    }
}

impl Default for DedupState {
    fn default() -> Self {
        Self::new()
    }
}

// ── Tear detection ──────────────────────────────────────────────────

/// Information about a detected screen tear.
#[derive(Clone, Debug)]
pub struct TearInfo {
    pub frame_index: u64,
    pub tear_y: u32,
    pub tear_position: f32,
}

/// Детектировать тир между двумя кадрами.
pub fn detect_tear(
    a: &[u8],
    b: &[u8],
    width: u32,
    height: u32,
    threshold_high: f64,
    threshold_low: f64,
) -> Option<TearInfo> {
    let w = width as usize;
    let h = height as usize;
    let row_bytes = w * 3;

    for y in 1..h {
        let off_prev = (y - 1) * row_bytes;
        let off_cur = y * row_bytes;

        let prev_row_diff: f64 = a[off_prev..off_prev + row_bytes]
            .iter()
            .zip(b[off_prev..off_prev + row_bytes].iter())
            .map(|(&x, &y)| x.abs_diff(y) as u64)
            .sum::<u64>() as f64
            / w as f64;

        let cur_row_diff: f64 = a[off_cur..off_cur + row_bytes]
            .iter()
            .zip(b[off_cur..off_cur + row_bytes].iter())
            .map(|(&x, &y)| x.abs_diff(y) as u64)
            .sum::<u64>() as f64
            / w as f64;

        if cur_row_diff > threshold_high && prev_row_diff < threshold_low {
            return Some(TearInfo {
                frame_index: 0,
                tear_y: y as u32,
                tear_position: y as f32 / h as f32,
            });
        }
    }

    None
}
