//! Comparator'ы без внешних крейтов — чистая арифметика над RGB-байтами.

use crate::error::Result;
use crate::pixel_buffer::PixelBuffer;
use crate::traits::FrameComparator;

/// Mean absolute difference per byte. Шкала [0, 255], не зависит от размера кадра.
/// `higher_is_similar = false`: меньше = похожее.
#[derive(Debug, Clone, Copy, Default)]
pub struct MadComparator;

impl FrameComparator for MadComparator {
    fn compare(&self, a: &PixelBuffer, b: &PixelBuffer) -> Result<f64> {
        let a = a.as_bytes();
        let b = b.as_bytes();
        debug_assert_eq!(a.len(), b.len());
        if a.is_empty() {
            return Ok(0.0);
        }
        let sum: u64 = a.iter().zip(b).map(|(&x, &y)| u64::from(x.abs_diff(y))).sum();
        Ok(sum as f64 / a.len() as f64)
    }
    fn higher_is_similar(&self) -> bool {
        false
    }
    fn name(&self) -> &'static str {
        "mad"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn buf(v: u8, n: u32) -> PixelBuffer {
        PixelBuffer::new(vec![v; (n * 3) as usize], n, 1).unwrap()
    }

    #[test]
    fn identical_is_zero() {
        let a = buf(120, 4);
        assert!(MadComparator.compare(&a, &a).unwrap().abs() < f64::EPSILON);
    }
    #[test]
    fn max_is_255() {
        let a = buf(0, 4);
        let b = buf(255, 4);
        assert!((MadComparator.compare(&a, &b).unwrap() - 255.0).abs() < f64::EPSILON);
    }
    #[test]
    fn size_independent() {
        let a2 = buf(100, 2);
        let b2 = buf(105, 2);
        let a4 = buf(100, 4);
        let b4 = buf(105, 4);
        assert!((MadComparator.compare(&a2, &b2).unwrap() - 5.0).abs() < f64::EPSILON);
        assert!((MadComparator.compare(&a4, &b4).unwrap() - 5.0).abs() < f64::EPSILON);
    }
    #[test]
    fn not_higher_is_similar() {
        assert!(!MadComparator.higher_is_similar());
        assert_eq!(MadComparator.name(), "mad");
    }
}
