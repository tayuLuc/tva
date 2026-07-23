//! Render FPS/frametime graph overlay onto video.

use crate::error::Result;
use crate::report::Report;
use std::path::Path;

/// Render overlay video with FPS graph.
/// ponytail: stub — ffmpeg filter chain when implemented.
pub fn overlay_video(_path: &Path, _report: &Report, _out: &Path) -> Result<()> {
    Ok(())
}
