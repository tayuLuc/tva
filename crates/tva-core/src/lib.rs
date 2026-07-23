#![forbid(unsafe_code)]

pub mod adapters;
pub mod config;
pub mod decoder;
pub mod detect;
pub mod error;
pub mod events;
pub mod export;
pub mod frame;
pub mod metrics;
pub mod overlay;
pub mod pipeline;
pub mod pixel_buffer;
pub mod report;
pub mod resolution;
pub mod traits;

pub use detect::TearInfo;
pub use error::{Result, TvaError};
pub use frame::{Frame, VideoMeta};
pub use metrics::{FrameMetric, SummaryMetrics};
pub use pixel_buffer::PixelBuffer;
pub use report::Report;
pub use resolution::ResolutionResult;
pub use traits::{FrameComparator, FrameDecoder, Smoother};

pub use adapters::identity_smoother::IdentitySmoother;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
