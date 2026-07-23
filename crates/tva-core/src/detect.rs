//! Detect duplicate frames and screen tears.

use crate::Frame;

/// A frame is a duplicate if mean pixel diff from previous is below threshold.
pub fn is_duplicate(a: &Frame, b: &Frame) -> bool {
    // ponytail: simple diff threshold; add SSIM if false positives appear
    crate::compare::cielab_diff(a, b) < 0.02
}

/// Detect a screen-tear line (horizontal discontinuity between scanlines).
pub fn find_tear_line(frame: &Frame) -> Option<u32> {
    let (w, h) = frame.image.dimensions();
    // ponytail: naive scanline diff; adaptive threshold when needed
    for y in 1..h {
        let diff: u32 = (0..w)
            .map(|x| {
                let p = frame.image.get_pixel(x, y);
                let q = frame.image.get_pixel(x, y - 1);
                let dr = p[0] as i32 - q[0] as i32;
                let dg = p[1] as i32 - q[1] as i32;
                let db = p[2] as i32 - q[2] as i32;
                (dr * dr + dg * dg + db * db) as u32
            })
            .sum::<u32>()
            / w;
        if diff > 5000 {
            return Some(y);
        }
    }
    None
}
