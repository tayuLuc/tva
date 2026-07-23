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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TearInfo {
    pub frame_index: u64,
    pub tear_y: u32,
    pub tear_position: f32,
    #[serde(default)]
    pub old_fraction: f32,
    #[serde(default)]
    pub new_fraction: f32,
}

pub struct Peek {
    pub first: bool,
    pub score: f64,
    pub dup_by_pixels: bool,
}

pub struct DedupState {
    prev: Option<Vec<u8>>,
    prev_size: (u32, u32),
    pub(crate) current_streak: u32,
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

    pub fn prev_bytes(&self) -> Option<&[u8]> {
        self.prev.as_deref()
    }

    pub fn peek(&self, frame: &Frame, comparator: &dyn FrameComparator) -> Result<Peek> {
        if !self.has_prev {
            return Ok(Peek { first: true, score: 0.0, dup_by_pixels: false });
        }
        let prev_buf = PixelBuffer::new(self.prev.clone().unwrap(), self.prev_size.0, self.prev_size.1)?;
        let score = comparator.compare(&frame.data, &prev_buf)?;
        let dup = if self.higher_is_similar { score > self.threshold } else { score < self.threshold };
        Ok(Peek { first: false, score, dup_by_pixels: dup })
    }

    pub fn commit(&mut self, frame: &Frame, is_unique: bool) -> Option<DuplicateInfo> {
        if !self.has_prev {
            self.prev = Some(frame.data.as_bytes().to_vec());
            self.prev_size = (frame.data.width(), frame.data.height());
            self.has_prev = true;
            self.current_streak = 1;
            self.unique_frame_index = 0;
            return None;
        }
        if is_unique {
            self.prev = Some(frame.data.as_bytes().to_vec());
            self.prev_size = (frame.data.width(), frame.data.height());
            self.current_streak = 1;
            self.unique_frame_index = frame.index;
            None
        } else {
            self.current_streak += 1;
            Some(DuplicateInfo {
                frame_index: frame.index,
                streak_start: self.unique_frame_index,
                streak_length: self.current_streak,
            })
        }
    }

    pub fn process(&mut self, frame: &Frame, comparator: &dyn FrameComparator) -> Result<Option<DuplicateInfo>> {
        let p = self.peek(frame, comparator)?;
        if p.first {
            return Ok(self.commit(frame, true));
        }
        Ok(self.commit(frame, !p.dup_by_pixels))
    }
}

pub fn detect_tear(a: &Frame, b: &Frame, th_high: f64, th_low: f64) -> Result<Option<TearInfo>> {
    let ab = a.data.as_bytes();
    let bb = b.data.as_bytes();
    let w = a.data.width() as usize;
    let h = a.data.height() as usize;
    if ab.len() != bb.len() || w == 0 || h < 2 {
        return Ok(None);
    }
    let row = w * 3;

    let row_diffs: Vec<f64> = (0..h)
        .map(|y| {
            let o = y * row;
            ab[o..o + row].iter().zip(&bb[o..o + row]).map(|(&x, &y)| u64::from(x.abs_diff(y))).sum::<u64>() as f64
                / row as f64
        })
        .collect();

    let mut pref = vec![0.0_f64; h + 1];
    for i in 0..h {
        pref[i + 1] = pref[i] + row_diffs[i];
    }
    let avg = |l: usize, r: usize| -> f64 {
        if r <= l {
            0.0
        } else {
            (pref[r] - pref[l]) / (r - l) as f64
        }
    };

    let mut tear_y = 0_usize;
    let mut best_grad = -1.0_f64;
    for y in 1..h {
        let g = (row_diffs[y] - row_diffs[y - 1]).abs();
        if g > best_grad {
            best_grad = g;
            tear_y = y;
        }
    }
    if tear_y == 0 || tear_y >= h {
        return Ok(None);
    }

    let left = avg(0, tear_y);
    let right = avg(tear_y, h);
    if left.min(right) >= th_low || left.max(right) <= th_high {
        return Ok(None);
    }

    let old_top = left < right;
    let old_fraction = if old_top { tear_y as f32 / h as f32 } else { (h - tear_y) as f32 / h as f32 };

    Ok(Some(TearInfo {
        frame_index: a.index,
        tear_y: tear_y as u32,
        tear_position: tear_y as f32 / h as f32,
        old_fraction,
        new_fraction: 1.0 - old_fraction,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
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
    fn streak_back_compat() {
        let mut d = DedupState::new(0.5, true);
        assert!(d.process(&f(vec![100; 12], 2, 2, 0), &Dummy).unwrap().is_none());
        assert!(d.process(&f(vec![100; 12], 2, 2, 1), &Dummy).unwrap().is_some());
        assert!(d.process(&f(vec![200; 12], 2, 2, 2), &Dummy).unwrap().is_none());
    }

    #[test]
    fn peek_commit_suppress() {
        let mut d = DedupState::new(0.5, true);
        let a = f(vec![100; 12], 2, 2, 0);
        let b = f(vec![200; 12], 2, 2, 1);
        assert!(d.peek(&a, &Dummy).unwrap().first);
        d.commit(&a, true);
        let p = d.peek(&b, &Dummy).unwrap();
        assert!(!p.dup_by_pixels);
        d.commit(&b, false);
        assert_eq!(d.current_streak, 2);
    }

    #[test]
    fn tear_half_old_half_new() {
        let mut cur = vec![10u8; 48];
        for y in 2..4 {
            for x in 0..12 {
                cur[y * 12 + x] = 200;
            }
        }
        let a = f(vec![10u8; 48], 4, 4, 0);
        let b = f(cur, 4, 4, 1);
        let t = detect_tear(&a, &b, 30.0, 5.0).unwrap().unwrap();
        assert_eq!(t.tear_y, 2);
        assert!((t.old_fraction - 0.5).abs() < 1e-3);
    }

    #[test]
    fn tear_mostly_new() {
        let mut cur = vec![200u8; 48];
        for x in 0..12 {
            cur[x] = 10;
        }
        let a = f(vec![10u8; 48], 4, 4, 0);
        let b = f(cur, 4, 4, 1);
        let t = detect_tear(&a, &b, 30.0, 5.0).unwrap().unwrap();
        assert!((t.new_fraction - 0.75).abs() < 1e-3);
    }

    #[test]
    fn scene_change_is_not_tear() {
        assert!(detect_tear(&f(vec![10u8; 48], 4, 4, 0), &f(vec![200u8; 48], 4, 4, 1), 30.0, 5.0).unwrap().is_none());
    }

    #[test]
    fn identical_is_not_tear() {
        let v = vec![100u8; 48];
        assert!(detect_tear(&f(v.clone(), 4, 4, 0), &f(v, 4, 4, 1), 30.0, 5.0).unwrap().is_none());
    }
}
