use crate::error::Result;
use crate::frame::{Frame, VideoMeta};
use std::path::Path;

pub fn decode_file(_path: &Path) -> Result<(Vec<Frame>, VideoMeta)> {
    Err(crate::error::TvaError::AdapterNotEnabled("decode"))
}
