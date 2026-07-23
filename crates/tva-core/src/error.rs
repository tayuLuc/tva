use thiserror::Error;

#[derive(Debug, Error)]
pub enum TvaError {
    #[error("frame size mismatch: expected {expected}, got {got}")]
    SizeMismatch { expected: usize, got: usize },

    #[error("empty frame data")]
    EmptyFrame,

    #[error("ssim computation failed")]
    SsimFailed,

    #[error("savgol filter failed: {0}")]
    SavgolFailed(String),

    #[error("fft failed: {0}")]
    FftFailed(String),

    #[error("decode error: {0}")]
    Decode(String),
}

pub type Result<T> = std::result::Result<T, TvaError>;
