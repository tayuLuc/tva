//! Native decode via ffmpeg-next (libav statically linked, no ffmpeg in PATH).
//! Feature `decode-ffmpeg-native`.
//!
//! Flow: format::input -> best video stream -> decoder -> read/decode loop ->
//! sws scale+convert to RGB24 -> PixelBuffer. PTS is honest:
//! packet.pts() * stream time_base -> ms (not CFR approximation).

use std::path::Path;
use std::sync::Once;

use ffmpeg_next::format::{input, Pixel};
use ffmpeg_next::media::Type;
use ffmpeg_next::software::scaling::{context::Context as ScaleCtx, flag::Flags};
use ffmpeg_next::util::frame::video::Video as VideoFrame;
use ffmpeg_next::Rational;

use crate::error::{Result, TvaError};
use crate::frame::{Frame, VideoMeta};
use crate::pixel_buffer::PixelBuffer;
use crate::traits::FrameDecoder;

/// ffmpeg-next requires one-time library initialisation.
static INIT: Once = Once::new();
fn init_once() {
    INIT.call_once(|| {
        ffmpeg_next::init().expect("ffmpeg_next::init failed");
    });
}

pub struct FfmpegNativeDecoder {
    meta: VideoMeta,
    input: ffmpeg_next::format::context::Input,
    decoder: ffmpeg_next::decoder::Video,
    scaler: ScaleCtx,
    rgb_frame: VideoFrame,
    video_stream_index: usize,
    width: u32,
    height: u32,
    index: u64,
}

impl FfmpegNativeDecoder {
    pub fn open(path: &Path) -> Result<Self> {
        init_once();
        let mut input = input(path).map_err(|e| TvaError::Decode(format!("open {}: {e}", path.display())))?;

        let stream = input.streams().best(Type::Video).ok_or_else(|| TvaError::Decode("no video stream".into()))?;

        let video_stream_index = stream.index();
        let tb = stream.time_base();

        // Metadata from decoder context
        let ctx = ffmpeg_next::codec::context::Context::from_parameters(stream.parameters())
            .map_err(|e| TvaError::Decode(format!("codec context: {e}")))?;
        let mut decoder = ctx.decoder().video().map_err(|e| TvaError::Decode(format!("open video decoder: {e}")))?;
        let width = decoder.width();
        let height = decoder.height();
        let src_format = decoder.format();
        let fps = rational_to_f64(stream.avg_frame_rate());
        let duration_ms = if stream.duration() > 0 {
            stream.duration() as f64 * tb.numerator() as f64 / tb.denominator() as f64 * 1000.0
        } else {
            0.0
        };
        let total_frames = if fps > 0.0 && duration_ms > 0.0 { (duration_ms / 1000.0 * fps).round() as u64 } else { 0 };

        let scaler = ScaleCtx::get(src_format, width, height, Pixel::RGB24, width, height, Flags::BILINEAR)
            .map_err(|e| TvaError::Decode(format!("init scaler: {e}")))?;

        let rgb_frame = VideoFrame::new(Pixel::RGB24, width, height);

        Ok(Self {
            meta: VideoMeta { fps, width, height, total_frames, duration_ms, codec: "ffmpeg-native".into() },
            input,
            decoder,
            scaler,
            rgb_frame,
            video_stream_index,
            width,
            height,
            index: 0,
        })
    }

    fn decode_next(&mut self) -> Option<(Vec<u8>, f64)> {
        loop {
            match self.input.packets().next() {
                Some(Ok((stream, packet))) => {
                    if stream.index() != self.video_stream_index {
                        continue;
                    }
                    let pts_ms = packet_pts_ms(packet.pts(), stream.time_base());
                    if self.decoder.send_packet(&packet).is_err() {
                        continue;
                    }
                    let mut frame = VideoFrame::empty();
                    if self.decoder.receive_frame(&mut frame).is_ok() {
                        return self.scale_to_rgb(frame, pts_ms);
                    }
                }
                Some(Err(_)) => continue,
                None => {
                    let _ = self.decoder.send_eof();
                    let mut frame = VideoFrame::empty();
                    if self.decoder.receive_frame(&mut frame).is_ok() {
                        return self.scale_to_rgb(frame, self.index as f64 / self.meta.fps.max(1.0) * 1000.0);
                    }
                    return None;
                }
            }
        }
    }

    fn scale_to_rgb(&mut self, frame: VideoFrame, pts_ms: f64) -> Option<(Vec<u8>, f64)> {
        self.scaler.run(&frame, &mut self.rgb_frame).ok()?;
        let stride = self.rgb_frame.stride(0);
        let w = self.width as usize;
        let row_bytes = w * 3;
        let mut rgb = Vec::with_capacity(row_bytes * self.height as usize);
        for y in 0..self.height as usize {
            let start = y * stride;
            let row = &self.rgb_frame.data(0)[start..start + row_bytes];
            rgb.extend_from_slice(row);
        }
        Some((rgb, pts_ms))
    }
}

impl FrameDecoder for FfmpegNativeDecoder {
    fn metadata(&self) -> VideoMeta {
        self.meta.clone()
    }

    fn next_frame(&mut self) -> Option<Frame> {
        let (rgb_bytes, pts_ms) = self.decode_next()?;
        let data = PixelBuffer::new(rgb_bytes, self.width, self.height).ok()?;
        let idx = self.index;
        self.index += 1;
        Some(Frame { data, index: idx, timestamp_ms: pts_ms })
    }
}

impl Drop for FfmpegNativeDecoder {
    fn drop(&mut self) {}
}

fn rational_to_f64(r: Rational) -> f64 {
    if r.denominator() == 0 {
        return 0.0;
    }
    r.numerator() as f64 / r.denominator() as f64
}

fn packet_pts_ms(pts: Option<i64>, tb: Rational) -> f64 {
    match pts {
        Some(p) if p >= 0 => p as f64 * tb.numerator() as f64 / tb.denominator() as f64 * 1000.0,
        _ => 0.0,
    }
}
