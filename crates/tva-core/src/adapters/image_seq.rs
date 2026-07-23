use crate::error::{Result, TvaError};
use crate::frame::{Frame, VideoMeta};
use crate::pixel_buffer::PixelBuffer;
use crate::traits::FrameDecoder;
use std::path::{Path, PathBuf};

pub struct ImageSeqDecoder {
    paths: Vec<PathBuf>,
    meta: VideoMeta,
    index: usize,
    fps: f64,
}

impl ImageSeqDecoder {
    pub fn open(dir: &Path, fps: Option<f64>) -> Result<Self> {
        if !dir.is_dir() {
            return Err(TvaError::Decode(format!("not a directory: {}", dir.display())));
        }

        let mut paths: Vec<PathBuf> = std::fs::read_dir(dir)
            .map_err(|e| TvaError::Decode(e.to_string()))?
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .filter(|p| {
                p.extension()
                    .map(|ext| {
                        matches!(
                            ext.to_str().unwrap_or("").to_lowercase().as_str(),
                            "png" | "jpg" | "jpeg" | "tiff" | "tif" | "bmp" | "webp"
                        )
                    })
                    .unwrap_or(false)
            })
            .collect();

        paths.sort();

        if paths.is_empty() {
            return Err(TvaError::Decode(format!("no image files found in {}", dir.display())));
        }

        let fps = fps.unwrap_or(60.0);
        let total = paths.len() as u64;

        let first = image::open(&paths[0]).map_err(|e| TvaError::Decode(format!("{}: {e}", paths[0].display())))?;
        let (w, h) = (first.width(), first.height());

        let meta = VideoMeta {
            fps,
            width: w,
            height: h,
            total_frames: total,
            duration_ms: total as f64 / fps * 1000.0,
            codec: "image-sequence".into(),
        };

        Ok(Self { paths, meta, index: 0, fps })
    }
}

impl FrameDecoder for ImageSeqDecoder {
    fn metadata(&self) -> VideoMeta {
        self.meta.clone()
    }

    fn next_frame(&mut self) -> Option<Frame> {
        let path = self.paths.get(self.index)?;
        let img = image::open(path).ok()?;
        let rgb = img.to_rgb8();
        let buf = PixelBuffer::new(rgb.as_raw().clone(), rgb.width(), rgb.height()).ok()?;
        let idx = self.index as u64;
        let ts = idx as f64 / self.fps * 1000.0;
        self.index += 1;
        Some(Frame { data: buf, index: idx, timestamp_ms: ts })
    }
}
