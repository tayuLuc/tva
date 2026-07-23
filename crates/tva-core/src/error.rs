use thiserror::Error;

#[derive(Debug, Error)]
pub enum TvaError {
    #[error("frame size mismatch: expected {expected}, got {got}")]
    SizeMismatch { expected: usize, got: usize },

    #[error("empty frame data")]
    EmptyFrame,

    #[error("comparison failed: {0}")]
    CompareFailed(String),

    #[error("smoothing failed: {0}")]
    SmoothingFailed(String),

    #[error("fft failed: {0}")]
    FftFailed(String),

    #[error("decode error: {0}")]
    Decode(String),

    #[error("adapter not enabled: enable feature `{0}`")]
    AdapterNotEnabled(&'static str),

    #[error("serialization error: {0}")]
    Serialization(String),

    #[error("csv error: {0}")]
    Csv(String),

    #[error("unknown metric `{0}` (expected ssim, hybrid, mad)")]
    UnknownMetric(String),
}

impl From<serde_json::Error> for TvaError {
    fn from(e: serde_json::Error) -> Self {
        TvaError::Serialization(e.to_string())
    }
}

impl From<csv::Error> for TvaError {
    fn from(e: csv::Error) -> Self {
        TvaError::Csv(e.to_string())
    }
}

impl From<std::io::Error> for TvaError {
    fn from(e: std::io::Error) -> Self {
        TvaError::Decode(e.to_string())
    }
}

pub type Result<T> = std::result::Result<T, TvaError>;
