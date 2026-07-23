#![forbid(unsafe_code)]

pub mod pixel_buffer;
pub mod frame;
pub mod error;
pub mod traits;
pub mod adapters;
pub mod config;
pub mod events;
pub mod source;
pub mod detect;
pub mod metrics;
pub mod resolution;
pub mod report;
pub mod pipeline;
pub mod export;
pub mod overlay;
pub mod decoder;

pub use pixel_buffer::PixelBuffer;
pub use frame::{Frame, VideoMeta};
pub use error::{Result, TvaError};
pub use traits::{FrameComparator, Smoother};
pub use detect::TearInfo;
pub use metrics::{FrameMetric, SummaryMetrics};
pub use report::Report;
pub use resolution::ResolutionResult;
