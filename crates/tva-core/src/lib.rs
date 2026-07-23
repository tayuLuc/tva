pub mod compare;
pub mod detect;
pub mod error;
pub mod metrics;
pub mod resolution;
pub mod smooth;

pub mod decoder;
pub mod export;
pub mod overlay;
pub mod pipeline;
pub mod ffi;

pub use compare::{CompareMethod, CompareResult};
pub use detect::{DedupState, DuplicateInfo, TearInfo};
pub use error::{Result, TvaError};
pub use metrics::{FrameMetric, SummaryMetrics};
pub use resolution::ResolutionResult;
