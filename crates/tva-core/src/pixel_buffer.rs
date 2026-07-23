#[derive(Debug, Clone)]
pub struct PixelBuffer {
    data: Vec<u8>,
    width: u32,
    height: u32,
}

impl PixelBuffer {
    pub fn new(data: Vec<u8>, width: u32, height: u32) -> Result<Self, crate::error::TvaError> {
        let exp = (width * height * 3) as usize;
        if data.len() != exp {
            return Err(crate::error::TvaError::SizeMismatch { expected: exp, got: data.len() });
        }
        Ok(Self { data, width, height })
    }

    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }
    #[must_use]
    pub fn into_bytes(self) -> Vec<u8> {
        self.data
    }
    #[must_use]
    pub fn width(&self) -> u32 {
        self.width
    }
    #[must_use]
    pub fn height(&self) -> u32 {
        self.height
    }
    #[must_use]
    pub fn pixel_count(&self) -> usize {
        (self.width * self.height) as usize
    }
    #[must_use]
    pub fn row_bytes(&self) -> usize {
        self.width as usize * 3
    }

    pub fn split_rgb(&self) -> (Vec<u8>, Vec<u8>, Vec<u8>) {
        let n = self.pixel_count();
        let (mut r, mut g, mut b) = (Vec::with_capacity(n), Vec::with_capacity(n), Vec::with_capacity(n));
        for ch in self.data.chunks_exact(3) {
            r.push(ch[0]);
            g.push(ch[1]);
            b.push(ch[2]);
        }
        (r, g, b)
    }
}

#[cfg(any(feature = "compare-image", feature = "compare-dssim"))]
impl PixelBuffer {
    #[must_use]
    pub fn to_dynamic_image(&self) -> image::DynamicImage {
        image::DynamicImage::ImageRgb8(
            image::RgbImage::from_raw(self.width, self.height, self.data.clone())
                .expect("PixelBuffer validated dimensions"),
        )
    }

    #[must_use]
    pub fn from_dynamic_image(img: &image::DynamicImage) -> Self {
        let rgb = img.to_rgb8();
        Self { data: rgb.as_raw().clone(), width: rgb.width(), height: rgb.height() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn valid() {
        PixelBuffer::new(vec![0u8; 36], 12, 1).unwrap();
    }
    #[test]
    fn bad_size() {
        assert!(PixelBuffer::new(vec![0u8; 10], 12, 3).is_err());
    }
    #[test]
    fn split() {
        let b = PixelBuffer::new(vec![0, 1, 2, 3, 4, 5], 2, 1).unwrap();
        let (r, g, _) = b.split_rgb();
        assert_eq!(r, vec![0, 3]);
        assert_eq!(g, vec![1, 4]);
    }
}
