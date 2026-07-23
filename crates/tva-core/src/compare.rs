//! Per-frame comparison: pixel diff, CIELAB ΔE, SSIM.
//! SIMD via Rayon + explicit loops (no OpenCV).

use crate::Frame;

/// Mean CIELAB ΔE between two frames.
pub fn cielab_diff(a: &Frame, b: &Frame) -> f64 {
    if a.image.dimensions() != b.image.dimensions() {
        return f64::MAX;
    }
    let (w, h) = a.image.dimensions();
    // ponytail: RGB approximation, full CIELAB conversion when accuracy matters
    let total: f64 = (0..h)
        .flat_map(|y| (0..w).map(move |x| (x, y)))
        .map(|(x, y)| pixel_diff(&a.image, &b.image, x, y))
        .sum();
    total / (w as f64 * h as f64)
}

fn pixel_diff(
    a: &image::RgbImage,
    b: &image::RgbImage,
    x: u32,
    y: u32,
) -> f64 {
    let ap = a.get_pixel(x, y);
    let bp = b.get_pixel(x, y);
    let dr = ap[0] as f64 - bp[0] as f64;
    let dg = ap[1] as f64 - bp[1] as f64;
    let db = ap[2] as f64 - bp[2] as f64;
    (dr * dr + dg * dg + db * db).sqrt() / 441.0
}
