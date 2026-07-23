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
}

pub type Result<T> = std::result::Result<T, TvaError>;
