#[cfg(feature = "compare-image")]
pub mod image_compare;

#[cfg(feature = "compare-dssim")]
pub mod dssim;

#[cfg(feature = "smooth-savgol")]
pub mod savgol;

#[cfg(feature = "decode-ffmpeg")]
pub mod video_rs;

#[cfg(feature = "decode-images")]
pub mod image_seq;

pub mod identity_smoother;
